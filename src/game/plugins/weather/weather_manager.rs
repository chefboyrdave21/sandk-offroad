use bevy::prelude::*;
use std::time::Duration;

/// Different types of weather conditions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Weather {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Fog,
    Snow,
}

/// Represents the current state of the weather, including transition effects
#[derive(Debug, Clone)]
pub struct WeatherState {
    /// Current primary weather type
    weather: Weather,
    /// Weather we're transitioning to (if any)
    transitioning_to: Option<Weather>,
    /// Transition progress (0.0 - 1.0)
    transition_progress: f32,
    /// Current cloud coverage (0.0 - 1.0)
    cloud_coverage: f32,
    /// Current precipitation intensity (0.0 - 1.0)
    precipitation: f32,
    /// Current wind speed (m/s)
    wind_speed: f32,
    /// Current wind direction (radians)
    wind_direction: f32,
    /// Current fog density (0.0 - 1.0)
    fog_density: f32,
}

impl WeatherState {
    /// Create a new weather state with the given weather type
    pub fn new(weather: Weather) -> Self {
        let mut state = Self {
            weather,
            transitioning_to: None,
            transition_progress: 0.0,
            cloud_coverage: 0.0,
            precipitation: 0.0,
            wind_speed: 0.0,
            wind_direction: 0.0,
            fog_density: 0.0,
        };
        state.apply_weather_parameters(weather);
        state
    }

    /// Apply base parameters for a weather type
    fn apply_weather_parameters(&mut self, weather: Weather) {
        match weather {
            Weather::Clear => {
                self.cloud_coverage = 0.1;
                self.precipitation = 0.0;
                self.wind_speed = 2.0;
                self.fog_density = 0.0;
            }
            Weather::Cloudy => {
                self.cloud_coverage = 0.7;
                self.precipitation = 0.0;
                self.wind_speed = 5.0;
                self.fog_density = 0.1;
            }
            Weather::Rain => {
                self.cloud_coverage = 0.9;
                self.precipitation = 0.6;
                self.wind_speed = 8.0;
                self.fog_density = 0.3;
            }
            Weather::Storm => {
                self.cloud_coverage = 1.0;
                self.precipitation = 1.0;
                self.wind_speed = 15.0;
                self.fog_density = 0.4;
            }
            Weather::Fog => {
                self.cloud_coverage = 0.5;
                self.precipitation = 0.0;
                self.wind_speed = 2.0;
                self.fog_density = 0.8;
            }
            Weather::Snow => {
                self.cloud_coverage = 0.8;
                self.precipitation = 0.7;
                self.wind_speed = 4.0;
                self.fog_density = 0.2;
            }
        }
    }

    /// Get the light intensity modifier for the current weather
    pub fn light_intensity_modifier(&self) -> f32 {
        let base = match self.weather {
            Weather::Clear => 1.0,
            Weather::Cloudy => 0.8,
            Weather::Rain => 0.6,
            Weather::Storm => 0.4,
            Weather::Fog => 0.7,
            Weather::Snow => 0.9,
        };

        if let Some(target) = self.transitioning_to {
            let target_mod = match target {
                Weather::Clear => 1.0,
                Weather::Cloudy => 0.8,
                Weather::Rain => 0.6,
                Weather::Storm => 0.4,
                Weather::Fog => 0.7,
                Weather::Snow => 0.9,
            };
            base.lerp(target_mod, self.transition_progress)
        } else {
            base
        }
    }

    /// Get the ambient color modifier for the current weather
    pub fn ambient_color_modifier(&self) -> Color {
        let base = match self.weather {
            Weather::Clear => Color::rgb(1.0, 1.0, 1.0),
            Weather::Cloudy => Color::rgb(0.9, 0.9, 0.95),
            Weather::Rain => Color::rgb(0.7, 0.7, 0.8),
            Weather::Storm => Color::rgb(0.6, 0.6, 0.7),
            Weather::Fog => Color::rgb(0.8, 0.8, 0.85),
            Weather::Snow => Color::rgb(1.0, 1.0, 1.1),
        };

        if let Some(target) = self.transitioning_to {
            let target_color = match target {
                Weather::Clear => Color::rgb(1.0, 1.0, 1.0),
                Weather::Cloudy => Color::rgb(0.9, 0.9, 0.95),
                Weather::Rain => Color::rgb(0.7, 0.7, 0.8),
                Weather::Storm => Color::rgb(0.6, 0.6, 0.7),
                Weather::Fog => Color::rgb(0.8, 0.8, 0.85),
                Weather::Snow => Color::rgb(1.0, 1.0, 1.1),
            };
            base.lerp(target_color, self.transition_progress)
        } else {
            base
        }
    }

    /// Get the ambient light intensity modifier
    pub fn ambient_intensity_modifier(&self) -> f32 {
        1.0 - (self.cloud_coverage * 0.5)
    }
}

/// Resource that manages weather transitions and state
#[derive(Resource)]
pub struct WeatherManager {
    /// Current weather state
    state: WeatherState,
    /// Duration of weather transitions
    transition_duration: Duration,
    /// Minimum time between random weather changes
    min_change_interval: Duration,
    /// Time since last weather change
    time_since_change: Duration,
}

impl Default for WeatherManager {
    fn default() -> Self {
        Self {
            state: WeatherState::new(Weather::Clear),
            transition_duration: Duration::from_secs(30),
            min_change_interval: Duration::from_secs(300),
            time_since_change: Duration::ZERO,
        }
    }
}

impl WeatherManager {
    /// Update the weather manager with elapsed time
    pub fn update(&mut self, delta_seconds: f32) {
        let delta = Duration::from_secs_f32(delta_seconds);
        self.time_since_change += delta;

        // Update transition if in progress
        if let Some(target) = self.state.transitioning_to {
            self.state.transition_progress += delta_seconds / self.transition_duration.as_secs_f32();
            
            if self.state.transition_progress >= 1.0 {
                // Transition complete
                self.state.weather = target;
                self.state.transitioning_to = None;
                self.state.transition_progress = 0.0;
                self.state.apply_weather_parameters(target);
            }
        }

        // Random weather changes (disabled for now, will be controlled by game logic)
        /*
        if self.time_since_change >= self.min_change_interval {
            // 5% chance per second to change weather
            if rand::random::<f32>() < 0.05 * delta_seconds {
                self.change_weather(self.random_weather());
            }
        }
        */
    }

    /// Get the current weather state
    pub fn current_state(&self) -> &WeatherState {
        &self.state
    }

    /// Change to a new weather type with transition
    pub fn change_weather(&mut self, weather: Weather) {
        if weather != self.state.weather && self.state.transitioning_to.is_none() {
            self.state.transitioning_to = Some(weather);
            self.state.transition_progress = 0.0;
            self.time_since_change = Duration::ZERO;
        }
    }

    /// Set the weather transition duration
    pub fn set_transition_duration(&mut self, duration: Duration) {
        self.transition_duration = duration;
    }

    /// Get a random weather type (excluding current)
    fn random_weather(&self) -> Weather {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let options = [
            Weather::Clear,
            Weather::Cloudy,
            Weather::Rain,
            Weather::Storm,
            Weather::Fog,
            Weather::Snow,
        ];
        
        loop {
            let weather = options[rng.gen_range(0..options.len())];
            if weather != self.state.weather {
                return weather;
            }
        }
    }
} 