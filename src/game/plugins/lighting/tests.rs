use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use super::{
    volumetric_pipeline::{VolumetricSettings, VolumetricPipeline, VolumetricSettingsBuffer},
    volumetric_texture::{VolumetricTexture, VOLUME_SIZE, update_volume_texture},
    VolumetricLightingPlugin,
    examples::volumetric_demo::VolumetricDemoPlugin,
};

#[test]
fn test_volumetric_settings() {
    // Test default settings
    let default_settings = VolumetricSettings::default();
    assert!(default_settings.density > 0.0 && default_settings.density <= 1.0);
    assert!(default_settings.scattering > 0.0 && default_settings.scattering <= 1.0);
    assert!(default_settings.absorption >= 0.0 && default_settings.absorption <= 1.0);
    assert!(default_settings.max_distance > 0.0);

    // Test custom settings with clamping
    let custom_settings = VolumetricSettings::new(1.5, -0.5, 2.0, -10.0);
    assert_eq!(custom_settings.density, 1.0); // Should clamp to 1.0
    assert_eq!(custom_settings.scattering, 0.0); // Should clamp to 0.0
    assert_eq!(custom_settings.absorption, 1.0); // Should clamp to 1.0
    assert_eq!(custom_settings.max_distance, 0.0); // Should clamp to 0.0

    // Test preset configurations
    let fog = VolumetricSettings::dense_fog();
    assert!(fog.density > 0.5); // Should be dense
    assert!(fog.max_distance < 50.0); // Should be short range

    let haze = VolumetricSettings::light_haze();
    assert!(haze.density < 0.2); // Should be light
    assert!(haze.max_distance > 50.0); // Should be long range

    let clouds = VolumetricSettings::clouds();
    assert!(clouds.scattering > 0.7); // Should have high scattering
    
    // Test interpolation between settings
    let start = VolumetricSettings::dense_fog();
    let end = VolumetricSettings::light_haze();
    let t = 0.5;
    let interpolated = VolumetricSettings::new(
        start.density * (1.0 - t) + end.density * t,
        start.scattering * (1.0 - t) + end.scattering * t,
        start.absorption * (1.0 - t) + end.absorption * t,
        start.max_distance * (1.0 - t) + end.max_distance * t,
    );
    assert!(interpolated.density > end.density && interpolated.density < start.density);
}

#[test]
fn test_volumetric_texture_creation() {
    // Create a test app
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<VolumetricTexture>();

    // Get the created texture
    let texture = app.world.resource::<VolumetricTexture>();
    let images = app.world.resource::<Assets<Image>>();
    
    // Verify texture exists
    let image = images.get(&texture.texture).expect("Texture should exist");
    
    // Verify texture properties
    assert_eq!(image.texture_descriptor.size.width, VOLUME_SIZE.0);
    assert_eq!(image.texture_descriptor.size.height, VOLUME_SIZE.1);
    assert_eq!(image.texture_descriptor.size.depth_or_array_layers, VOLUME_SIZE.2);
    assert_eq!(image.texture_descriptor.format, TextureFormat::Rgba8Unorm);
    assert!(image.texture_descriptor.usage.contains(TextureUsages::TEXTURE_BINDING));
    assert!(image.texture_descriptor.usage.contains(TextureUsages::COPY_DST));
}

#[test]
fn test_volumetric_texture_update() {
    // Create a test app
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<VolumetricTexture>();
    
    // Add the update system
    app.add_systems(Update, update_volume_texture);

    // Run the app for a few frames
    for _ in 0..3 {
        app.update();
    }

    // Get the updated texture
    let texture = app.world.resource::<VolumetricTexture>();
    let images = app.world.resource::<Assets<Image>>();
    let image = images.get(&texture.texture).unwrap();

    // Verify texture data is being updated
    let data = image.data.as_slice();
    
    // Check that we have non-zero values (indicating noise generation is working)
    let has_non_zero = data.iter().any(|&x| x != 0);
    assert!(has_non_zero, "Texture should contain non-zero values after update");
    
    // Check that we have varying values (indicating proper noise distribution)
    let has_variation = data.windows(2).any(|w| w[0] != w[1]);
    assert!(has_variation, "Texture should contain varying values");
}

#[test]
fn test_volumetric_plugin_setup() {
    // Create a test app
    let mut app = App::new();
    
    // Add required plugins
    app.add_plugins((
        MinimalPlugins,
        VolumetricLightingPlugin,
    ));

    // Verify resources are initialized
    assert!(app.world.contains_resource::<VolumetricTexture>());
    assert!(app.world.contains_resource::<VolumetricSettings>());

    // Verify systems are added
    let system_exists = app
        .world
        .schedule
        .iter_schedules()
        .any(|schedule| {
            schedule.iter_systems().any(|system| {
                system.name().contains("update_volume_texture")
            })
        });
    assert!(system_exists, "update_volume_texture system should be registered");
}

