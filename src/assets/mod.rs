use bevy::prelude::*;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>();
    }
}

#[derive(Resource, Default)]
pub struct GameAssets {
    // Add asset handles here as needed
} 