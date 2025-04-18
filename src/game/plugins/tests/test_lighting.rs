use bevy::prelude::*;
use bevy::app::App;
use bevy::math::Vec3;
use bevy::time::Time;
use std::f32::consts::PI;

use crate::game::plugins::lighting::{
    LightingPlugin,
    LightFlicker,
    LightTemperature,
    spawn_point_light,
    spawn_spot_light,
    temperature_to_rgb,
};

#[test]
fn test_lighting_plugin_setup() {
    let mut app = App::new();
    app.add_plugins(LightingPlugin);
    
    // Verify that the plugin added necessary resources
    assert!(app.world.get_resource::<AmbientLight>().is_some());
    
    // Check if directional light was spawned
    let mut query = app.world.query::<(&DirectionalLight, &LightTemperature)>();
    let count = query.iter(&app.world).count();
    assert_eq!(count, 1);
}

#[test]
fn test_point_light_flicker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Create a point light with flicker
    let light_entity = {
        let mut commands = app.world.spawn_empty();
        let entity = commands.id();
        commands.insert(PointLight {
            intensity: 1000.0,
            ..default()
        });
        commands.insert(LightFlicker {
            base_intensity: 1000.0,
            flicker_speed: 5.0,
            flicker_intensity: 0.2,
            time: 0.0,
        });
        entity
    };
    
    // Advance time and check intensity changes
    let mut time = app.world.resource_mut::<Time>();
    time.update();
    time.advance_by(std::time::Duration::from_secs_f32(0.5));
    
    let light = app.world.get::<PointLight>(light_entity).unwrap();
    assert!(light.intensity != 1000.0); // Intensity should have changed due to flicker
}

#[test]
fn test_spot_light_creation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let position = Vec3::new(0.0, 10.0, 0.0);
    let target = Vec3::ZERO;
    let color = Color::WHITE;
    let intensity = 1000.0;
    let range = 20.0;
    let inner_angle = 0.5;
    let outer_angle = 0.8;
    
    let mut commands = app.world.spawn_empty().into_commands();
    let entity = spawn_spot_light(
        &mut commands,
        position,
        target,
        color,
        intensity,
        range,
        inner_angle,
        outer_angle,
    );
    
    let spot_light = app.world.get::<SpotLight>(entity).unwrap();
    assert_eq!(spot_light.intensity, intensity);
    assert_eq!(spot_light.range, range);
    assert_eq!(spot_light.inner_angle, inner_angle);
    assert_eq!(spot_light.outer_angle, outer_angle);
}

#[test]
fn test_directional_light_movement() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn directional light
    let light_entity = {
        let mut commands = app.world.spawn_empty();
        let entity = commands.id();
        commands.insert(DirectionalLight::default());
        commands.insert(Transform::from_xyz(50.0, 50.0, 50.0));
        entity
    };
    
    // Advance time
    let mut time = app.world.resource_mut::<Time>();
    time.update();
    time.advance_by(std::time::Duration::from_secs_f32(PI));
    
    // Check if position has changed
    let transform = app.world.get::<Transform>(light_entity).unwrap();
    assert!(transform.translation.x != 50.0 || transform.translation.z != 50.0);
}

#[test]
fn test_color_temperature() {
    // Test daylight temperature (6500K)
    let daylight = temperature_to_rgb(6500.0);
    assert!(daylight.r() > 0.9); // Should be close to white
    assert!(daylight.g() > 0.9);
    assert!(daylight.b() > 0.9);
    
    // Test warm light (2700K)
    let warm = temperature_to_rgb(2700.0);
    assert!(warm.r() > warm.b()); // Should be more red than blue
    
    // Test cool light (10000K)
    let cool = temperature_to_rgb(10000.0);
    assert!(cool.b() > cool.r()); // Should be more blue than red
}

#[test]
fn test_light_flicker_component() {
    let flicker = LightFlicker::default();
    assert_eq!(flicker.base_intensity, 1000.0);
    assert_eq!(flicker.flicker_speed, 5.0);
    assert_eq!(flicker.flicker_intensity, 0.2);
    assert_eq!(flicker.time, 0.0);
}

#[test]
fn test_light_temperature_component() {
    let temp = LightTemperature::default();
    assert_eq!(temp.kelvin, 6500.0); // Should default to daylight temperature
    
    let color = temperature_to_rgb(temp.kelvin);
    assert!(color.r() > 0.9 && color.g() > 0.9 && color.b() > 0.9); // Should be close to white
}

