use bevy::{
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, ShaderRef, ShaderType,
            BlendState, ColorWrites, Face, TextureFormat,
        },
        texture::Image,
    },
};

/// Configuration for texture atlas animation
#[derive(Clone, ShaderType)]
pub struct AtlasConfig {
    pub grid_size: Vec2,      // Number of frames in x and y
    pub frame_time: f32,      // Time per frame in seconds
    pub total_frames: u32,    // Total number of frames in the atlas
    pub current_time: f32,    // Current animation time
}

/// Level of Detail settings for particle optimization
#[derive(Clone, ShaderType)]
pub struct LodSettings {
    pub start_distance: f32,  // Distance at which LOD starts
    pub end_distance: f32,    // Distance at which particle is fully simplified/faded
    pub min_size: f32,       // Minimum particle size at max distance
    pub max_size: f32,       // Maximum particle size at min distance
    pub enabled: u32,        // Whether LOD is enabled (0 = disabled, 1 = enabled)
}

/// Custom material properties for particle rendering
#[derive(Clone, ShaderType)]
pub struct ParticleProperties {
    pub emission_strength: f32,
    pub soft_particles: f32,  // Depth fade factor for soft particles
    pub distortion_amount: f32,
    pub normal_strength: f32,
    pub color_tint: Vec4,    // Global color tint for all particles
    pub custom_params: Vec4,  // Custom parameters for shader effects
}

/// Material configuration for particle rendering
#[derive(Component, Clone, Default, AsBindGroup)]
pub struct ParticleMaterial {
    #[uniform(0)]
    pub atlas_config: AtlasConfig,
    
    #[uniform(1)]
    pub properties: ParticleProperties,
    
    #[uniform(2)]
    pub lod_settings: LodSettings,
    
    #[texture(3)]
    #[sampler(4)]
    pub diffuse_texture: Option<Handle<Image>>,
    
    #[texture(5)]
    #[sampler(6)]
    pub normal_texture: Option<Handle<Image>>,
    
    // Additional material properties
    pub blend_mode: ParticleBlendMode,
    pub double_sided: bool,
    pub alpha_cutoff: f32,
}

/// Available blend modes for particles
#[derive(Clone, Copy, PartialEq)]
pub enum ParticleBlendMode {
    Additive,
    AlphaBlend,
    Premultiplied,
    Multiply,
}

impl Default for ParticleBlendMode {
    fn default() -> Self {
        ParticleBlendMode::AlphaBlend
    }
}

impl Default for AtlasConfig {
    fn default() -> Self {
        Self {
            grid_size: Vec2::new(1.0, 1.0),
            frame_time: 0.1,
            total_frames: 1,
            current_time: 0.0,
        }
    }
}

impl Default for LodSettings {
    fn default() -> Self {
        Self {
            start_distance: 10.0,
            end_distance: 100.0,
            min_size: 0.1,
            max_size: 1.0,
            enabled: 1,
        }
    }
}

impl Default for ParticleProperties {
    fn default() -> Self {
        Self {
            emission_strength: 1.0,
            soft_particles: 0.5,
            distortion_amount: 0.0,
            normal_strength: 1.0,
            color_tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
            custom_params: Vec4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl Material for ParticleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/particle.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/particle.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        match self.blend_mode {
            ParticleBlendMode::Additive => AlphaMode::Add,
            ParticleBlendMode::AlphaBlend => AlphaMode::Blend,
            ParticleBlendMode::Premultiplied => AlphaMode::Premultiplied,
            ParticleBlendMode::Multiply => AlphaMode::Multiply,
        }
    }

    fn cull_mode(&self) -> Option<Face> {
        if self.double_sided {
            None
        } else {
            Some(Face::Back)
        }
    }
}

impl ParticleMaterial {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_texture(mut self, texture: Handle<Image>) -> Self {
        self.diffuse_texture = Some(texture);
        self
    }

    pub fn with_normal_map(mut self, texture: Handle<Image>) -> Self {
        self.normal_texture = Some(texture);
        self
    }

    pub fn with_atlas_config(mut self, grid_size: Vec2, frame_time: f32, total_frames: u32) -> Self {
        self.atlas_config = AtlasConfig {
            grid_size,
            frame_time,
            total_frames,
            current_time: 0.0,
        };
        self
    }

    pub fn with_blend_mode(mut self, mode: ParticleBlendMode) -> Self {
        self.blend_mode = mode;
        self
    }

    pub fn with_properties(mut self, properties: ParticleProperties) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_lod_settings(mut self, settings: LodSettings) -> Self {
        self.lod_settings = settings;
        self
    }

    pub fn with_color_tint(mut self, tint: Vec4) -> Self {
        self.properties.color_tint = tint;
        self
    }

    pub fn with_custom_params(mut self, params: Vec4) -> Self {
        self.properties.custom_params = params;
        self
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update atlas animation
        if self.atlas_config.total_frames > 1 {
            self.atlas_config.current_time += delta_time;
            if self.atlas_config.current_time >= self.atlas_config.frame_time * self.atlas_config.total_frames as f32 {
                self.atlas_config.current_time = 0.0;
            }
        }
    }

    // Preset configurations for common effects
    pub fn preset_fire() -> Self {
        Self::new()
            .with_blend_mode(ParticleBlendMode::Additive)
            .with_color_tint(Vec4::new(1.0, 0.5, 0.1, 1.0))
            .with_properties(ParticleProperties {
                emission_strength: 2.0,
                soft_particles: 0.3,
                distortion_amount: 0.2,
                normal_strength: 0.0,
                color_tint: Vec4::new(1.0, 0.5, 0.1, 1.0),
                custom_params: Vec4::new(1.0, 0.5, 0.2, 0.0), // size_scale, heat_distortion, flicker_intensity
            })
    }

    pub fn preset_smoke() -> Self {
        Self::new()
            .with_blend_mode(ParticleBlendMode::AlphaBlend)
            .with_color_tint(Vec4::new(0.5, 0.5, 0.5, 0.7))
            .with_properties(ParticleProperties {
                emission_strength: 0.5,
                soft_particles: 0.8,
                distortion_amount: 0.1,
                normal_strength: 0.0,
                color_tint: Vec4::new(0.5, 0.5, 0.5, 0.7),
                custom_params: Vec4::new(1.2, 0.3, 0.0, 0.0), // size_scale, turbulence
            })
    }

    pub fn preset_magic() -> Self {
        Self::new()
            .with_blend_mode(ParticleBlendMode::Additive)
            .with_color_tint(Vec4::new(0.2, 0.4, 1.0, 1.0))
            .with_properties(ParticleProperties {
                emission_strength: 3.0,
                soft_particles: 0.5,
                distortion_amount: 0.1,
                normal_strength: 0.0,
                color_tint: Vec4::new(0.2, 0.4, 1.0, 1.0),
                custom_params: Vec4::new(0.8, 1.0, 0.5, 0.0), // size_scale, sparkle_intensity, pulse_frequency
            })
    }
}

// System to update particle materials
pub fn update_particle_materials(
    time: Res<Time>,
    mut materials: Query<&mut ParticleMaterial>,
) {
    for mut material in materials.iter_mut() {
        material.update(time.delta_seconds());
    }
}

// Plugin to register the particle material and systems
pub struct ParticleMaterialPlugin;

impl Plugin for ParticleMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_particle_materials);
    }
} 