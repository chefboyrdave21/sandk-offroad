use bevy::prelude::*;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::render::camera::Camera3d;
use std::f32::consts::PI;

use crate::game::plugins::lighting::{
    VolumetricLightingPlugin,
    VolumetricSettings,
};

/// Demo app showcasing volumetric lighting effects in different scenarios
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(VolumetricLightingPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (
            rotate_camera,
            cycle_weather_conditions,
            update_lights,
        ))
        .run();
}

#[derive(Resource)]
struct DemoState {
    elapsed_time: f32,
    weather_cycle_duration: f32,
    light_rotation: f32,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            elapsed_time: 0.0,
            weather_cycle_duration: 30.0, // Complete cycle every 30 seconds
            light_rotation: 0.0,
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Initialize demo state
    commands.insert_resource(DemoState::default());

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 15.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.1, 0.1, 0.15)),
                ..default()
            },
            ..default()
        },
    ));

    // Ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
        transform: Transform::from_xyz(0.0, -2.0, 0.0),
        ..default()
    });

    // Scene objects to demonstrate volumetric effects
    // Central sphere
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::UVSphere::default().into()),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: Color::rgb(2.0, 2.0, 2.0), // Bright emissive material
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Surrounding pillars
    for i in 0..6 {
        let angle = i as f32 * PI / 3.0;
        let (sin, cos) = angle.sin_cos();
        let distance = 5.0;
        
        commands.spawn(PbrBundle {
            mesh: meshes.add(shape::Capsule::default().into()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.8, 0.8),
                ..default()
            }),
            transform: Transform::from_xyz(cos * distance, 0.0, sin * distance)
                .with_scale(Vec3::new(0.5, 2.0, 0.5)),
            ..default()
        });
    }

    // Directional light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 50000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 8.0, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Point lights for additional atmosphere
    for i in 0..3 {
        let angle = i as f32 * PI * 2.0 / 3.0;
        let (sin, cos) = angle.sin_cos();
        let distance = 8.0;
        
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 5000.0,
                color: Color::rgb(0.8, 0.6, 0.3),
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(cos * distance, 3.0, sin * distance),
            ..default()
        });
    }

    // Initial volumetric settings
    commands.insert_resource(VolumetricSettings {
        density: 0.5,
        scattering: 0.8,
        absorption: 0.2,
        max_distance: 50.0,
    });
}

fn rotate_camera(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    let rotation_speed = 0.1;
    for mut transform in query.iter_mut() {
        let angle = time.elapsed_seconds() * rotation_speed;
        let radius = 15.0;
        let height = 5.0 + (time.elapsed_seconds() * 0.2).sin() * 2.0;
        
        transform.translation = Vec3::new(
            angle.cos() * radius,
            height,
            angle.sin() * radius,
        );
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn cycle_weather_conditions(
    time: Res<Time>,
    mut state: ResMut<DemoState>,
    mut settings: ResMut<VolumetricSettings>,
) {
    state.elapsed_time += time.delta_seconds();
    let cycle_progress = (state.elapsed_time / state.weather_cycle_duration) * PI * 2.0;

    // Smoothly interpolate between different atmospheric conditions
    settings.density = 0.3 + (cycle_progress.sin() * 0.3);
    settings.scattering = 0.5 + (cycle_progress.cos() * 0.3);
    settings.absorption = 0.1 + ((cycle_progress * 2.0).sin() * 0.1);
}

fn update_lights(
    time: Res<Time>,
    mut state: ResMut<DemoState>,
    mut query: Query<(&mut Transform, &mut DirectionalLight)>,
) {
    state.light_rotation += time.delta_seconds() * 0.2;
    
    for (mut transform, mut light) in query.iter_mut() {
        // Rotate the main directional light
        let angle = state.light_rotation;
        transform.translation = Vec3::new(
            angle.cos() * 10.0,
            8.0 + (angle * 0.5).sin() * 2.0,
            angle.sin() * 10.0,
        );
        transform.look_at(Vec3::ZERO, Vec3::Y);

        // Adjust light intensity based on height
        light.illuminance = 40000.0 + (angle * 0.5).sin() * 20000.0;
    }
} 