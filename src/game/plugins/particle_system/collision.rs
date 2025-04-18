use bevy::{
    prelude::*,
    render::{
        render_resource::{Buffer, BufferDescriptor, BufferUsages},
        renderer::RenderDevice,
    },
};
use bytemuck::{Pod, Zeroable};

use super::ParticleSystem;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CollisionPlane {
    pub normal: Vec3,
    pub distance: f32,
    pub friction: f32,
    pub restitution: f32,
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CollisionSphere {
    pub center: Vec3,
    pub radius: f32,
    pub friction: f32,
    pub restitution: f32,
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CollisionBox {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub rotation: Vec4, // quaternion
    pub friction: f32,
    pub restitution: f32,
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CollisionParams {
    pub num_planes: u32,
    pub num_spheres: u32,
    pub num_boxes: u32,
    pub delta_time: f32,
    pub collision_damping: f32,
    pub friction_damping: f32,
    _padding: [f32; 2],
}

impl Default for CollisionParams {
    fn default() -> Self {
        Self {
            num_planes: 0,
            num_spheres: 0,
            num_boxes: 0,
            delta_time: 0.016,
            collision_damping: 0.98,
            friction_damping: 0.5,
            _padding: [0.0; 2],
        }
    }
}

#[derive(Resource)]
pub struct ParticleCollisionBuffers {
    pub planes_buffer: Buffer,
    pub spheres_buffer: Buffer,
    pub boxes_buffer: Buffer,
    pub params_buffer: Buffer,
    pub max_planes: u32,
    pub max_spheres: u32,
    pub max_boxes: u32,
}

impl ParticleCollisionBuffers {
    pub fn new(device: &RenderDevice, max_planes: u32, max_spheres: u32, max_boxes: u32) -> Self {
        let planes_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particle Collision Planes Buffer"),
            size: (max_planes * std::mem::size_of::<CollisionPlane>() as u32) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let spheres_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particle Collision Spheres Buffer"),
            size: (max_spheres * std::mem::size_of::<CollisionSphere>() as u32) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let boxes_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particle Collision Boxes Buffer"),
            size: (max_boxes * std::mem::size_of::<CollisionBox>() as u32) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particle Collision Params Buffer"),
            size: std::mem::size_of::<CollisionParams>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            planes_buffer,
            spheres_buffer,
            boxes_buffer,
            params_buffer,
            max_planes,
            max_spheres,
            max_boxes,
        }
    }
}

// Components for collision objects
#[derive(Component)]
pub struct CollisionPlaneComponent {
    pub normal: Vec3,
    pub distance: f32,
    pub friction: f32,
    pub restitution: f32,
}

#[derive(Component)]
pub struct CollisionSphereComponent {
    pub radius: f32,
    pub friction: f32,
    pub restitution: f32,
}

#[derive(Component)]
pub struct CollisionBoxComponent {
    pub half_extents: Vec3,
    pub friction: f32,
    pub restitution: f32,
}

// Systems for updating collision objects
pub fn update_collision_objects(
    mut commands: Commands,
    planes: Query<(Entity, &Transform, &CollisionPlaneComponent)>,
    spheres: Query<(Entity, &Transform, &CollisionSphereComponent)>,
    boxes: Query<(Entity, &Transform, &CollisionBoxComponent)>,
    time: Res<Time>,
    render_device: Res<RenderDevice>,
    mut collision_buffers: ResMut<ParticleCollisionBuffers>,
) {
    let mut planes_data = Vec::new();
    let mut spheres_data = Vec::new();
    let mut boxes_data = Vec::new();

    // Update planes
    for (_entity, transform, plane) in planes.iter() {
        if planes_data.len() >= collision_buffers.max_planes as usize {
            break;
        }
        planes_data.push(CollisionPlane {
            normal: plane.normal,
            distance: plane.distance,
            friction: plane.friction,
            restitution: plane.restitution,
            _padding: [0.0; 2],
        });
    }

    // Update spheres
    for (_entity, transform, sphere) in spheres.iter() {
        if spheres_data.len() >= collision_buffers.max_spheres as usize {
            break;
        }
        spheres_data.push(CollisionSphere {
            center: transform.translation,
            radius: sphere.radius,
            friction: sphere.friction,
            restitution: sphere.restitution,
            _padding: [0.0; 2],
        });
    }

    // Update boxes
    for (_entity, transform, box_comp) in boxes.iter() {
        if boxes_data.len() >= collision_buffers.max_boxes as usize {
            break;
        }
        boxes_data.push(CollisionBox {
            center: transform.translation,
            half_extents: box_comp.half_extents,
            rotation: Vec4::new(
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ),
            friction: box_comp.friction,
            restitution: box_comp.restitution,
            _padding: [0.0; 2],
        });
    }

    // Update collision parameters
    let params = CollisionParams {
        num_planes: planes_data.len() as u32,
        num_spheres: spheres_data.len() as u32,
        num_boxes: boxes_data.len() as u32,
        delta_time: time.delta_seconds(),
        collision_damping: 0.98,
        friction_damping: 0.5,
        _padding: [0.0; 2],
    };

    // Queue buffer updates
    render_device.queue().write_buffer(
        &collision_buffers.planes_buffer,
        0,
        bytemuck::cast_slice(&planes_data),
    );
    render_device.queue().write_buffer(
        &collision_buffers.spheres_buffer,
        0,
        bytemuck::cast_slice(&spheres_data),
    );
    render_device.queue().write_buffer(
        &collision_buffers.boxes_buffer,
        0,
        bytemuck::cast_slice(&boxes_data),
    );
    render_device.queue().write_buffer(
        &collision_buffers.params_buffer,
        0,
        bytemuck::bytes_of(&params),
    );
}

// Helper functions for creating collision objects
pub fn spawn_collision_plane(
    commands: &mut Commands,
    normal: Vec3,
    distance: f32,
    friction: f32,
    restitution: f32,
) -> Entity {
    commands
        .spawn((
            TransformBundle::default(),
            CollisionPlaneComponent {
                normal,
                distance,
                friction,
                restitution,
            },
        ))
        .id()
}

pub fn spawn_collision_sphere(
    commands: &mut Commands,
    position: Vec3,
    radius: f32,
    friction: f32,
    restitution: f32,
) -> Entity {
    commands
        .spawn((
            TransformBundle::from_transform(Transform::from_translation(position)),
            CollisionSphereComponent {
                radius,
                friction,
                restitution,
            },
        ))
        .id()
}

pub fn spawn_collision_box(
    commands: &mut Commands,
    position: Vec3,
    rotation: Quat,
    half_extents: Vec3,
    friction: f32,
    restitution: f32,
) -> Entity {
    commands
        .spawn((
            TransformBundle::from_transform(Transform {
                translation: position,
                rotation,
                ..default()
            }),
            CollisionBoxComponent {
                half_extents,
                friction,
                restitution,
            },
        ))
        .id()
} 