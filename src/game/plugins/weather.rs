use bevy::prelude::*;
use std::f32::consts::PI;

use crate::game::plugins::{
    lighting::{LightTemperature, VolumetricSettings},
    particle_system::{ParticlePresets, PresetConfig},
};

#[derive(Resource)]
pub struct WeatherState {
    pub current_weather: Weather,
    pub target_weather: Weather,
    pub transition_progress: f32,
    pub transition_duration: f32,
    pub time_of_day: f32, // 0.0 to 1.0, representing 24 hours
    pub day_cycle_duration: f32, // Duration of a full day in seconds
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            current_weather: Weather::Clear,
            target_weather: Weather::Clear,
            transition_progress: 0.0,
            transition_duration: 10.0, // Weather transitions take 10 seconds
            time_of_day: 0.5, // Start at noon
            day_cycle_duration: 600.0, // 10 minutes per day cycle
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Fog,
    Snow,
}

impl Weather {
    pub fn get_volumetric_settings(&self) -> VolumetricSettings {
        match self {
            Weather::Clear => VolumetricSettings {
                density: 0.1,
                scattering: 0.8,
                absorption: 0.1,
                max_distance: 100.0,
            },
            Weather::Cloudy => VolumetricSettings {
                density: 0.4,
                scattering: 0.7,
                absorption: 0.3,
                max_distance: 80.0,
            },
            Weather::Rain => VolumetricSettings {
                density: 0.5,
                scattering: 0.6,
                absorption: 0.4,
                max_distance: 60.0,
            },
            Weather::Storm => VolumetricSettings {
                density: 0.7,
                scattering: 0.5,
                absorption: 0.6,
                max_distance: 40.0,
            },
            Weather::Fog => VolumetricSettings {
                density: 0.8,
                scattering: 0.9,
                absorption: 0.2,
                max_distance: 30.0,
            },
            Weather::Snow => VolumetricSettings {
                density: 0.6,
                scattering: 0.9,
                absorption: 0.3,
                max_distance: 50.0,
            },
        }
    }

    pub fn get_light_settings(&self) -> (f32, Color) {
        match self {
            Weather::Clear => (100000.0, Color::rgb(1.0, 1.0, 1.0)),
            Weather::Cloudy => (70000.0, Color::rgb(0.9, 0.9, 0.9)),
            Weather::Rain => (50000.0, Color::rgb(0.7, 0.7, 0.8)),
            Weather::Storm => (30000.0, Color::rgb(0.6, 0.6, 0.7)),
            Weather::Fog => (40000.0, Color::rgb(0.8, 0.8, 0.8)),
            Weather::Snow => (80000.0, Color::rgb(1.0, 1.0, 1.1)),
        }
    }
}

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherState>()
            .add_systems(Update, (
                update_time_of_day,
                update_weather_transition,
                update_lighting,
                update_weather_effects,
            ));
    }
}

fn update_time_of_day(
    time: Res<Time>,
    mut weather_state: ResMut<WeatherState>,
) {
    weather_state.time_of_day += time.delta_seconds() / weather_state.day_cycle_duration;
    if weather_state.time_of_day >= 1.0 {
        weather_state.time_of_day -= 1.0;
    }
}

fn update_weather_transition(
    time: Res<Time>,
    mut weather_state: ResMut<WeatherState>,
) {
    if weather_state.current_weather != weather_state.target_weather {
        weather_state.transition_progress += time.delta_seconds() / weather_state.transition_duration;
        if weather_state.transition_progress >= 1.0 {
            weather_state.current_weather = weather_state.target_weather;
            weather_state.transition_progress = 0.0;
        }
    }
}

