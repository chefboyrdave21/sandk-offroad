use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Resource)]
pub struct TimeOfDayManager {
    // Time tracking
    pub current_time: f32,  // 0.0 to 24.0 hours
    pub day_length: f32,    // Real-time seconds for one in-game day
    pub time_scale: f32,    // Multiplier for time progression
    
    // Celestial body positions
    pub sun_position: Vec3,
    pub moon_position: Vec3,
    
    // Lighting parameters
    pub sun_intensity: f32,
    pub moon_intensity: f32,
    pub ambient_intensity: f32,
    pub sun_color: Color,
    pub moon_color: Color,
    pub ambient_color: Color,
}

impl Default for TimeOfDayManager {
    fn default() -> Self {
        Self {
            current_time: 12.0,        // Start at noon
            day_length: 1200.0,        // 20 minutes real-time = 1 day
            time_scale: 1.0,           // Normal time progression
            sun_position: Vec3::ZERO,
            moon_position: Vec3::ZERO,
            sun_intensity: 1.0,
            moon_intensity: 0.3,
            ambient_intensity: 0.1,
            sun_color: Color::rgb(1.0, 0.95, 0.8),    // Warm sunlight
            moon_color: Color::rgb(0.6, 0.6, 0.8),    // Cool moonlight
            ambient_color: Color::rgb(0.5, 0.5, 0.6),  // Neutral ambient
        }
    }
}

impl TimeOfDayManager {
    pub fn update(&mut self, delta_seconds: f32) {
        // Update time
        let time_delta = (delta_seconds * self.time_scale) / self.day_length * 24.0;
        self.current_time = (self.current_time + time_delta) % 24.0;
        
        // Calculate sun and moon positions
        self.update_celestial_positions();
        
        // Update lighting parameters
        self.update_lighting();
    }
    
    fn update_celestial_positions(&mut self) {
        // Convert current time to radians (0-24 hours maps to 0-2π)
        let time_radians = (self.current_time / 24.0) * 2.0 * PI;
        
        // Calculate sun position (moves in an arc from east to west)
        let sun_height = time_radians.sin();
        let sun_horizontal = time_radians.cos();
        self.sun_position = Vec3::new(
            sun_horizontal,
            sun_height,
            0.0
        ).normalize() * 100.0;  // Distance from origin
        
        // Moon is opposite to the sun
        self.moon_position = -self.sun_position;
    }
    
    fn update_lighting(&mut self) {
        // Calculate sun contribution based on height
        let sun_height = self.sun_position.y / 100.0;  // Normalized height
        let sun_factor = (sun_height + 0.3).clamp(0.0, 1.0);  // Allow some light during dawn/dusk
        
        // Calculate moon contribution
        let moon_height = self.moon_position.y / 100.0;
        let moon_factor = (moon_height + 0.3).clamp(0.0, 1.0);
        
        // Update intensities
        self.sun_intensity = sun_factor;
        self.moon_intensity = moon_factor * 0.3;  // Moon is dimmer
        
        // Calculate time-based ambient light
        let base_ambient = 0.1;
        let time_ambient = match self.current_time {
            t if t < 6.0 => base_ambient,  // Night
            t if t < 8.0 => base_ambient + (t - 6.0) / 2.0 * 0.2,  // Dawn
            t if t < 18.0 => base_ambient + 0.2,  // Day
            t if t < 20.0 => base_ambient + (20.0 - t) / 2.0 * 0.2,  // Dusk
            _ => base_ambient,  // Night
        };
        self.ambient_intensity = time_ambient;
        
        // Update colors based on time of day
        self.update_colors();
    }
    
    fn update_colors(&mut self) {
        // Base colors
        let day_sun = Color::rgb(1.0, 0.95, 0.8);     // Warm daylight
        let dawn_sun = Color::rgb(1.0, 0.8, 0.6);     // Orange sunrise/sunset
        let night_moon = Color::rgb(0.6, 0.6, 0.8);   // Cool moonlight
        
        // Interpolate colors based on time of day
        self.sun_color = match self.current_time {
            t if t < 6.0 => night_moon,  // Night
            t if t < 8.0 => dawn_sun,    // Dawn
            t if t < 18.0 => day_sun,    // Day
            t if t < 20.0 => dawn_sun,   // Dusk
            _ => night_moon,             // Night
        };
        
        // Ambient color follows a similar pattern but more muted
        self.ambient_color = match self.current_time {
            t if t < 6.0 => Color::rgb(0.3, 0.3, 0.4),  // Night
            t if t < 8.0 => Color::rgb(0.4, 0.35, 0.3), // Dawn
            t if t < 18.0 => Color::rgb(0.5, 0.5, 0.5), // Day
            t if t < 20.0 => Color::rgb(0.4, 0.35, 0.3), // Dusk
            _ => Color::rgb(0.3, 0.3, 0.4),             // Night
        };
    }
    
    pub fn get_sun_direction(&self) -> Vec3 {
        self.sun_position.normalize()
    }
    
    pub fn get_moon_direction(&self) -> Vec3 {
        self.moon_position.normalize()
    }
    
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.clamp(0.0, 100.0);  // Limit to reasonable range
    }
    
    pub fn set_time(&mut self, hours: f32) {
        self.current_time = hours.clamp(0.0, 24.0);
        self.update_celestial_positions();
        self.update_lighting();
    }
} 