#[test]
fn test_volumetric_render_setup() {
    // Create a test app with rendering support
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        VolumetricLightingPlugin,
    ));
    
    // Get the render app
    let render_app = app.sub_app_mut(RenderApp);
    
    // Verify render resources are initialized
    assert!(render_app.world.contains_resource::<VolumetricPipeline>());
    assert!(render_app.world.contains_resource::<VolumetricSettingsBuffer>());
    
    // Verify the pipeline is created
    let pipeline = render_app.world.resource::<VolumetricPipeline>();
    assert!(pipeline.bind_group_layout.as_raw() != 0, "Bind group layout should be valid");
    
    // Verify settings buffer is created
    let settings_buffer = render_app.world.resource::<VolumetricSettingsBuffer>();
    assert!(settings_buffer.buffer.as_raw() != 0, "Settings buffer should be valid");
}

#[test]
fn test_volumetric_camera_integration() {
    // Create a test app
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        VolumetricLightingPlugin,
    ));
    
    // Spawn a camera
    let camera_entity = app.world.spawn((
        Camera3dBundle::default(),
    )).id();
    
    // Update once to let systems run
    app.update();
    
    // Verify camera has necessary components
    let camera_exists = app.world.entity(camera_entity).contains::<Camera3d>();
    assert!(camera_exists, "Camera should exist and have Camera3d component");
}

#[test]
fn test_volumetric_demo_setup() {
    // Create a test app
    let mut app = App::new();
    
    // Add required plugins
    app.add_plugins((
        MinimalPlugins,
        VolumetricDemoPlugin,
    ));
    
    // Run startup systems
    app.update();
    
    // Verify camera setup
    let camera_query = app.world.query_filtered::<&Transform, With<Camera3d>>();
    assert_eq!(camera_query.iter(&app.world).count(), 1, "Camera should be spawned");
    
    // Verify light setup
    let light_query = app.world.query_filtered::<&DirectionalLight, With<DirectionalLight>>();
    assert_eq!(light_query.iter(&app.world).count(), 1, "Directional light should be spawned");
    
    // Verify scene objects
    let mesh_query = app.world.query_filtered::<&Handle<Mesh>, With<Transform>>();
    let object_count = mesh_query.iter(&app.world).count();
    assert_eq!(object_count, 10, "Scene should have 10 objects (1 plane + 1 cube + 8 columns)");
    
    // Verify volumetric settings
    assert!(app.world.contains_resource::<VolumetricSettings>(), "Volumetric settings should be initialized");
    
    // Run a few update frames to test weather cycle
    for _ in 0..10 {
        app.update();
    }
    
    // Verify camera movement
    let camera_transform = app.world
        .query_filtered::<&Transform, With<Camera3d>>()
        .single(&app.world);
    
    assert!(camera_transform.translation.length() > 0.0, "Camera should have moved");
    assert!(camera_transform.translation.y > 0.0, "Camera should be above ground");
}

#[test]
fn test_weather_cycle_transitions() {
    use std::time::Duration;
    
    // Create a test app
    let mut app = App::new();
    app.add_plugins(VolumetricDemoPlugin)
        .init_resource::<Time>();
    
    // Get initial settings
    let initial_settings = app.world
        .resource::<VolumetricSettings>()
        .clone();
    
    // Advance time to middle of first transition
    let mut time = app.world.resource_mut::<Time>();
    time.update();
    time.advance_by(Duration::from_secs_f32(30.0));  // Half of first cycle
    
    // Update systems
    app.update();
    
    // Get updated settings
    let mid_settings = app.world
        .resource::<VolumetricSettings>()
        .clone();
    
    // Verify settings have changed
    assert!(
        (mid_settings.density - initial_settings.density).abs() > 0.01,
        "Settings should change during weather cycle"
    );
    
    // Advance to next cycle
    time.advance_by(Duration::from_secs_f32(30.0));
    app.update();
    
    // Get final settings
    let final_settings = app.world
        .resource::<VolumetricSettings>()
        .clone();
    
    // Verify we've transitioned to a different state
    assert!(
        (final_settings.density - mid_settings.density).abs() > 0.01,
        "Settings should continue changing through cycles"
    );
}

