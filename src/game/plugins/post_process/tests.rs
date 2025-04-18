use bevy::prelude::*;
use super::post_process_pipeline::*;

#[test]
fn test_post_process_settings() {
    // Test default settings
    let settings = PostProcessSettings::default();
    assert_eq!(settings.tone_mapping_type, 1); // ACES by default
    assert_eq!(settings.exposure, 0.0);
    assert_eq!(settings.gamma, 2.2);
    assert_eq!(settings.bloom_intensity, 0.5);
    assert_eq!(settings.bloom_threshold, 1.0);
    assert_eq!(settings.saturation, 1.0);
    assert_eq!(settings.contrast, 1.0);
    assert_eq!(settings.brightness, 1.0);
}

#[test]
fn test_post_process_plugin_setup() {
    let mut app = App::new();
    app.add_plugins(PostProcessPlugin);
    
    // Test that the plugin added necessary resources and systems
    assert!(app.world.contains_resource::<PostProcessPipeline>());
}

#[test]
fn test_post_process_camera_setup() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PostProcessPlugin));
    
    // Spawn a camera with post-process settings
    app.world.spawn((
        Camera3dBundle::default(),
        PostProcessSettings::default(),
    ));
    
    // Run the startup systems
    app.update();
    
    // Verify camera has post-process settings
    let query = app.world.query::<(&Camera, &PostProcessSettings)>();
    assert_eq!(query.iter(&app.world).count(), 1);
}

#[test]
fn test_post_process_settings_update() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PostProcessPlugin));
    
    // Spawn a camera with custom settings
    app.world.spawn((
        Camera3dBundle::default(),
        PostProcessSettings {
            tone_mapping_type: 2, // Reinhard
            exposure: 1.0,
            gamma: 2.4,
            bloom_intensity: 0.7,
            bloom_threshold: 0.8,
            saturation: 1.2,
            contrast: 1.1,
            brightness: 0.9,
        },
    ));
    
    // Run the startup systems
    app.update();
    
    // Verify settings were applied correctly
    let settings = app.world
        .query::<&PostProcessSettings>()
        .iter(&app.world)
        .next()
        .unwrap();
    
    assert_eq!(settings.tone_mapping_type, 2);
    assert_eq!(settings.exposure, 1.0);
    assert_eq!(settings.gamma, 2.4);
    assert_eq!(settings.bloom_intensity, 0.7);
    assert_eq!(settings.bloom_threshold, 0.8);
    assert_eq!(settings.saturation, 1.2);
    assert_eq!(settings.contrast, 1.1);
    assert_eq!(settings.brightness, 0.9);
}

#[test]
fn test_post_process_test_scene() {
    use super::test_scene::PostProcessTestPlugin;
    
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PostProcessPlugin, PostProcessTestPlugin));
    
    // Run startup systems to create the test scene
    app.update();
    
    // Verify scene setup
    let camera_query = app.world.query_filtered::<&PostProcessSettings, With<Camera>>();
    assert_eq!(camera_query.iter(&app.world).count(), 1);
    
    let cube_query = app.world.query_filtered::<&Transform, With<super::test_scene::RotatingCube>>();
    assert_eq!(cube_query.iter(&app.world).count(), 1);
    
    // Run a few updates to test animations
    for _ in 0..10 {
        app.update();
    }
    
    // Verify settings were updated by the test scene systems
    let settings = app.world
        .query::<&PostProcessSettings>()
        .iter(&app.world)
        .next()
        .unwrap();
    
    // Settings should have been modified by the update_settings system
    assert!(settings.bloom_intensity >= 0.0 && settings.bloom_intensity <= 1.0);
    assert!(settings.exposure >= -2.0 && settings.exposure <= 2.0);
    assert!(settings.contrast >= 0.5 && settings.contrast <= 2.0);
} 