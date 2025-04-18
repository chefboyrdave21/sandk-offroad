use bevy::prelude::*;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::reflect::TypeUuid;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::path::Path;

/// Custom asset type for vehicle configurations
#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "f9c9dc52-fc1a-45ea-a7ba-3e4ea34e0298"]
pub struct VehicleConfig {
    pub name: String,
    pub mass: f32,
    pub suspension_config: SuspensionConfig,
    pub engine_config: EngineConfig,
    pub wheel_config: WheelConfig,
}

#[derive(Debug, Deserialize)]
pub struct SuspensionConfig {
    pub spring_stiffness: f32,
    pub damping: f32,
    pub travel: f32,
}

#[derive(Debug, Deserialize)]
pub struct EngineConfig {
    pub max_power: f32,
    pub max_torque: f32,
    pub redline: f32,
}

#[derive(Debug, Deserialize)]
pub struct WheelConfig {
    pub radius: f32,
    pub width: f32,
    pub mass: f32,
}

/// Custom asset loader for vehicle configurations
#[derive(Default)]
pub struct VehicleConfigLoader;

impl AssetLoader for VehicleConfigLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let config: VehicleConfig = serde_json::from_slice(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(config));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["vehicle.json"]
    }
}

/// Resource to track loaded assets and their status
#[derive(Resource, Default)]
pub struct AssetLoadingState {
    pub total_assets: usize,
    pub loaded_assets: usize,
    pub failed_assets: Vec<String>,
}

/// Plugin to handle asset management
pub struct AssetManagerPlugin;

impl Plugin for AssetManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetLoadingState>()
            .add_asset::<VehicleConfig>()
            .init_asset_loader::<VehicleConfigLoader>()
            .add_systems(Update, check_asset_loading);
    }
}

/// System to monitor asset loading progress
fn check_asset_loading(
    asset_server: Res<AssetServer>,
    mut loading_state: ResMut<AssetLoadingState>,
    vehicle_configs: Res<Assets<VehicleConfig>>,
) {
    // Update loading progress
    let progress = asset_server.get_group_load_state(loading_state.total_assets.into());
    
    match progress {
        bevy::asset::LoadState::Loading => {
            loading_state.loaded_assets = vehicle_configs.len();
        }
        bevy::asset::LoadState::Loaded => {
            loading_state.loaded_assets = loading_state.total_assets;
        }
        bevy::asset::LoadState::Failed => {
            // Add failed asset paths to the list
            if let Some(failed_handle) = asset_server.get_load_state(0.into()) {
                loading_state.failed_assets.push(format!("{:?}", failed_handle));
            }
        }
        _ => {}
    }
}

/// Helper functions for asset loading
impl AssetLoadingState {
    /// Load all vehicle configurations from a directory
    pub fn load_vehicle_configs(
        &mut self,
        asset_server: &AssetServer,
        directory: &Path,
    ) -> Vec<Handle<VehicleConfig>> {
        let mut handles = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(directory) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "vehicle.json" {
                        let handle = asset_server.load(entry.path().to_str().unwrap());
                        handles.push(handle);
                        self.total_assets += 1;
                    }
                }
            }
        }
        
        handles
    }

    /// Check if all assets are loaded
    pub fn is_loading_complete(&self) -> bool {
        self.loaded_assets == self.total_assets
    }

    /// Get loading progress as a percentage
    pub fn get_loading_progress(&self) -> f32 {
        if self.total_assets == 0 {
            return 1.0;
        }
        self.loaded_assets as f32 / self.total_assets as f32
    }
}

// Add this to your app setup:
// app.add_plugin(AssetManagerPlugin); 