#[test]
fn test_volumetric_texture_noise_distribution() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<VolumetricTexture>();
    app.add_systems(Update, update_volume_texture);

    // Run multiple updates to ensure proper noise evolution
    for _ in 0..10 {
        app.update();
    }

    let texture = app.world.resource::<VolumetricTexture>();
    let images = app.world.resource::<Assets<Image>>();
    let image = images.get(&texture.texture).unwrap();
    let data = image.data.as_slice();

    // Calculate basic statistics
    let sum: f32 = data.iter().map(|&x| x as f32).sum::<f32>() / 255.0;
    let mean = sum / (data.len() as f32);
    
    // Verify mean is roughly in the middle range (0.3 to 0.7)
    assert!(mean > 0.3 && mean < 0.7, "Noise mean should be roughly centered");

    // Check value distribution
    let low_values = data.iter().filter(|&&x| x < 85).count();  // < 1/3
    let mid_values = data.iter().filter(|&&x| x >= 85 && x <= 170).count();  // 1/3 - 2/3
    let high_values = data.iter().filter(|&&x| x > 170).count();  // > 2/3

    let total = data.len();
    
    // Verify roughly even distribution across ranges
    assert!(low_values as f32 / total as f32 > 0.2, "Should have sufficient low values");
    assert!(mid_values as f32 / total as f32 > 0.2, "Should have sufficient mid values");
    assert!(high_values as f32 / total as f32 > 0.2, "Should have sufficient high values");
}

#[test]
fn test_volumetric_settings_interpolation_edge_cases() {
    // Test extreme value interpolation
    let min_settings = VolumetricSettings::new(0.0, 0.0, 0.0, 0.0);
    let max_settings = VolumetricSettings::new(1.0, 1.0, 1.0, 100.0);

    // Test various interpolation points
    let interpolation_points = [0.0, 0.25, 0.5, 0.75, 1.0];
    
    for t in interpolation_points.iter() {
        let interpolated = VolumetricSettings::new(
            min_settings.density * (1.0 - t) + max_settings.density * t,
            min_settings.scattering * (1.0 - t) + max_settings.scattering * t,
            min_settings.absorption * (1.0 - t) + max_settings.absorption * t,
            min_settings.max_distance * (1.0 - t) + max_settings.max_distance * t,
        );

        // Verify interpolated values
        assert_eq!(
            interpolated.density,
            t.clamp(0.0, 1.0),
            "Density interpolation failed at t={}", t
        );
        assert_eq!(
            interpolated.scattering,
            t.clamp(0.0, 1.0),
            "Scattering interpolation failed at t={}", t
        );
        assert_eq!(
            interpolated.absorption,
            t.clamp(0.0, 1.0),
            "Absorption interpolation failed at t={}", t
        );
        assert_eq!(
            interpolated.max_distance,
            100.0 * t.clamp(0.0, 1.0),
            "Max distance interpolation failed at t={}", t
        );
    }
}

#[test]
fn test_volumetric_demo_scene_stress() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, VolumetricDemoPlugin));

    // Run many updates to stress test the weather cycle
    for _ in 0..1000 {
        app.update();
    }

    // Verify resources are still valid
    assert!(app.world.contains_resource::<VolumetricSettings>());
    
    // Get final settings and verify they're in valid ranges
    let settings = app.world.resource::<VolumetricSettings>();
    assert!(settings.density >= 0.0 && settings.density <= 1.0);
    assert!(settings.scattering >= 0.0 && settings.scattering <= 1.0);
    assert!(settings.absorption >= 0.0 && settings.absorption <= 1.0);
    assert!(settings.max_distance >= 0.0);

    // Verify camera is still functioning
    let camera = app.world
        .query_filtered::<&Transform, With<Camera3d>>()
        .single(&app.world);
    
    assert!(camera.translation.length() > 0.0);
    assert!(camera.translation.y > 0.0);
}

#[test]
fn test_volumetric_render_pipeline_validation() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, VolumetricLightingPlugin));
    
    let render_app = app.sub_app_mut(RenderApp);
    let pipeline = render_app.world.resource::<VolumetricPipeline>();
    
    // Verify bind group layout entries
    let layout = &pipeline.bind_group_layout;
    assert!(layout.as_raw() != 0, "Bind group layout should be valid");
    
    // Verify settings buffer
    let settings_buffer = render_app.world.resource::<VolumetricSettingsBuffer>();
    assert!(settings_buffer.buffer.as_raw() != 0, "Settings buffer should be valid");
    
    // Run multiple updates to verify pipeline stability
    for _ in 0..100 {
        app.update();
    }
    
    // Verify resources are still valid after updates
    assert!(render_app.world.contains_resource::<VolumetricPipeline>());
    assert!(render_app.world.contains_resource::<VolumetricSettingsBuffer>());
} 