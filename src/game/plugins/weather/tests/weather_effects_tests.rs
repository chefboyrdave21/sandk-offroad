use bevy::prelude::*;
use crate::game::plugins::weather::{
    WeatherEffects,
    WeatherEffectType,
    WeatherState,
    Weather,
    TimeOfDay,
};

#[test]
fn test_weather_effects_initialization() {
    let effects = WeatherEffects::default();
    assert_eq!(effects.active_effects.len(), 0);
    assert_eq!(effects.max_effects, 3);
    assert_eq!(effects.transitions.len(), 0);
}

#[test]
fn test_rain_effects_creation() {
    let mut effects = WeatherEffects::default();
    let mut weather_state = WeatherState::new(Weather::Rain);
    weather_state.precipitation = 0.8;
    
    effects.update(&weather_state, TimeOfDay::Noon);
    
    // Should create HeavyRain effect due to high precipitation
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::HeavyRain));
}

#[test]
fn test_weather_transitions() {
    let mut effects = WeatherEffects::default();
    
    // Start transition
    effects.transition_effect(WeatherEffectType::Rain, 1.0);
    assert_eq!(effects.transitions.len(), 1);
    
    // Update transitions
    effects.update_transitions(0.5);
    let (_, intensity, _) = effects.transitions[0];
    assert!(intensity > 0.0);
}

#[test]
fn test_time_of_day_color_adjustments() {
    let effects = WeatherEffects::default();
    let base_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
    
    let night_color = effects.get_time_adjusted_color(base_color, TimeOfDay::Night);
    let noon_color = effects.get_time_adjusted_color(base_color, TimeOfDay::Noon);
    
    assert_ne!(night_color, noon_color);
}

#[test]
fn test_effect_intensity_thresholds() {
    let mut effects = WeatherEffects::default();
    let mut weather_state = WeatherState::new(Weather::Snow);
    
    // Test low intensity
    weather_state.precipitation = 0.2;
    effects.update(&weather_state, TimeOfDay::Noon);
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::LightSnow));
    
    // Test high intensity
    weather_state.precipitation = 0.8;
    weather_state.wind_speed = 12.0;
    effects.update(&weather_state, TimeOfDay::Noon);
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::Blizzard));
}

#[test]
fn test_storm_effect_combinations() {
    let mut effects = WeatherEffects::default();
    let mut weather_state = WeatherState::new(Weather::Storm);
    weather_state.precipitation = 0.9;
    weather_state.wind_speed = 15.0;
    
    effects.update(&weather_state, TimeOfDay::Night);
    
    // Should have multiple effects active
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::HeavyRain));
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::ThunderStorm));
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::Hail));
}

#[test]
fn test_effect_cleanup() {
    let mut effects = WeatherEffects::default();
    let weather_state = WeatherState::new(Weather::Rain);
    
    // Create some effects
    effects.update(&weather_state, TimeOfDay::Noon);
    assert!(!effects.active_effects.is_empty());
    
    // Clear effects
    let mut commands = Commands::default();
    effects.clear_effects(&mut commands);
    assert!(effects.active_effects.is_empty());
}

#[test]
fn test_fog_density_variations() {
    let mut effects = WeatherEffects::default();
    let mut weather_state = WeatherState::new(Weather::Fog);
    
    // Test different fog densities
    weather_state.fog_density = 0.2;
    effects.update(&weather_state, TimeOfDay::Morning);
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::Mist));
    
    weather_state.fog_density = 0.5;
    effects.update(&weather_state, TimeOfDay::Morning);
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::Fog));
    
    weather_state.fog_density = 0.8;
    effects.update(&weather_state, TimeOfDay::Morning);
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::ThickFog));
}

#[test]
fn test_wind_based_effects() {
    let mut effects = WeatherEffects::default();
    let mut weather_state = WeatherState::new(Weather::Clear);
    
    // Test wind speed thresholds
    weather_state.wind_speed = 9.0;
    effects.update(&weather_state, TimeOfDay::Noon);
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::Dust));
    
    weather_state.wind_speed = 13.0;
    effects.update(&weather_state, TimeOfDay::Noon);
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::Sandstorm));
}

