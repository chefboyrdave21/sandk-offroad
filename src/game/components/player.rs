use bevy::prelude::*;

/// Component for marking and configuring player entities
#[derive(Component)]
pub struct Player {
    /// Player health (0.0 - 100.0)
    pub health: f32,
    /// Player name
    pub name: String,
    /// Player experience points
    pub xp: u32,
    /// Player level
    pub level: u32,
    /// Whether the player is local or remote
    pub is_local: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            health: 100.0,
            name: "Player".to_string(),
            xp: 0,
            level: 1,
            is_local: true,
        }
    }
}

/// Component for tracking player input state
#[derive(Component)]
pub struct PlayerInput {
    /// Forward/backward input (-1.0 to 1.0)
    pub throttle: f32,
    /// Left/right input (-1.0 to 1.0)
    pub steering: f32,
    /// Brake input (0.0 to 1.0)
    pub brake: f32,
    /// Handbrake input (0.0 to 1.0)
    pub handbrake: f32,
    /// Camera rotation input
    pub camera_rotation: Vec2,
    /// Camera zoom input
    pub camera_zoom: f32,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            steering: 0.0,
            brake: 0.0,
            handbrake: 0.0,
            camera_rotation: Vec2::ZERO,
            camera_zoom: 0.0,
        }
    }
} 