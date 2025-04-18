use bevy::prelude::*;
use bevy::render::camera::Camera3d;
use crate::game::plugins::camera::*;

/// Helper function to setup a test app with camera plugin
fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(CameraPlugin)
        .init_resource::<crate::InputState>();
    app
}

#[test]
fn test_camera_setup() {
    let mut app = setup_test_app();
    app.update();

    // Verify camera entity was spawned
    let camera = app.world.query_filtered::<Entity, With<GameCamera>>().iter(&app.world).next();
    assert!(camera.is_some(), "Camera entity should be spawned");

    // Verify camera has correct components
    let camera_entity = camera.unwrap();
    assert!(app.world.get::<Camera3d>(camera_entity).is_some(), "Camera should have Camera3d component");
    assert!(app.world.get::<Transform>(camera_entity).is_some(), "Camera should have Transform component");
}

#[test]
fn test_camera_settings() {
    let app = setup_test_app();
    
    // Verify default settings
    let settings = app.world.resource::<CameraSettings>();
    assert_eq!(settings.follow_distance, 10.0);
    assert_eq!(settings.follow_height, 5.0);
    assert_eq!(settings.follow_smoothness, 0.1);
    assert_eq!(settings.rotation_sensitivity, 0.005);
    assert_eq!(settings.zoom_sensitivity, 0.5);
    assert_eq!(settings.min_zoom, 5.0);
    assert_eq!(settings.max_zoom, 20.0);
}

#[test]
fn test_camera_rotation() {
    let mut app = setup_test_app();
    
    // Setup test input
    let mut input = app.world.resource_mut::<crate::InputState>();
    input.camera_rotate = Vec2::new(1.0, 0.5);
    
    // Run update
    app.update();
    
    // Verify camera rotation
    let camera = app.world.query::<&GameCamera>().single();
    assert!(camera.orbit_angle.x > 0.0, "Camera should rotate horizontally");
    assert!(camera.orbit_angle.y > 0.0, "Camera should rotate vertically");
    
    // Test pitch clamping
    let mut camera = app.world.query::<&mut GameCamera>().single_mut();
    camera.orbit_angle.y = std::f32::consts::PI;
    app.update();
    assert!(camera.orbit_angle.y <= std::f32::consts::FRAC_PI_2, "Camera pitch should be clamped");
}

#[test]
fn test_camera_zoom() {
    let mut app = setup_test_app();
    
    // Setup test input
    let mut input = app.world.resource_mut::<crate::InputState>();
    input.camera_zoom = 1.0;
    
    // Run update
    app.update();
    
    // Verify zoom
    let camera = app.world.query::<&GameCamera>().single();
    assert!(camera.current_zoom > 10.0, "Camera should zoom out");
    
    // Test zoom clamping
    let mut input = app.world.resource_mut::<crate::InputState>();
    input.camera_zoom = 100.0;
    app.update();
    
    let camera = app.world.query::<&GameCamera>().single();
    assert_eq!(camera.current_zoom, 20.0, "Camera zoom should be clamped to max");
}

#[test]
fn test_camera_follow() {
    let mut app = setup_test_app();
    
    // Spawn target entity
    let target = app.world.spawn(Transform::from_xyz(5.0, 0.0, 5.0)).id();
    
    // Set camera target
    let mut camera = app.world.query::<&mut GameCamera>().single_mut();
    camera.target = Some(target);
    
    // Run multiple updates to allow for smooth following
    for _ in 0..10 {
        app.update();
    }
    
    // Verify camera position
    let camera_transform = app.world.query::<&Transform>().iter(&app.world).next().unwrap();
    assert!(camera_transform.translation.distance(Vec3::new(5.0, 0.0, 5.0)) > 5.0, 
        "Camera should maintain distance from target");
    
    // Test target movement
    let mut target_transform = app.world.get_mut::<Transform>(target).unwrap();
    target_transform.translation = Vec3::new(10.0, 0.0, 10.0);
    
    // Run updates to allow camera to follow
    for _ in 0..10 {
        app.update();
    }
    
    let camera_transform = app.world.query::<&Transform>().iter(&app.world).next().unwrap();
    assert!(camera_transform.translation.distance(Vec3::new(10.0, 0.0, 10.0)) > 5.0,
        "Camera should follow moving target");
} 