use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_systems(Startup, setup_core)
            .add_systems(Update, handle_game_state);
    }
}

fn setup_core(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn handle_game_state(
    state: Res<State<GameState>>,
    keyboard: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    match state.get() {
        GameState::Loading => {
            // Transition to main menu once loading is complete
            next_state.set(GameState::MainMenu);
        }
        GameState::MainMenu => {
            if keyboard.just_pressed(KeyCode::Return) {
                next_state.set(GameState::Playing);
            }
        }
        GameState::Playing => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Paused);
            }
        }
        GameState::Paused => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Playing);
            }
        }
        GameState::GameOver => {
            if keyboard.just_pressed(KeyCode::Return) {
                next_state.set(GameState::MainMenu);
            }
        }
    }
} 