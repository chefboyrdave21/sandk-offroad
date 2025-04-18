use bevy::prelude::*;
use crate::game::plugins::weather::time_of_day::TimeOfDayManager;

#[derive(Resource)]
pub struct SkySystem {
    // Atmospheric scattering parameters
    rayleigh_coefficient: f32,
    mie_coefficient: f32,
    rayleigh_height: f32,
    mie_height: f32,
    mie_direction: f32,
    
    // Sky colors
    zenith_color: Color,
    horizon_color: Color,
    ground_color: Color,
    
    // Cloud parameters
    cloud_coverage: f32,
    cloud_density: f32,
    cloud_speed: f32,
}

impl Default for SkySystem {
    fn default() -> Self {
        Self {
            // Atmospheric scattering defaults based on Earth's atmosphere
            rayleigh_coefficient: 1.0,
            mie_coefficient: 0.5,
            rayleigh_height: 8.0,
            mie_height: 1.2,
            mie_direction: 0.758,  // g parameter for Mie scattering
            
            // Default sky colors
            zenith_color: Color::rgb(0.0, 0.4, 0.8),
            horizon_color: Color::rgb(0.8, 0.9, 1.0),
            ground_color: Color::rgb(0.2, 0.2, 0.2),
            
            // Default cloud parameters
            cloud_coverage: 0.5,
            cloud_density: 1.0,
            cloud_speed: 1.0,
        }
    }
}

impl SkySystem {
    pub fn update(&mut self, time_of_day: &TimeOfDayManager) {
        // Update sky colors based on sun position and time of day
        self.update_sky_colors(time_of_day);
        
        // Update atmospheric scattering based on sun angle
        self.update_scattering(time_of_day);
    }
    
    fn update_sky_colors(&mut self, time_of_day: &TimeOfDayManager) {
        let sun_height = time_of_day.get_sun_direction().y;
        let moon_height = time_of_day.get_moon_direction().y;
        
        // Base sky colors for different times of day
        let day_zenith = Color::rgb(0.0, 0.4, 0.8);    // Deep blue
        let day_horizon = Color::rgb(0.8, 0.9, 1.0);   // Light blue
        let night_zenith = Color::rgb(0.0, 0.0, 0.1);  // Dark blue
        let night_horizon = Color::rgb(0.1, 0.1, 0.2); // Dark blue-grey
        let dawn_zenith = Color::rgb(0.2, 0.4, 0.6);   // Medium blue
        let dawn_horizon = Color::rgb(1.0, 0.8, 0.6);  // Orange
        
        // Determine time of day phase
        let (phase, blend) = match time_of_day.current_time {
            t if t < 6.0 => ("night", 0.0),
            t if t < 8.0 => ("dawn", (t - 6.0) / 2.0),
            t if t < 18.0 => ("day", 1.0),
            t if t < 20.0 => ("dusk", (20.0 - t) / 2.0),
            _ => ("night", 0.0),
        };
        
        // Blend colors based on phase
        self.zenith_color = match phase {
            "dawn" | "dusk" => lerp_color(night_zenith, dawn_zenith, blend),
            "day" => lerp_color(dawn_zenith, day_zenith, blend),
            _ => night_zenith,
        };
        
        self.horizon_color = match phase {
            "dawn" | "dusk" => lerp_color(night_horizon, dawn_horizon, blend),
            "day" => lerp_color(dawn_horizon, day_horizon, blend),
            _ => night_horizon,
        };
        
        // Ground color is always darker but influenced by ambient light
        self.ground_color = Color::rgb(
            time_of_day.ambient_color.r() * 0.2,
            time_of_day.ambient_color.g() * 0.2,
            time_of_day.ambient_color.b() * 0.2,
        );
    }
    
    fn update_scattering(&mut self, time_of_day: &TimeOfDayManager) {
        let sun_height = time_of_day.get_sun_direction().y;
        
        // Adjust Rayleigh scattering based on sun height
        // More blue during the day, more neutral at dawn/dusk
        self.rayleigh_coefficient = if sun_height > 0.0 {
            1.0 + sun_height * 0.2  // Stronger during day
        } else {
            0.8  // Reduced at night
        };
        
        // Adjust Mie scattering for sunrise/sunset effects
        let sun_angle = sun_height.abs();
        if sun_angle < 0.2 {  // Near horizon
            self.mie_coefficient = 1.0;  // Increased scattering
            self.mie_direction = 0.85;   // More forward scattering
        } else {
            self.mie_coefficient = 0.5;  // Normal scattering
            self.mie_direction = 0.76;   // Normal direction
        }
    }
    
    pub fn set_cloud_parameters(&mut self, coverage: f32, density: f32, speed: f32) {
        self.cloud_coverage = coverage.clamp(0.0, 1.0);
        self.cloud_density = density.clamp(0.0, 2.0);
        self.cloud_speed = speed.clamp(0.0, 5.0);
    }
    
    // Getters for shader parameters
    pub fn get_scattering_parameters(&self) -> (f32, f32, f32, f32, f32) {
        (
            self.rayleigh_coefficient,
            self.mie_coefficient,
            self.rayleigh_height,
            self.mie_height,
            self.mie_direction
        )
    }
    
    pub fn get_sky_colors(&self) -> (Color, Color, Color) {
        (self.zenith_color, self.horizon_color, self.ground_color)
    }
    
    pub fn get_cloud_parameters(&self) -> (f32, f32, f32) {
        (self.cloud_coverage, self.cloud_density, self.cloud_speed)
    }
}

// Helper function to linearly interpolate between colors
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::rgb(
        a.r() + (b.r() - a.r()) * t,
        a.g() + (b.g() - a.g()) * t,
        a.b() + (b.b() - a.b()) * t,
    )
} 