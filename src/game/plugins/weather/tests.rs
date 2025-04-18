#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::Time;
    use crate::game::plugins::weather::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(WeatherPlugin)
            .init_resource::<WeatherState>();
        app
    }

    #[test]
    fn test_weather_state_initialization() {
        let app = setup_test_app();
        let weather_state = app.world.resource::<WeatherState>();
        
        assert_eq!(weather_state.current_weather, Weather::Clear);
        assert_eq!(weather_state.target_weather, Weather::Clear);
        assert_eq!(weather_state.transition_progress, 1.0);
        assert_eq!(weather_state.time_of_day, 0.5); // Assuming noon is default
    }

    #[test]
    fn test_time_of_day_update() {
        let mut app = setup_test_app();
        
        // Advance time by 1 second
        app.world.resource_mut::<Time>().advance_by(std::time::Duration::from_secs(1));
        app.update();

        let weather_state = app.world.resource::<WeatherState>();
        assert!(weather_state.time_of_day > 0.5); // Time should have advanced
    }

    #[test]
    fn test_weather_transition() {
        let mut app = setup_test_app();
        let mut weather_state = app.world.resource_mut::<WeatherState>();
        
        // Start transition to rain
        weather_state.target_weather = Weather::Rain;
        assert_eq!(weather_state.current_weather, Weather::Clear);
        
        // Advance time to allow transition
        app.world.resource_mut::<Time>().advance_by(std::time::Duration::from_secs(2));
        app.update();

        let weather_state = app.world.resource::<WeatherState>();
        assert!(weather_state.transition_progress < 1.0); // Should be transitioning
    }

    #[test]
    fn test_volumetric_settings() {
        let mut app = setup_test_app();
        let mut weather_state = app.world.resource_mut::<WeatherState>();
        
        // Test clear weather
        weather_state.current_weather = Weather::Clear;
        let clear_density = weather_state.get_volumetric_settings().density;
        
        // Test foggy weather
        weather_state.current_weather = Weather::Fog;
        let fog_density = weather_state.get_volumetric_settings().density;
        
        assert!(fog_density > clear_density); // Fog should be denser
    }

    #[test]
    fn test_light_settings() {
        let mut app = setup_test_app();
        let mut weather_state = app.world.resource_mut::<WeatherState>();
        
        // Test clear weather
        weather_state.current_weather = Weather::Clear;
        let clear_intensity = weather_state.get_light_settings().intensity;
        
        // Test stormy weather
        weather_state.current_weather = Weather::Storm;
        let storm_intensity = weather_state.get_light_settings().intensity;
        
        assert!(clear_intensity > storm_intensity); // Clear should be brighter
    }

    #[test]
    fn test_particle_effects() {
        let mut app = setup_test_app();
        let mut weather_state = app.world.resource_mut::<WeatherState>();
        
        // Test rain particles
        weather_state.current_weather = Weather::Rain;
        let rain_particles = weather_state.get_particle_settings();
        assert!(rain_particles.spawn_rate > 0.0);
        
        // Test snow particles
        weather_state.current_weather = Weather::Snow;
        let snow_particles = weather_state.get_particle_settings();
        assert!(snow_particles.spawn_rate > 0.0);
        assert!(snow_particles.gravity < rain_particles.gravity); // Snow should fall slower
    }
} 