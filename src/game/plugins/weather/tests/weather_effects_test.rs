use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use crate::game::plugins::weather::{
    WeatherPlugin,
    WeatherState,
    WeatherEffects,
    WeatherEffectType,
    TimeOfDay,
};

#[test]
fn test_weather_effects_initialization() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    app.update();
    
    assert!(app.world.get_resource::<WeatherEffects>().is_some());
    assert!(app.world.get_resource::<WeatherState>().is_some());
}

#[test]
fn test_weather_effect_spawning() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    let mut weather_state = WeatherState::default();
    weather_state.precipitation = 0.8;
    weather_state.weather = Weather::Rain;
    app.insert_resource(weather_state);
    
    app.update();
    
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::HeavyRain));
}

#[test]
fn test_ground_effect_persistence() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    let mut weather_state = WeatherState::default();
    weather_state.precipitation = 0.8;
    weather_state.weather = Weather::Rain;
    app.insert_resource(weather_state);
    
    // Run for a few frames to allow ground effects to spawn
    for _ in 0..10 {
        app.update();
    }
    
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    assert!(effects.ground_effects.iter().any(|(t, _)| *t == WeatherEffectType::Puddles));
    
    // Change weather and verify ground effects persist
    let mut weather_state = app.world.get_resource_mut::<WeatherState>().unwrap();
    weather_state.weather = Weather::Clear;
    weather_state.precipitation = 0.0;
    
    // Run for a short time (less than persistence duration)
    for _ in 0..5 {
        app.update();
    }
    
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    assert!(effects.ground_effects.iter().any(|(t, _)| *t == WeatherEffectType::Puddles));
}

#[test]
fn test_time_of_day_influence() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    let mut weather_state = WeatherState::default();
    weather_state.fog_density = 0.5;
    weather_state.weather = Weather::Fog;
    app.insert_resource(weather_state);
    
    // Test effect parameters at different times
    for time in [TimeOfDay::Dawn, TimeOfDay::Noon, TimeOfDay::Night].iter() {
        let mut effects = app.world.get_resource_mut::<WeatherEffects>().unwrap();
        effects.update(&weather_state, *time, &Audio::default(), 0.016);
        
        match time {
            TimeOfDay::Dawn => {
                // Dawn should have enhanced visibility
                if let Some((_, entity)) = effects.active_effects.iter().find(|(t, _)| *t == WeatherEffectType::Fog) {
                    let material = entity.get_component::<Handle<ParticleMaterial>>().unwrap();
                    assert!(material.color.a() > 0.3); // Higher visibility at dawn
                }
            }
            TimeOfDay::Night => {
                // Night should have reduced visibility
                if let Some((_, entity)) = effects.active_effects.iter().find(|(t, _)| *t == WeatherEffectType::Fog) {
                    let material = entity.get_component::<Handle<ParticleMaterial>>().unwrap();
                    assert!(material.color.a() < 0.3); // Lower visibility at night
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test_wind_influence() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    let mut weather_state = WeatherState::default();
    weather_state.wind_speed = 15.0;
    weather_state.wind_direction = Vec2::new(1.0, 0.0);
    weather_state.weather = Weather::Clear;
    app.insert_resource(weather_state);
    
    app.update();
    
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::Sandstorm));
}

#[test]
fn test_special_weather_combinations() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    let mut weather_state = WeatherState::default();
    weather_state.temperature = -5.0;
    weather_state.precipitation = 0.6;
    weather_state.weather = Weather::Rain;
    app.insert_resource(weather_state);
    
    app.update();
    
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::FreezingRain));
}

#[test]
fn test_debug_visualization() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    // Enable debug visualization
    app.insert_resource(DebugState {
        show_weather_effects: true,
        show_ground_effects: true,
    });
    
    let mut weather_state = WeatherState::default();
    weather_state.precipitation = 0.8;
    weather_state.weather = Weather::Rain;
    app.insert_resource(weather_state);
    
    // Run for a few frames to allow effects to spawn
    for _ in 0..10 {
        app.update();
    }
    
    // Verify that debug gizmos are being drawn
    // Note: We can't directly test the visual output, but we can verify the system runs
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    assert!(!effects.active_effects.is_empty());
    assert!(!effects.ground_effects.is_empty());
}

#[test]
fn test_weather_transition_effects() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    let mut weather_state = WeatherState::default();
    weather_state.precipitation = 0.3;
    weather_state.weather = Weather::Rain;
    app.insert_resource(weather_state);
    
    // Initial update
    app.update();
    
    // Transition to snow
    {
        let mut weather = app.world.get_resource_mut::<WeatherState>().unwrap();
        weather.temperature = -2.0;
        weather.weather = Weather::Snow;
    }
    
    // Run updates during transition
    for _ in 0..30 {
        app.update();
    }
    
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::Snow));
    assert!(!effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::Rain));
}

#[test]
fn test_sound_effect_loading() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    app.update();
    
    let sound_assets = app.world.get_resource::<WeatherSoundAssets>().unwrap();
    assert!(sound_assets.light_rain.id() != Handle::<AudioSource>::default().id());
    assert!(sound_assets.heavy_rain.id() != Handle::<AudioSource>::default().id());
    assert!(sound_assets.storm.id() != Handle::<AudioSource>::default().id());
}

#[test]
fn test_multiple_ground_effects() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    let mut weather_state = WeatherState::default();
    weather_state.precipitation = 0.9;
    weather_state.wind_speed = 12.0;
    weather_state.weather = Weather::Storm;
    app.insert_resource(weather_state);
    
    // Run for enough frames to accumulate effects
    for _ in 0..20 {
        app.update();
    }
    
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    // Should have both puddles and wind-blown debris
    assert!(effects.ground_effects.iter().any(|(t, _)| *t == WeatherEffectType::Puddles));
    assert!(effects.ground_effects.iter().any(|(t, _)| *t == WeatherEffectType::DustDeposit));
}

#[test]
fn test_effect_cleanup() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    // Start with heavy rain
    let mut weather_state = WeatherState::default();
    weather_state.precipitation = 0.9;
    weather_state.weather = Weather::Rain;
    app.insert_resource(weather_state);
    
    // Run for a while to accumulate effects
    for _ in 0..10 {
        app.update();
    }
    
    // Switch to clear weather
    {
        let mut weather = app.world.get_resource_mut::<WeatherState>().unwrap();
        weather.weather = Weather::Clear;
        weather.precipitation = 0.0;
    }
    
    // Run for long enough to clean up effects
    for _ in 0..100 {
        app.update();
    }
    
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    assert!(effects.active_effects.is_empty());
    assert!(effects.ground_effects.is_empty());
}

#[test]
fn test_temperature_based_effects() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AudioPlugin))
        .add_plugins(WeatherPlugin);
    
    let mut weather_state = WeatherState::default();
    weather_state.precipitation = 0.7;
    weather_state.temperature = 30.0; // Hot weather
    weather_state.weather = Weather::Clear;
    app.insert_resource(weather_state);
    
    app.update();
    
    let effects = app.world.get_resource::<WeatherEffects>().unwrap();
    // Should see heat distortion effects
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::HeatDistortion));
} 