use bevy::prelude::*;
use bevy::math::Vec3;
use super::super::{
    Trail,
    TrailPoints,
    TrailPoint,
    TrailPlugin,
    spawn_trail,
};

#[test]
fn test_trail_point_creation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TrailPlugin);

    // Create a test entity with a trail
    let entity = app.world.spawn(Transform::default()).id();
    let mut commands = app.world.spawn_empty().commands();
    spawn_trail(
        &mut commands,
        entity,
        5, // max_points
        1.0, // point_distance
        2.0, // fade_time
        0.5, // width
    );

    // Run systems once
    app.update();

    // Check initial state
    let trail_points = app.world.get::<TrailPoints>(entity).unwrap();
    assert_eq!(trail_points.0.len(), 1, "Should have initial point");
}

#[test]
fn test_trail_point_limit() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TrailPlugin);

    // Create a test entity with a trail
    let entity = app.world.spawn(Transform::default()).id();
    let mut commands = app.world.spawn_empty().commands();
    spawn_trail(
        &mut commands,
        entity,
        3, // max_points
        0.1, // point_distance
        2.0, // fade_time
        0.5, // width
    );

    // Move the entity in a straight line
    for i in 0..5 {
        app.world.entity_mut(entity)
            .get_mut::<Transform>()
            .unwrap()
            .translation = Vec3::new(i as f32 * 0.2, 0.0, 0.0);
        app.update();
    }

    // Check point limit
    let trail_points = app.world.get::<TrailPoints>(entity).unwrap();
    assert_eq!(trail_points.0.len(), 3, "Should respect max_points limit");
}

#[test]
fn test_trail_point_fade() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TrailPlugin);

    // Create a test entity with a trail
    let entity = app.world.spawn(Transform::default()).id();
    let mut commands = app.world.spawn_empty().commands();
    spawn_trail(
        &mut commands,
        entity,
        10, // max_points
        0.1, // point_distance
        0.5, // fade_time (short for testing)
        0.5, // width
    );

    // Create some points
    for i in 0..3 {
        app.world.entity_mut(entity)
            .get_mut::<Transform>()
            .unwrap()
            .translation = Vec3::new(i as f32 * 0.2, 0.0, 0.0);
        app.update();
    }

    // Advance time significantly
    app.world.resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(1.0));
    app.update();

    // Check that old points were removed
    let trail_points = app.world.get::<TrailPoints>(entity).unwrap();
    assert_eq!(trail_points.0.len(), 0, "Old points should be removed after fade time");
}

#[test]
fn test_trail_point_distance() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TrailPlugin);

    // Create a test entity with a trail
    let entity = app.world.spawn(Transform::default()).id();
    let mut commands = app.world.spawn_empty().commands();
    spawn_trail(
        &mut commands,
        entity,
        10, // max_points
        1.0, // point_distance
        2.0, // fade_time
        0.5, // width
    );

    // Move the entity a small distance (less than point_distance)
    app.world.entity_mut(entity)
        .get_mut::<Transform>()
        .unwrap()
        .translation = Vec3::new(0.5, 0.0, 0.0);
    app.update();

    // Check that no new point was created
    let trail_points = app.world.get::<TrailPoints>(entity).unwrap();
    assert_eq!(trail_points.0.len(), 1, "Should not create point for small movement");

    // Move the entity a larger distance
    app.world.entity_mut(entity)
        .get_mut::<Transform>()
        .unwrap()
        .translation = Vec3::new(2.0, 0.0, 0.0);
    app.update();

    // Check that a new point was created
    let trail_points = app.world.get::<TrailPoints>(entity).unwrap();
    assert_eq!(trail_points.0.len(), 2, "Should create point for large movement");
} 