#[test]
fn test_day_night_cycle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn directional light
    let light_entity = {
        let mut commands = app.world.spawn_empty();
        let entity = commands.id();
        commands.insert(DirectionalLight {
            illuminance: 100000.0,
            ..default()
        });
        commands.insert(Transform::from_xyz(50.0, 50.0, 50.0));
        entity
    };
    
    // Test multiple times of day
    let test_times = [0.0, PI/2.0, PI, 3.0*PI/2.0]; // Dawn, noon, dusk, midnight
    let mut time = app.world.resource_mut::<Time>();
    
    for &test_time in test_times.iter() {
        time.update();
        time.advance_by(std::time::Duration::from_secs_f32(test_time * 10.0));
        
        let light = app.world.get::<DirectionalLight>(light_entity).unwrap();
        let transform = app.world.get::<Transform>(light_entity).unwrap();
        
        // Verify light position and intensity changes
        if test_time == PI/2.0 { // Noon
            assert!(transform.translation.y > 45.0);
            assert!(light.illuminance > 90000.0);
        } else if test_time == 3.0*PI/2.0 { // Midnight
            assert!(transform.translation.y < 45.0);
            assert!(light.illuminance < 50000.0);
        }
    }
}

#[test]
fn test_point_light_range() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let test_ranges = [5.0, 20.0, 50.0];
    
    for range in test_ranges {
        let entity = {
            let mut commands = app.world.spawn_empty().into_commands();
            spawn_point_light(
                &mut commands,
                Vec3::new(0.0, 10.0, 0.0),
                Color::WHITE,
                1000.0,
                range,
                None,
            )
        };
        
        let light = app.world.get::<PointLight>(entity).unwrap();
        assert_eq!(light.range, range);
        assert!(light.shadows_enabled); // Verify shadows are enabled
    }
}

#[test]
fn test_spot_light_height_adaptation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let test_heights = [5.0, 15.0, 30.0];
    
    for height in test_heights {
        let entity = {
            let mut commands = app.world.spawn_empty().into_commands();
            spawn_spot_light(
                &mut commands,
                Vec3::new(0.0, height, 0.0),
                Vec3::ZERO,
                Color::WHITE,
                1000.0,
                20.0,
                0.5,
                0.8,
            )
        };
        
        let light = app.world.get::<SpotLight>(entity).unwrap();
        let transform = app.world.get::<Transform>(entity).unwrap();
        
        // Verify that spot light adapts to height
        assert!(light.range >= 5.0 && light.range <= 50.0);
        assert!(light.outer_angle >= 0.5 && light.outer_angle <= 1.2);
        assert_eq!(transform.translation.y, height);
    }
}

#[test]
fn test_extreme_temperature_values() {
    // Test extreme temperatures
    let extreme_temps = [1000.0, 2000.0, 15000.0, 20000.0];
    
    for temp in extreme_temps {
        let color = temperature_to_rgb(temp);
        
        // Ensure color values are valid (between 0 and 1)
        assert!(color.r() >= 0.0 && color.r() <= 1.0);
        assert!(color.g() >= 0.0 && color.g() <= 1.0);
        assert!(color.b() >= 0.0 && color.b() <= 1.0);
    }
}

#[test]
fn test_flicker_intensity_bounds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let test_cases = [
        (1000.0, 10.0, 0.5), // High speed, high intensity
        (500.0, 1.0, 0.1),   // Low speed, low intensity
        (2000.0, 20.0, 0.8), // Extreme values
    ];
    
    for (base, speed, intensity) in test_cases {
        let light_entity = {
            let mut commands = app.world.spawn_empty();
            let entity = commands.id();
            commands.insert(PointLight {
                intensity: base,
                ..default()
            });
            commands.insert(LightFlicker {
                base_intensity: base,
                flicker_speed: speed,
                flicker_intensity: intensity,
                time: 0.0,
            });
            entity
        };
        
        // Run multiple time steps
        let mut time = app.world.resource_mut::<Time>();
        for _ in 0..10 {
            time.update();
            time.advance_by(std::time::Duration::from_secs_f32(0.1));
            
            let light = app.world.get::<PointLight>(light_entity).unwrap();
            
            // Verify intensity stays within expected bounds
            let max_intensity = base * (1.0 + intensity);
            let min_intensity = base * (1.0 - intensity);
            assert!(light.intensity <= max_intensity);
} 