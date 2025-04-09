use bevy::prelude::*;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_rendering)
            .add_systems(Update, (
                update_camera,
                handle_particle_effects,
            ));
    }
}

#[derive(Component)]
pub struct MainCamera {
    pub target: Option<Entity>,
    pub offset: Vec3,
    pub smoothness: f32,
}

#[derive(Component)]
pub struct ParticleEffect {
    pub lifetime: f32,
    pub current_lifetime: f32,
    pub velocity: Vec3,
    pub color: Color,
}

fn setup_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Setup lighting
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Setup skybox
    commands.spawn(SkyboxBundle {
        skybox: meshes.add(Mesh::from(shape::Cube { size: 1000.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.4, 0.6, 0.8),
            ..default()
        }),
        ..default()
    });
}

fn update_camera(
    mut camera_query: Query<(&mut Transform, &MainCamera)>,
    target_query: Query<&Transform, Without<MainCamera>>,
    time: Res<Time>,
) {
    for (mut transform, camera) in camera_query.iter_mut() {
        if let Some(target_entity) = camera.target {
            if let Ok(target_transform) = target_query.get(target_entity) {
                let target_position = target_transform.translation + camera.offset;
                transform.translation = transform.translation.lerp(
                    target_position,
                    camera.smoothness * time.delta_seconds(),
                );
                transform.look_at(target_transform.translation, Vec3::Y);
            }
        }
    }
}

fn handle_particle_effects(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ParticleEffect, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut effect, mut transform) in query.iter_mut() {
        effect.current_lifetime -= time.delta_seconds();
        if effect.current_lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation += effect.velocity * time.delta_seconds();
        }
    }
} 