use bevy::prelude::*;
use super::trail_material::TrailMaterial;
use super::trail_mesh::{TrailMesh, update_trail_meshes};

#[derive(Component, Default)]
pub struct Trail {
    pub max_points: usize,
    pub point_distance: f32,
    pub fade_time: f32,
    pub width: f32,
}

#[derive(Component)]
pub struct TrailPoint {
    pub position: Vec3,
    pub velocity: Vec3,
    pub creation_time: f32,
}

impl Default for Trail {
    fn default() -> Self {
        Self {
            max_points: 50,
            point_distance: 0.1,
            fade_time: 1.0,
            width: 0.2,
        }
    }
}

pub struct TrailPlugin;

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_trails,
            update_trail_materials,
            update_trail_meshes.after(update_trails),
        ));
    }
}

#[derive(Component)]
pub struct TrailPoints(pub Vec<TrailPoint>);

fn update_trails(
    time: Res<Time>,
    mut query: Query<(&mut TrailPoints, &Trail, &Transform)>,
) {
    for (mut trail_points, trail, transform) in query.iter_mut() {
        let current_time = time.elapsed_seconds();
        let current_pos = transform.translation;

        // Remove old points
        trail_points.0.retain(|point| {
            current_time - point.creation_time <= trail.fade_time
        });

        // Add new point if needed
        if trail_points.0.is_empty() || 
           (current_pos - trail_points.0.last().unwrap().position).length() >= trail.point_distance {
            let velocity = if trail_points.0.is_empty() {
                Vec3::ZERO
            } else {
                (current_pos - trail_points.0.last().unwrap().position) / time.delta_seconds()
            };

            trail_points.0.push(TrailPoint {
                position: current_pos,
                velocity,
                creation_time: current_time,
            });
        }

        // Limit number of points
        if trail_points.0.len() > trail.max_points {
            trail_points.0.drain(0..trail_points.0.len() - trail.max_points);
        }
    }
}

fn update_trail_materials(
    time: Res<Time>,
    mut materials: Query<&mut TrailMaterial>,
) {
    for mut material in materials.iter_mut() {
        material.update(time.elapsed_seconds());
    }
}

// Helper function to spawn a trail entity
pub fn spawn_trail(
    commands: &mut Commands,
    parent: Entity,
    max_points: usize,
    point_distance: f32,
    fade_time: f32,
    width: f32,
) {
    commands.entity(parent).insert((
        Trail {
            max_points,
            point_distance,
            fade_time,
            width,
        },
        TrailPoints(Vec::new()),
        TrailMesh,
    ));
} 