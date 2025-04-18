use bevy::prelude::*;
use bevy::asset::{LoadState, AssetServer};
use bevy::pbr::StandardMaterial;
use bevy::scene::Scene;
use bevy::audio::AudioSource;
use std::collections::{HashMap, VecDeque};

/// Asset loading priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    Critical,   // Must be loaded before game starts (UI, core assets)
    High,       // Load as soon as possible (player vehicle, current level)
    Medium,     // Load during gameplay (other vehicles, effects)
    Low,        // Can be loaded in background (unused assets)
}

/// Represents an asset to be loaded with its priority
#[derive(Debug)]
struct PendingAsset<T: bevy::asset::Asset> {
    path: String,
    priority: LoadPriority,
    handle: Handle<T>,
}

/// Tracks the loading state of game assets
#[derive(Resource)]
pub struct AssetLoadingState {
    pub total_assets: usize,
    pub loaded_assets: usize,
    pub failed_assets: usize,
    pub is_complete: bool,
    pub loading_queue: VecDeque<String>,
    pub current_priority: LoadPriority,
}

impl Default for AssetLoadingState {
    fn default() -> Self {
        Self {
            total_assets: 0,
            loaded_assets: 0,
            failed_assets: 0,
            is_complete: false,
            loading_queue: VecDeque::new(),
            current_priority: LoadPriority::Critical,
        }
    }
}

/// Resource that holds all game assets
#[derive(Resource)]
pub struct GameAssets {
    // Vehicle assets
    pub vehicle_models: HashMap<String, Handle<Scene>>,
    pub vehicle_textures: HashMap<String, Handle<Image>>,
    pub vehicle_materials: HashMap<String, Handle<StandardMaterial>>,
    pub vehicle_configs: HashMap<String, Handle<std::fs::File>>, // Vehicle configuration files
    
    // Audio assets
    pub engine_sounds: HashMap<String, Handle<AudioSource>>,
    pub environment_sounds: HashMap<String, Handle<AudioSource>>,
    pub music_tracks: HashMap<String, Handle<AudioSource>>,
    pub ui_sounds: HashMap<String, Handle<AudioSource>>,
    pub radio_stations: HashMap<String, Handle<AudioSource>>, // CB radio stations
    pub voice_lines: HashMap<String, Handle<AudioSource>>, // NPC/Radio DJ voice lines
    
    // UI assets
    pub ui_textures: HashMap<String, Handle<Image>>,
    pub ui_icons: HashMap<String, Handle<Image>>, // Separate icons for better organization
    pub fonts: HashMap<String, Handle<Font>>,
    pub ui_animations: HashMap<String, Handle<Scene>>, // UI animation assets
    
    // Effect assets
    pub particle_textures: HashMap<String, Handle<Image>>,
    pub weather_effects: HashMap<String, Handle<Scene>>,
    pub decal_textures: HashMap<String, Handle<Image>>, // Tire tracks, damage marks, etc.
    pub trail_markers: HashMap<String, Handle<Scene>>, // Trail/checkpoint markers
    
    // Terrain assets
    pub terrain_textures: HashMap<String, Handle<Image>>, // Base terrain textures
    pub terrain_materials: HashMap<String, Handle<StandardMaterial>>,
    pub terrain_heightmaps: HashMap<String, Handle<Image>>,
    pub terrain_objects: HashMap<String, Handle<Scene>>, // Trees, rocks, etc.
    
    // Shader assets
    pub custom_shaders: HashMap<String, Handle<Shader>>,
    pub shader_materials: HashMap<String, Handle<StandardMaterial>>,
}

impl Default for GameAssets {
    fn default() -> Self {
        Self {
            // Vehicle assets
            vehicle_models: HashMap::new(),
            vehicle_textures: HashMap::new(),
            vehicle_materials: HashMap::new(),
            vehicle_configs: HashMap::new(),
            
            // Audio assets
            engine_sounds: HashMap::new(),
            environment_sounds: HashMap::new(),
            music_tracks: HashMap::new(),
            ui_sounds: HashMap::new(),
            radio_stations: HashMap::new(),
            voice_lines: HashMap::new(),
            
            // UI assets
            ui_textures: HashMap::new(),
            ui_icons: HashMap::new(),
            fonts: HashMap::new(),
            ui_animations: HashMap::new(),
            
            // Effect assets
            particle_textures: HashMap::new(),
            weather_effects: HashMap::new(),
            decal_textures: HashMap::new(),
            trail_markers: HashMap::new(),
            
            // Terrain assets
            terrain_textures: HashMap::new(),
            terrain_materials: HashMap::new(),
            terrain_heightmaps: HashMap::new(),
            terrain_objects: HashMap::new(),
            
            // Shader assets
            custom_shaders: HashMap::new(),
            shader_materials: HashMap::new(),
        }
    }
}

