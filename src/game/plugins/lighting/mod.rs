/// Volumetric lighting module for rendering atmospheric effects like fog and clouds
/// 
/// This module provides a complete volumetric lighting system that includes:
/// - 3D texture management for volumetric data
/// - Render pipeline for volumetric effects
/// - Dynamic updates for animated effects
/// 
/// The system uses a 128x128x64 3D texture to store volumetric data, which is:
/// - Updated each frame with animated noise patterns
/// - Rendered using ray marching in the fragment shader
/// - Blended with the scene using physically-based light scattering
mod volumetric_pipeline;
mod volumetric_texture;

use bevy::prelude::*;
use volumetric_pipeline::VolumetricRenderPlugin;
use volumetric_texture::{VolumetricTexture, update_volume_texture};

/// Plugin that sets up the volumetric lighting system
/// 
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use sandk_offroad::plugins::lighting::VolumetricLightingPlugin;
/// 
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(VolumetricLightingPlugin)
///         .run();
/// }
/// ```
pub struct VolumetricLightingPlugin;

impl Plugin for VolumetricLightingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VolumetricTexture>()
            .add_systems(Update, update_volume_texture)
            .add_plugins(VolumetricRenderPlugin);
    }
}

// Re-export the settings struct for configuration
pub use volumetric_pipeline::VolumetricSettings; 