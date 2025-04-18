use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::texture::BevyDefault;
use super::particle_material::{ParticleMaterial, ParticleProperties, AtlasConfig, ParticleBlendMode};

fn setup_test_env() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::render::RenderPlugin::default())
        .add_plugins(bevy::asset::AssetPlugin::default())
        .add_plugins(bevy::core_pipeline::CorePipelinePlugin::default());
    app
}

#[test]
fn test_particle_material_creation() {
    let mut app = setup_test_env();
    
    // Create a basic particle material
    let material = ParticleMaterial {
        atlas_config: AtlasConfig {
            grid_size: Vec2::new(1.0, 1.0),
            frame_time: 0.1,
            total_frames: 1,
            current_time: 0.0,
        },
        properties: ParticleProperties {
            emission_strength: 1.0,
            soft_particles: 0.0,
            distortion_amount: 0.0,
            normal_strength: 0.0,
        },
        blend_mode: ParticleBlendMode::AlphaBlend,
        diffuse_texture: None,
        normal_texture: None,
    };
    
    // Add material to assets
    let material_handle = app.world.resource_mut::<Assets<ParticleMaterial>>()
        .add(material);
    
    // Verify material exists in assets
    let materials = app.world.resource::<Assets<ParticleMaterial>>();
    assert!(materials.get(&material_handle).is_some());
}

#[test]
fn test_atlas_animation() {
    let mut app = setup_test_env();
    
    // Create material with atlas animation
    let mut material = ParticleMaterial {
        atlas_config: AtlasConfig {
            grid_size: Vec2::new(2.0, 2.0),
            frame_time: 0.1,
            total_frames: 4,
            current_time: 0.0,
        },
        properties: ParticleProperties::default(),
        blend_mode: ParticleBlendMode::AlphaBlend,
        diffuse_texture: None,
        normal_texture: None,
    };
    
    // Test frame progression
    let initial_frame = material.atlas_config.current_time / material.atlas_config.frame_time;
    material.atlas_config.current_time += 0.15; // Advance time by more than one frame
    let next_frame = material.atlas_config.current_time / material.atlas_config.frame_time;
    
    assert!(next_frame > initial_frame);
    assert!(next_frame < material.atlas_config.total_frames as f32);
}

#[test]
fn test_blend_modes() {
    let mut app = setup_test_env();
    
    // Test each blend mode
    let blend_modes = vec![
        ParticleBlendMode::Additive,
        ParticleBlendMode::AlphaBlend,
        ParticleBlendMode::Premultiplied,
        ParticleBlendMode::Multiply,
    ];
    
    for blend_mode in blend_modes {
        let material = ParticleMaterial {
            atlas_config: AtlasConfig::default(),
            properties: ParticleProperties::default(),
            blend_mode,
            diffuse_texture: None,
            normal_texture: None,
        };
        
        let handle = app.world.resource_mut::<Assets<ParticleMaterial>>()
            .add(material);
            
        let materials = app.world.resource::<Assets<ParticleMaterial>>();
        let loaded_material = materials.get(&handle).unwrap();
        assert_eq!(loaded_material.blend_mode, blend_mode);
    }
}

#[test]
fn test_texture_assignment() {
    let mut app = setup_test_env();
    let mut textures = app.world.resource_mut::<Assets<Image>>();
    
    // Create test texture
    let test_texture = Image::new_fill(
        bevy::render::render_resource::Extent3d::default(),
        bevy::render::render_resource::TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::bevy_default(),
    );
    let texture_handle = textures.add(test_texture);
    
    // Create material with texture
    let material = ParticleMaterial {
        atlas_config: AtlasConfig::default(),
        properties: ParticleProperties::default(),
        blend_mode: ParticleBlendMode::AlphaBlend,
        diffuse_texture: Some(texture_handle.clone()),
        normal_texture: None,
    };
    
    let material_handle = app.world.resource_mut::<Assets<ParticleMaterial>>()
        .add(material);
        
    let materials = app.world.resource::<Assets<ParticleMaterial>>();
    let loaded_material = materials.get(&material_handle).unwrap();
    assert!(loaded_material.diffuse_texture.is_some());
    assert_eq!(loaded_material.diffuse_texture.as_ref().unwrap(), &texture_handle);
}

#[test]
fn test_particle_properties() {
    let mut app = setup_test_env();
    
    let properties = ParticleProperties {
        emission_strength: 2.0,
        soft_particles: 0.5,
        distortion_amount: 0.3,
        normal_strength: 1.0,
    };
    
    let material = ParticleMaterial {
        atlas_config: AtlasConfig::default(),
        properties,
        blend_mode: ParticleBlendMode::AlphaBlend,
        diffuse_texture: None,
        normal_texture: None,
    };
    
    let material_handle = app.world.resource_mut::<Assets<ParticleMaterial>>()
        .add(material);
        
    let materials = app.world.resource::<Assets<ParticleMaterial>>();
    let loaded_material = materials.get(&material_handle).unwrap();
    
    assert_eq!(loaded_material.properties.emission_strength, 2.0);
    assert_eq!(loaded_material.properties.soft_particles, 0.5);
    assert_eq!(loaded_material.properties.distortion_amount, 0.3);
    assert_eq!(loaded_material.properties.normal_strength, 1.0);
} 