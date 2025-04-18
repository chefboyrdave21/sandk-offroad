use bevy::prelude::*;
use crate::game::plugins::weather::*;
use crate::game::plugins::particle_system::presets::ParticlePresets;

pub struct WeatherDemoPlugin;

impl Plugin for WeatherDemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene)
            .add_systems(Update, (handle_input, update_ui));
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Add light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Add ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });

    // Add some scenery
    for i in -2..=2 {
        for j in -2..=2 {
            if i == 0 && j == 0 { continue; }
            
            commands.spawn(PbrBundle {
                mesh: meshes.add(shape::Box::new(1.0, 3.0, 1.0).into()),
                material: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
                transform: Transform::from_xyz(i as f32 * 4.0, 1.0, j as f32 * 4.0),
                ..default()
            });
        }
    }

    // Add UI
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
            "Press 1-5 to change weather: 1) Clear 2) Rain 3) Snow 4) Fog 5) Storm",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn handle_input(
    keyboard: Res<Input<KeyCode>>,
    mut weather_state: ResMut<WeatherState>,
) {
    if keyboard.just_pressed(KeyCode::Key1) {
        weather_state.target_weather = Weather::Clear;
    } else if keyboard.just_pressed(KeyCode::Key2) {
        weather_state.target_weather = Weather::Rain;
    } else if keyboard.just_pressed(KeyCode::Key3) {
        weather_state.target_weather = Weather::Snow;
    } else if keyboard.just_pressed(KeyCode::Key4) {
        weather_state.target_weather = Weather::Fog;
    } else if keyboard.just_pressed(KeyCode::Key5) {
        weather_state.target_weather = Weather::Storm;
    }
}

fn update_ui(
    weather_state: Res<WeatherState>,
    mut query: Query<&mut Text>,
) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!(
            "Current Weather: {:?} ({}% transition) - Press 1-5 to change weather",
            weather_state.current_weather,
            (weather_state.transition_progress * 100.0) as i32
        );
    }
} 