use bevy::prelude::*;
use crate::game::plugins::weather::{
    CloudMaterial,
    CloudParams,
    CloudNoiseTextureHandles,
    NoiseTexturePlugin,
    Weather,
};

pub struct CloudDemoPlugin;

impl Plugin for CloudDemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                MaterialPlugin::<CloudMaterial>::default(),
                NoiseTexturePlugin,
            ))
            .init_resource::<DemoState>()
            .add_systems(Startup, setup_demo)
            .add_systems(Update, (
                update_cloud_parameters,
                handle_input,
                handle_camera,
                update_ui,
            ));
    }
}

#[derive(Resource, Default)]
struct DemoState {
    cloud_handle: Option<Handle<CloudMaterial>>,
    weather: Weather,
    transition_time: f32,
}

#[derive(Component)]
struct DemoCamera {
    pub speed: f32,
    pub sensitivity: f32,
    pub orbit_distance: f32,
}

impl Default for DemoCamera {
    fn default() -> Self {
        Self {
            speed: 5.0,
            sensitivity: 0.005,
            orbit_distance: 5.0,
        }
    }
}

fn setup_demo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
    noise_textures: Res<CloudNoiseTextureHandles>,
) {
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.0, 5.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        DemoCamera::default(),
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 32000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        transform: Transform::from_xyz(0.0, -2.0, 0.0),
        ..default()
    });

    // Cloud volume
    let cloud_material = CloudMaterial {
        params: CloudParams::default(),
        base_shape_texture: noise_textures.base_shape.clone(),
        detail_texture: noise_textures.detail.clone(),
        weather_texture: noise_textures.weather.clone(),
    };
    
    let cloud_handle = cloud_materials.add(cloud_material);
    
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(shape::Box::new(20.0, 4.0, 20.0).into()),
        material: cloud_handle.clone(),
        transform: Transform::from_xyz(0.0, 8.0, 0.0),
        ..default()
    });

    // UI for controls
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Controls:\n1-6: Weather Types | WASD: Move Camera | Mouse: Look | Scroll: Zoom",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });

    // Store material handle for updates
    commands.insert_resource(DemoState {
        cloud_handle: Some(cloud_handle),
        weather: Weather::Clear,
        transition_time: 0.0,
    });
}

fn handle_input(
    keys: Res<Input<KeyCode>>,
    mut demo_state: ResMut<DemoState>,
) {
    let new_weather = if keys.just_pressed(KeyCode::Key1) {
        Some(Weather::Clear)
    } else if keys.just_pressed(KeyCode::Key2) {
        Some(Weather::Cloudy)
    } else if keys.just_pressed(KeyCode::Key3) {
        Some(Weather::Rain)
    } else if keys.just_pressed(KeyCode::Key4) {
        Some(Weather::Storm)
    } else if keys.just_pressed(KeyCode::Key5) {
        Some(Weather::Fog)
    } else if keys.just_pressed(KeyCode::Key6) {
        Some(Weather::Snow)
    } else {
        None
    };

    if let Some(weather) = new_weather {
        demo_state.weather = weather;
        demo_state.transition_time = 0.0;
    }
}

fn update_cloud_parameters(
    demo_state: Res<DemoState>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
    time: Res<Time>,
) {
    if let Some(handle) = &demo_state.cloud_handle {
        if let Some(material) = cloud_materials.get_mut(handle) {
            material.update_from_weather(&demo_state.weather, time.elapsed_seconds());
        }
    }
}

fn handle_camera(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut DemoCamera)>,
) {
    let dt = time.delta_seconds();
    
    for (mut transform, mut camera) in query.iter_mut() {
        // Handle keyboard input for movement
        let mut movement = Vec3::ZERO;
        if keys.pressed(KeyCode::W) {
            movement += transform.forward();
        }
        if keys.pressed(KeyCode::S) {
            movement -= transform.forward();
        }
        if keys.pressed(KeyCode::A) {
            movement -= transform.right();
        }
        if keys.pressed(KeyCode::D) {
            movement += transform.right();
        }
        if keys.pressed(KeyCode::Q) {
            movement -= Vec3::Y;
        }
        if keys.pressed(KeyCode::E) {
            movement += Vec3::Y;
        }
        
        transform.translation += movement * camera.speed * dt;

        // Handle mouse input for rotation
        let mut mouse_delta = Vec2::ZERO;
        for event in mouse_motion.iter() {
            mouse_delta += event.delta;
        }
        
        if keys.pressed(KeyCode::ShiftLeft) {
            let pitch = (-mouse_delta.y * camera.sensitivity).clamp(-PI / 2.0, PI / 2.0);
            let yaw = -mouse_delta.x * camera.sensitivity;
            
            transform.rotation *= Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        }

        // Handle mouse wheel for zoom
        for event in mouse_wheel.iter() {
            camera.orbit_distance -= event.y * camera.speed;
            camera.orbit_distance = camera.orbit_distance.clamp(2.0, 20.0);
            
            // Update camera position while maintaining look direction
            let forward = transform.forward();
            transform.translation = -forward * camera.orbit_distance;
        }
    }
}

fn update_ui(
    demo_state: Res<DemoState>,
    mut query: Query<&mut Text>,
) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!(
            "Controls:\n1-6: Weather Types | WASD: Move Camera | Mouse: Look | Scroll: Zoom\nCurrent Weather: {:?}",
            demo_state.weather
        );
    }
} 