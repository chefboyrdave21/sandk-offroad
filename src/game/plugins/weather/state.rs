#[derive(Resource)]
pub struct WeatherState {
    // Base weather parameters
    pub temperature: f32,
    pub precipitation: f32,
    pub wind_speed: f32,
    pub wind_direction: Vec2,
    pub fog_density: f32,
    pub time_of_day: f32,

    // Rain parameters
    pub rain_intensity: f32,
    pub rain_drop_size: f32,
    pub rain_splash_size: f32,

    // Snow parameters
    pub snow_density: f32,
    pub snow_flake_size: f32,
    pub snow_drift_factor: f32,

    // Storm parameters
    pub lightning_frequency: f32,
    pub thunder_volume: f32,
    pub cloud_darkness: f32,

    // Fog parameters
    pub fog_height: f32,
    pub fog_falloff: f32,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            // Base parameters
            temperature: 20.0,
            precipitation: 0.0,
            wind_speed: 0.0,
            wind_direction: Vec2::new(1.0, 0.0),
            fog_density: 0.0,
            time_of_day: 12.0,

            // Rain parameters
            rain_intensity: 0.5,
            rain_drop_size: 1.0,
            rain_splash_size: 0.5,

            // Snow parameters
            snow_density: 0.5,
            snow_flake_size: 1.0,
            snow_drift_factor: 1.0,

            // Storm parameters
            lightning_frequency: 0.2,
            thunder_volume: 0.8,
            cloud_darkness: 0.7,

            // Fog parameters
            fog_height: 50.0,
            fog_falloff: 0.5,
        }
    }
} 