use bevy::prelude::*;
use bevy::asset::LoadState;
use bevy::pbr::StandardMaterial;
use bevy::scene::Scene;
use bevy::audio::AudioSource;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
           .init_resource::<AssetLoadingState>()
           .add_systems(Update, check_asset_loading_progress);
    }
}

/// Resource to hold all game asset handles
#[derive(Resource)]
pub struct GameAssets {
    // Vehicle assets
    pub vehicle_models: Vec<Handle<Scene>>,
    pub vehicle_textures: Vec<Handle<Image>>,
    pub vehicle_materials: Vec<Handle<StandardMaterial>>,
    
    // Terrain assets
    pub terrain_textures: Vec<Handle<Image>>,
    pub terrain_materials: Vec<Handle<StandardMaterial>>,
    
    // Audio assets
    pub engine_sounds: Vec<Handle<AudioSource>>,
    pub environment_sounds: Vec<Handle<AudioSource>>,
    pub impact_sounds: Vec<Handle<AudioSource>>,
    
    // UI assets
    pub ui_textures: Vec<Handle<Image>>,
    pub fonts: Vec<Handle<Font>>,
    
    // Effect assets
    pub particle_textures: Vec<Handle<Image>>,
    pub skybox: Option<Handle<Image>>,
}

impl Default for GameAssets {
    fn default() -> Self {
        Self {
            vehicle_models: Vec::new(),
            vehicle_textures: Vec::new(),
            vehicle_materials: Vec::new(),
            terrain_textures: Vec::new(),
            terrain_materials: Vec::new(),
            engine_sounds: Vec::new(),
            environment_sounds: Vec::new(),
            impact_sounds: Vec::new(),
            ui_textures: Vec::new(),
            fonts: Vec::new(),
            particle_textures: Vec::new(),
            skybox: None,
        }
    }
}

/// Resource to track asset loading progress
#[derive(Resource, Default)]
pub struct AssetLoadingState {
    pub total_assets: usize,
    pub loaded_assets: usize,
    pub failed_assets: Vec<String>,
    pub loading_complete: bool,
}

impl AssetLoadingState {
    /// Get loading progress as a percentage
    pub fn progress(&self) -> f32 {
        if self.total_assets == 0 {
            return 1.0;
        }
        self.loaded_assets as f32 / self.total_assets as f32
    }
}

/// System to check asset loading progress
fn check_asset_loading_progress(
    asset_server: Res<AssetServer>,
    mut loading_state: ResMut<AssetLoadingState>,
    game_assets: Res<GameAssets>,
) {
    // Collect all asset handles
    let mut handles = Vec::new();
    handles.extend(game_assets.vehicle_models.iter());
    handles.extend(game_assets.vehicle_textures.iter());
    handles.extend(game_assets.vehicle_materials.iter());
    handles.extend(game_assets.terrain_textures.iter());
    handles.extend(game_assets.terrain_materials.iter());
    handles.extend(game_assets.engine_sounds.iter());
    handles.extend(game_assets.environment_sounds.iter());
    handles.extend(game_assets.impact_sounds.iter());
    handles.extend(game_assets.ui_textures.iter());
    handles.extend(game_assets.fonts.iter());
    handles.extend(game_assets.particle_textures.iter());
    if let Some(skybox) = &game_assets.skybox {
        handles.push(skybox);
    }

    // Update loading state
    loading_state.total_assets = handles.len();
    loading_state.loaded_assets = handles
        .iter()
        .filter(|handle| {
            matches!(
                asset_server.get_load_state(handle.id()),
                Some(LoadState::Loaded)
            )
        })
        .count();

    // Check for failed assets
    loading_state.failed_assets = handles
        .iter()
        .filter_map(|handle| {
            if matches!(
                asset_server.get_load_state(handle.id()),
                Some(LoadState::Failed)
            ) {
                Some(format!("{:?}", handle.id()))
            } else {
                None
            }
        })
        .collect();

    // Update completion state
    loading_state.loading_complete = loading_state.loaded_assets == loading_state.total_assets;
}

/// Helper functions for loading assets
impl GameAssets {
    /// Load all game assets from their respective directories
    pub fn load_all(&mut self, asset_server: &AssetServer) {
        // Load vehicle assets
        self.vehicle_models = asset_server.load_folder("models/vehicles").unwrap_or_default();
        self.vehicle_textures = asset_server.load_folder("textures/vehicles").unwrap_or_default();
        
        // Load terrain assets
        self.terrain_textures = asset_server.load_folder("textures/terrain").unwrap_or_default();
        
        // Load audio assets
        self.engine_sounds = asset_server.load_folder("audio/engine").unwrap_or_default();
        self.environment_sounds = asset_server.load_folder("audio/environment").unwrap_or_default();
        self.impact_sounds = asset_server.load_folder("audio/impacts").unwrap_or_default();
        
        // Load UI assets
        self.ui_textures = asset_server.load_folder("textures/ui").unwrap_or_default();
        self.fonts = asset_server.load_folder("fonts").unwrap_or_default();
        
        // Load effect assets
        self.particle_textures = asset_server.load_folder("textures/particles").unwrap_or_default();
        self.skybox = asset_server.load_folder("textures/skybox").ok().and_then(|mut v| v.pop());
    }

    /// Hot reload all assets (useful during development)
    pub fn hot_reload(&self, asset_server: &AssetServer) {
        let handles = self.get_all_handles();
        for handle in handles {
            asset_server.reload_asset(handle.id());
        }
    }

    /// Get all asset handles as a vector
    fn get_all_handles(&self) -> Vec<&Handle<dyn bevy::asset::Asset>> {
        let mut handles = Vec::new();
        handles.extend(self.vehicle_models.iter().map(|h| h as _));
        handles.extend(self.vehicle_textures.iter().map(|h| h as _));
        handles.extend(self.vehicle_materials.iter().map(|h| h as _));
        handles.extend(self.terrain_textures.iter().map(|h| h as _));
        handles.extend(self.terrain_materials.iter().map(|h| h as _));
        handles.extend(self.engine_sounds.iter().map(|h| h as _));
        handles.extend(self.environment_sounds.iter().map(|h| h as _));
        handles.extend(self.impact_sounds.iter().map(|h| h as _));
        handles.extend(self.ui_textures.iter().map(|h| h as _));
        handles.extend(self.fonts.iter().map(|h| h as _));
        handles.extend(self.particle_textures.iter().map(|h| h as _));
        if let Some(skybox) = &self.skybox {
            handles.push(skybox as _);
        }
        handles
    }
} 