impl GameAssets {
    /// Load all game assets from their respective directories with prioritization
    pub fn load_all(&mut self, asset_server: &AssetServer) -> AssetLoadingState {
        let mut loading_state = AssetLoadingState::default();
        
        // Critical priority assets (UI, core assets)
        self.load_directory_with_priority("ui/textures", "png", &mut self.ui_textures, asset_server, &mut loading_state, LoadPriority::Critical);
        self.load_directory_with_priority("ui/fonts", "ttf", &mut self.fonts, asset_server, &mut loading_state, LoadPriority::Critical);
        self.load_directory_with_priority("ui/icons", "png", &mut self.ui_icons, asset_server, &mut loading_state, LoadPriority::Critical);
        
        // High priority assets (player vehicle, current level)
        self.load_directory_with_priority("vehicles/models", "gltf", &mut self.vehicle_models, asset_server, &mut loading_state, LoadPriority::High);
        self.load_directory_with_priority("vehicles/textures", "png", &mut self.vehicle_textures, asset_server, &mut loading_state, LoadPriority::High);
        self.load_directory_with_priority("vehicles/configs", "json", &mut self.vehicle_configs, asset_server, &mut loading_state, LoadPriority::High);
        
        // Medium priority assets (effects, terrain)
        self.load_directory_with_priority("effects/particles", "png", &mut self.particle_textures, asset_server, &mut loading_state, LoadPriority::Medium);
        self.load_directory_with_priority("effects/weather", "gltf", &mut self.weather_effects, asset_server, &mut loading_state, LoadPriority::Medium);
        self.load_directory_with_priority("terrain/textures", "png", &mut self.terrain_textures, asset_server, &mut loading_state, LoadPriority::Medium);
        self.load_directory_with_priority("terrain/heightmaps", "png", &mut self.terrain_heightmaps, asset_server, &mut loading_state, LoadPriority::Medium);
        
        // Low priority assets (audio, additional content)
        self.load_directory_with_priority("audio/engine", "ogg", &mut self.engine_sounds, asset_server, &mut loading_state, LoadPriority::Low);
        self.load_directory_with_priority("audio/environment", "ogg", &mut self.environment_sounds, asset_server, &mut loading_state, LoadPriority::Low);
        self.load_directory_with_priority("audio/music", "ogg", &mut self.music_tracks, asset_server, &mut loading_state, LoadPriority::Low);
        self.load_directory_with_priority("audio/radio", "ogg", &mut self.radio_stations, asset_server, &mut loading_state, LoadPriority::Low);
        self.load_directory_with_priority("audio/voice", "ogg", &mut self.voice_lines, asset_server, &mut loading_state, LoadPriority::Low);
        
        loading_state
    }
    
    /// Load assets from a directory into a HashMap with priority
    fn load_directory_with_priority<T: bevy::asset::Asset>(
        &mut self,
        directory: &str,
        extension: &str,
        map: &mut HashMap<String, Handle<T>>,
        asset_server: &AssetServer,
        loading_state: &mut AssetLoadingState,
        priority: LoadPriority,
    ) {
        if let Ok(paths) = std::fs::read_dir(directory) {
            for path in paths.flatten() {
                if let Some(filename) = path.file_name().to_str() {
                    if filename.ends_with(extension) {
                        let key = filename.trim_end_matches(extension).trim_end_matches('.').to_string();
                        let asset_path = format!("{}/{}", directory, filename);
                        
                        // Add to loading queue based on priority
                        loading_state.loading_queue.push_back(asset_path.clone());
                        map.insert(key, asset_server.load(&asset_path));
                        loading_state.total_assets += 1;
                    }
                }
            }
        }
    }
    
    /// Check the loading progress of all assets
    pub fn check_loading_progress(&self, asset_server: &AssetServer) -> AssetLoadingState {
        let mut state = AssetLoadingState::default();
        
        let check_map = |map: &HashMap<String, Handle<_>>| {
            for handle in map.values() {
                state.total_assets += 1;
                match asset_server.get_load_state(handle) {
                    Some(LoadState::Loaded) => state.loaded_assets += 1,
                    Some(LoadState::Failed) => state.failed_assets += 1,
                    _ => {}
                }
            }
        };
        
        // Check all asset maps in priority order
        // Critical
        check_map(&self.ui_textures);
        check_map(&self.fonts);
        check_map(&self.ui_icons);
        
        // High
        check_map(&self.vehicle_models);
        check_map(&self.vehicle_textures);
        check_map(&self.vehicle_configs);
        
        // Medium
        check_map(&self.particle_textures);
        check_map(&self.weather_effects);
        check_map(&self.terrain_textures);
        check_map(&self.terrain_heightmaps);
        check_map(&self.terrain_objects);
        
        // Low
        check_map(&self.engine_sounds);
        check_map(&self.environment_sounds);
        check_map(&self.music_tracks);
        check_map(&self.radio_stations);
        check_map(&self.voice_lines);
        check_map(&self.decal_textures);
        check_map(&self.trail_markers);
        check_map(&self.custom_shaders);
        check_map(&self.shader_materials);
        
        state.is_complete = state.loaded_assets + state.failed_assets == state.total_assets;
        state
    }
    
    /// Hot reload assets during development
    #[cfg(debug_assertions)]
    pub fn hot_reload(&mut self, asset_server: &AssetServer) {
        asset_server.mark_all_assets_for_reload();
    }
    
    /// Get the current loading priority based on progress
    pub fn get_current_loading_priority(&self, loading_state: &AssetLoadingState) -> LoadPriority {
        let progress = loading_state.loaded_assets as f32 / loading_state.total_assets as f32;
        match progress {
            p if p < 0.25 => LoadPriority::Critical,
            p if p < 0.50 => LoadPriority::High,
            p if p < 0.75 => LoadPriority::Medium,
            _ => LoadPriority::Low,
        }
    }
}

/// System to monitor asset loading progress
pub fn check_asset_loading_progress(
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>,
    mut loading_state: ResMut<AssetLoadingState>,
) {
    let new_state = game_assets.check_loading_progress(&asset_server);
    
    // Update current loading priority if needed
    if new_state.current_priority != loading_state.current_priority {
        info!(
            "Asset loading priority changed from {:?} to {:?}",
            loading_state.current_priority,
            new_state.current_priority
        );
    }
    
    *loading_state = new_state;
} 