#[test]
fn test_freezing_rain_creation() {
    let mut effects = WeatherEffects::default();
    let mut weather_state = WeatherState::new(Weather::Rain);
    weather_state.precipitation = 0.5;
    weather_state.temperature = -5.0;
    
    effects.update(&weather_state, TimeOfDay::Night);
    
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::FreezingRain));
}

#[test]
fn test_rainbow_mist_at_dawn() {
    let mut effects = WeatherEffects::default();
    let mut weather_state = WeatherState::new(Weather::Fog);
    weather_state.fog_density = 0.3;
    
    effects.update(&weather_state, TimeOfDay::Dawn);
    
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::RainbowMist));
}

#[test]
fn test_wind_influence_on_particles() {
    let mut effects = WeatherEffects::default();
    let base_velocity = Vec3::new(0.0, -20.0, 0.0);
    let wind_speed = 10.0;
    let wind_direction = Vec2::new(1.0, 0.0);
    
    let influenced_velocity = effects.get_wind_velocity(base_velocity, wind_speed, wind_direction);
    
    // Wind should affect x and z components but preserve y velocity
    assert_ne!(influenced_velocity.x, base_velocity.x);
    assert_eq!(influenced_velocity.y, base_velocity.y);
    assert_ne!(influenced_velocity.z, base_velocity.z);
}

#[test]
fn test_time_of_day_parameter_adjustments() {
    let effects = WeatherEffects::default();
    
    let (night_vis, night_size, night_life) = effects.get_time_adjusted_params(TimeOfDay::Night);
    let (noon_vis, noon_size, noon_life) = effects.get_time_adjusted_params(TimeOfDay::Noon);
    
    // Night should have reduced visibility but increased particle size and lifetime
    assert!(night_vis < noon_vis);
    assert!(night_size > noon_size);
    assert!(night_life > noon_life);
}

#[test]
fn test_random_variation_updates() {
    let mut effects = WeatherEffects::default();
    let initial_seed = effects.random_seed;
    
    effects.update_random(0.016);
    
    assert_ne!(effects.random_seed, initial_seed);
}

#[test]
fn test_extreme_weather_combinations() {
    let mut effects = WeatherEffects::default();
    let mut weather_state = WeatherState::new(Weather::Storm);
    weather_state.precipitation = 0.9;
    weather_state.wind_speed = 15.0;
    weather_state.temperature = -15.0;
    
    effects.update(&weather_state, TimeOfDay::Night);
    
    // Should have multiple effects including ice particles
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::IceParticles));
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::FreezingRain));
}

#[test]
fn test_weather_effect_transitions() {
    let mut effects = WeatherEffects::default();
    
    // Start a transition
    effects.transition_effect(WeatherEffectType::Rain, 1.0);
    let initial_transitions = effects.transitions.len();
    
    // Update transitions
    effects.update_transitions(0.5);
    
    // Transition should still be in progress
    assert_eq!(effects.transitions.len(), initial_transitions);
    
    // Complete transition
    for _ in 0..10 {
        effects.update_transitions(0.5);
    }
    
    // Transition should be complete and removed
    assert!(effects.transitions.is_empty());
}

#[test]
fn test_dawn_dusk_special_effects() {
    let mut effects = WeatherEffects::default();
    let mut weather_state = WeatherState::new(Weather::Clear);
    weather_state.fog_density = 0.4;
    
    // Test dawn effects
    effects.update(&weather_state, TimeOfDay::Dawn);
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::RainbowMist));
    
    effects.clear_effects(&mut Commands::default());
    
    // Test dusk effects
    effects.update(&weather_state, TimeOfDay::Dusk);
    assert!(effects.active_effects.iter().any(|(t, _)| *t == WeatherEffectType::RainbowMist));
} 