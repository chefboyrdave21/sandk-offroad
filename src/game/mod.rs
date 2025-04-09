use bevy::prelude::*;
use crate::physics::Vehicle;
use crate::core::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_game)
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, (
                update_game_state,
                handle_player_input,
            ));
    }
}

#[derive(Resource)]
pub struct GameSettings {
    pub difficulty: f32,
    pub vehicle_type: String,
    pub track_name: String,
}

fn setup_game(mut commands: Commands) {
    commands.insert_resource(GameSettings {
        difficulty: 1.0,
        vehicle_type: "default".to_string(),
        track_name: "desert".to_string(),
    });
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn player vehicle
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.2, 0.2),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Vehicle {
            max_speed: 30.0,
            acceleration: 10.0,
            turn_rate: 2.0,
            current_speed: 0.0,
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 1.0),
    ));
}

fn update_game_state(
    mut commands: Commands,
    state: Res<State<GameState>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Return) {
        match state.get() {
            GameState::MainMenu => {
                commands.insert_resource(NextState(Some(GameState::Playing)));
            }
            GameState::GameOver => {
                commands.insert_resource(NextState(Some(GameState::MainMenu)));
            }
            _ => {}
        }
    }
}

fn handle_player_input(
    mut vehicle_query: Query<&mut Vehicle>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for mut vehicle in vehicle_query.iter_mut() {
        if keyboard.pressed(KeyCode::W) {
            vehicle.current_speed = (vehicle.current_speed + vehicle.acceleration * time.delta_seconds())
                .min(vehicle.max_speed);
        } else if keyboard.pressed(KeyCode::S) {
            vehicle.current_speed = (vehicle.current_speed - vehicle.acceleration * time.delta_seconds())
                .max(-vehicle.max_speed * 0.5);
        } else {
            // Apply friction
            vehicle.current_speed = vehicle.current_speed * (1.0 - time.delta_seconds() * 2.0);
        }
    }
} 