use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;

use super::collision::{ParticleCollisionBuffers, update_collision_objects};

pub struct ParticleCollisionPlugin;

impl Plugin for ParticleCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_collision_objects);

        // Initialize collision buffers when render device is available
        app.add_systems(Startup, |render_device: Res<RenderDevice>| {
            let collision_buffers = ParticleCollisionBuffers::new(
                &render_device,
                32,  // max_planes
                64,  // max_spheres
                32,  // max_boxes
            );
            app.world.insert_resource(collision_buffers);
        });
    }
}

// Re-export commonly used types
pub use super::collision::{
    CollisionBoxComponent,
    CollisionPlaneComponent,
    CollisionSphereComponent,
    spawn_collision_box,
    spawn_collision_plane,
    spawn_collision_sphere,
}; 