fn update_lighting(
    weather_state: Res<WeatherState>,
    mut directional_light: Query<(&mut DirectionalLight, &mut Transform, &mut LightTemperature)>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    let time_angle = weather_state.time_of_day * PI * 2.0;
    let sun_height = time_angle.sin();
    
    // Get base light settings from current weather
    let (base_illuminance, base_color) = weather_state.current_weather.get_light_settings();
    
    // If transitioning, interpolate between current and target weather
    let (target_illuminance, target_color) = if weather_state.transition_progress > 0.0 {
        let target_settings = weather_state.target_weather.get_light_settings();
        let t = weather_state.transition_progress;
        (
            base_illuminance * (1.0 - t) + target_settings.0 * t,
            Color::rgb(
                base_color.r() * (1.0 - t) + target_color.r() * t,
                base_color.g() * (1.0 - t) + target_color.g() * t,
                base_color.b() * (1.0 - t) + target_color.b() * t,
            ),
        )
    } else {
        (base_illuminance, base_color)
    };

    // Update directional light (sun)
    if let Ok((mut light, mut transform, mut temperature)) = directional_light.get_single_mut() {
        // Update position
        let distance = 100.0;
        transform.translation = Vec3::new(
            time_angle.cos() * distance,
            sun_height * distance,
            time_angle.sin() * distance,
        );
        transform.look_at(Vec3::ZERO, Vec3::Y);

        // Update light properties
        let day_factor = ((sun_height + 1.0) * 0.5).clamp(0.1, 1.0);
        light.illuminance = target_illuminance * day_factor;
        light.color = target_color;
        
        // Update color temperature (warmer at sunrise/sunset)
        let base_temp = 6500.0;
        let sunset_temp = 2500.0;
        let sunset_factor = 1.0 - (sun_height.abs() * 2.0 - 1.0).abs();
        temperature.kelvin = base_temp * (1.0 - sunset_factor) + sunset_temp * sunset_factor;
    }

    // Update ambient light
    let ambient_intensity = ((sun_height + 1.0) * 0.3).clamp(0.1, 0.3);
    ambient_light.brightness = ambient_intensity;
    ambient_light.color = target_color;
}

fn update_weather_effects(
    mut commands: Commands,
    weather_state: Res<WeatherState>,
    time: Res<Time>,
) {
    // Spawn weather particles based on current weather
    match weather_state.current_weather {
        Weather::Rain | Weather::Storm => {
            // Spawn rain particles in a grid above the player
            let bounds = Vec3::new(50.0, 20.0, 50.0);
            let intensity = if weather_state.current_weather == Weather::Storm { 2.0 } else { 1.0 };
            
            for x in (-2..=2).step_by(1) {
                for z in (-2..=2).step_by(1) {
                    let position = Vec3::new(
                        x as f32 * 10.0,
                        bounds.y,
                        z as f32 * 10.0,
                    );
                    
                    let config = PresetConfig {
                        scale: 0.05,
                        intensity,
                        speed: 2.0,
                        lifetime: 1.5,
                        gravity: Vec3::new(0.0, -15.0, 0.0),
                        emission_strength: 0.5,
                    };
                    
                    ParticlePresets::water(
                        &mut commands,
                        Transform::from_translation(position),
                        Some(config),
                    );
                }
            }
            
            // Add lightning for storms
            if weather_state.current_weather == Weather::Storm {
                if rand::random::<f32>() < 0.01 { // 1% chance per frame
                    let strike_pos = Vec3::new(
                        rand::random::<f32>() * 100.0 - 50.0,
                        bounds.y,
                        rand::random::<f32>() * 100.0 - 50.0,
                    );
                    
                    ParticlePresets::lightning_strike(
                        &mut commands,
                        Transform::from_translation(strike_pos),
                        Some(PresetConfig {
                            scale: 2.0,
                            intensity: 0.3,
                            speed: 1.5,
                            lifetime: 0.2,
                            gravity: Vec3::ZERO,
                            emission_strength: 3.0,
                        }),
                    );
                }
            }
        },
        Weather::Snow => {
            // Spawn snow particles
            let bounds = Vec3::new(50.0, 20.0, 50.0);
            for x in (-2..=2).step_by(1) {
                for z in (-2..=2).step_by(1) {
                    let position = Vec3::new(
                        x as f32 * 10.0,
                        bounds.y,
                        z as f32 * 10.0,
                    );
                    
                    ParticlePresets::snow(
                        &mut commands,
                        Transform::from_translation(position),
                        Some(PresetConfig {
                            scale: 0.1,
                            intensity: 1.0,
                            speed: 1.0,
                            lifetime: 3.0,
                            gravity: Vec3::new(0.0, -5.0, 0.0),
                            emission_strength: 0.3,
                        }),
                    );
                }
            }
        },
        Weather::Fog => {
            // Spawn fog particles
            let bounds = Vec3::new(50.0, 10.0, 50.0);
            for x in (-2..=2).step_by(1) {
                for z in (-2..=2).step_by(1) {
                    let position = Vec3::new(
                        x as f32 * 10.0,
                        bounds.y * 0.5,
                        z as f32 * 10.0,
                    );
                    
                    ParticlePresets::fog(
                        &mut commands,
                        Transform::from_translation(position),
                        Some(PresetConfig {
                            scale: 5.0,
                            intensity: 0.3,
                            speed: 0.2,
                            lifetime: 5.0,
                            gravity: Vec3::ZERO,
                            emission_strength: 0.1,
                        }),
                    );
                }
            }
        },
        _ => (), // No particles for clear or cloudy weather
    }
} 