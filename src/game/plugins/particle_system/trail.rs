use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use std::collections::VecDeque;

#[derive(Component)]
pub struct ParticleTrail {
    pub max_points: usize,
    pub min_distance: f32,
    pub base_width: f32,
    pub fade_time: f32,
    pub color_gradient: Vec<Vec4>,
    pub texture: Option<Handle<Image>>,
    pub texture_scroll_speed: f32,
    pub velocity_width_factor: f32,
    pub min_width_scale: f32,
    pub max_width_scale: f32,
    points: VecDeque<TrailPoint>,
    mesh: Option<Handle<Mesh>>,
}

#[derive(Clone)]
struct TrailPoint {
    position: Vec3,
    normal: Vec3,
    velocity: Vec3,
    width_scale: f32,
    time: f32,
    custom_data: Vec4,  // For special effects (e.g. turbulence, glow, distortion)
}

impl Default for ParticleTrail {
    fn default() -> Self {
        Self {
            max_points: 30,
            min_distance: 0.1,
            base_width: 0.5,
            fade_time: 1.0,
            color_gradient: vec![Vec4::ONE],
            texture: None,
            texture_scroll_speed: 0.0,
            velocity_width_factor: 0.2,
            min_width_scale: 0.5,
            max_width_scale: 2.0,
            points: VecDeque::new(),
            mesh: None,
        }
    }
}

impl ParticleTrail {
    pub fn new(max_points: usize, min_distance: f32, base_width: f32) -> Self {
        Self {
            max_points,
            min_distance,
            base_width,
            ..Default::default()
        }
    }

    fn update_points(&mut self, position: Vec3, velocity: Vec3, time: f32, custom_data: Vec4) {
        // Remove old points
        while let Some(point) = self.points.front() {
            if time - point.time > self.fade_time {
                self.points.pop_front();
            } else {
                break;
            }
        }

        // Calculate width scale based on velocity
        let speed = velocity.length();
        let width_scale = (1.0 + speed * self.velocity_width_factor)
            .clamp(self.min_width_scale, self.max_width_scale);

        // Add new point if far enough from last point
        if let Some(last_point) = self.points.back() {
            if position.distance(last_point.position) >= self.min_distance {
                self.add_point(position, velocity, width_scale, time, custom_data);
            }
        } else {
            self.add_point(position, velocity, width_scale, time, custom_data);
        }

        // Limit number of points
        while self.points.len() > self.max_points {
            self.points.pop_front();
        }
    }

    fn add_point(&mut self, position: Vec3, velocity: Vec3, width_scale: f32, time: f32, custom_data: Vec4) {
        let normal = if self.points.len() >= 2 {
            let prev = self.points[self.points.len() - 2].position;
            let curr = self.points[self.points.len() - 1].position;
            (position - prev).normalize()
        } else {
            Vec3::Y
        };

        self.points.push_back(TrailPoint {
            position,
            normal,
            velocity,
            width_scale,
            time,
            custom_data,
        });
    }

    fn generate_mesh(&self, time: f32) -> Mesh {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut colors = Vec::new();
        let mut velocities = Vec::new();  // New velocity attribute
        let mut custom_data = Vec::new(); // New custom data attribute
        let mut indices = Vec::new();

        if self.points.len() < 2 {
            return Mesh::new(PrimitiveTopology::TriangleList);
        }

        // Calculate texture scroll offset
        let tex_scroll = time * self.texture_scroll_speed;

        // Generate vertices
        for (i, point) in self.points.iter().enumerate() {
            let t = (time - point.time) / self.fade_time;
            let color = self.sample_gradient(1.0 - t.clamp(0.0, 1.0));
            
            let right = if i < self.points.len() - 1 {
                (self.points[i + 1].position - point.position).normalize()
            } else {
                (point.position - self.points[i - 1].position).normalize()
            };
            
            let up = right.cross(point.normal).normalize();
            let width = self.base_width * point.width_scale;
            let offset = up * width * 0.5;

            // Add vertices for both sides of the trail
            positions.push((point.position + offset).to_array());
            positions.push((point.position - offset).to_array());
            
            normals.push(point.normal.to_array());
            normals.push(point.normal.to_array());
            
            // UV coordinates with scrolling
            let u = (i as f32 / (self.points.len() - 1) as f32) + tex_scroll;
            uvs.push([u, 0.0]);
            uvs.push([u, 1.0]);
            
            colors.push(color.to_array());
            colors.push(color.to_array());

            // Add velocity data
            velocities.push(point.velocity.to_array());
            velocities.push(point.velocity.to_array());

            // Add custom effect data
            custom_data.push(point.custom_data.to_array());
            custom_data.push(point.custom_data.to_array());

            // Generate indices for triangle strip
            if i < self.points.len() - 1 {
                let base = i * 2;
                indices.push(base as u32);
                indices.push((base + 1) as u32);
                indices.push((base + 2) as u32);
                indices.push((base + 1) as u32);
                indices.push((base + 3) as u32);
                indices.push((base + 2) as u32);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_attribute("Velocity", velocities);  // Custom velocity attribute
        mesh.insert_attribute("CustomData", custom_data); // Custom effect data attribute
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh
    }

    fn sample_gradient(&self, t: f32) -> Vec4 {
        if self.color_gradient.len() == 1 {
            return self.color_gradient[0];
        }

        let index = (t * (self.color_gradient.len() - 1) as f32).floor() as usize;
        let next_index = (index + 1).min(self.color_gradient.len() - 1);
        let local_t = t * (self.color_gradient.len() - 1) as f32 - index as f32;

        self.color_gradient[index].lerp(self.color_gradient[next_index], local_t)
    }

    // Builder methods for configuration
    pub fn with_texture(mut self, texture: Handle<Image>, scroll_speed: f32) -> Self {
        self.texture = Some(texture);
        self.texture_scroll_speed = scroll_speed;
        self
    }

    pub fn with_velocity_width(mut self, factor: f32, min_scale: f32, max_scale: f32) -> Self {
        self.velocity_width_factor = factor;
        self.min_width_scale = min_scale;
        self.max_width_scale = max_scale;
        self
    }
}

pub fn update_particle_trails(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &Velocity, &mut ParticleTrail)>,
) {
    for (entity, transform, velocity, mut trail) in query.iter_mut() {
        // Example custom data - can be modified based on specific effects needed
        let custom_data = Vec4::new(
            time.elapsed_seconds(), // Time for animation
            velocity.0.length(),    // Speed for effects
            0.0,                   // Available for custom effects
            0.0,                   // Available for custom effects
        );

        trail.update_points(
            transform.translation, 
            velocity.0, 
            time.elapsed_seconds(),
            custom_data
        );
        
        // Generate and update mesh
        let mesh = trail.generate_mesh(time.elapsed_seconds());
        if let Some(mesh_handle) = &trail.mesh {
            meshes.insert(mesh_handle.clone(), mesh);
        } else {
            let mesh_handle = meshes.add(mesh);
            trail.mesh = Some(mesh_handle.clone());
            
            // Add mesh component if it doesn't exist
            if !commands.get_entity(entity).unwrap().contains::<Handle<Mesh>>() {
                commands.entity(entity).insert(mesh_handle);
            }
        }
    }
}

pub struct ParticleTrailPlugin;

impl Plugin for ParticleTrailPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_particle_trails);
    }
} 