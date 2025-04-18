use bevy::prelude::*;
use crate::game::plugins::weather::{
    time_of_day::TimeOfDayManager,
    sky_system::SkySystem,
    weather_system::WeatherSystem,
};

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TimeOfDayManager>()
            .init_resource::<SkySystem>()
            .add_systems(Update, (
                update_environment,
                update_reflection_probes,
            ));
    }
}

// Main system to update all environment components
fn update_environment(
    mut time_of_day: ResMut<TimeOfDayManager>,
    mut sky_system: ResMut<SkySystem>,
    mut weather_system: ResMut<WeatherSystem>,
    time: Res<Time>,
) {
    // Update time of day
    time_of_day.update(time.delta_seconds());
    
    // Update sky system with new time of day
    sky_system.update(&time_of_day);
    
    // Update weather system with time of day influence
    weather_system.update_with_time(&time_of_day, time.delta_seconds());
}

// System to handle environment reflection probe updates
fn update_reflection_probes(
    mut commands: Commands,
    mut reflection_probes: Query<(Entity, &mut ReflectionProbe, &GlobalTransform)>,
    time_of_day: Res<TimeOfDayManager>,
    sky_system: Res<SkySystem>,
    weather_system: Res<WeatherSystem>,
) {
    for (entity, mut probe, transform) in reflection_probes.iter_mut() {
        // Check if probe needs update based on time or weather changes
        if needs_probe_update(&probe, &time_of_day, &weather_system) {
            // Get current environment parameters
            let (zenith_color, horizon_color, ground_color) = sky_system.get_sky_colors();
            let sun_direction = time_of_day.get_sun_direction();
            let moon_direction = time_of_day.get_moon_direction();
            
            // Update probe parameters
            probe.update_sky_colors(zenith_color, horizon_color, ground_color);
            probe.update_light_directions(sun_direction, moon_direction);
            
            // Apply weather influence
            if let Some(weather_state) = weather_system.current_state() {
                apply_weather_to_probe(&mut probe, weather_state);
            }
            
            // Schedule probe for rendering update
            commands.entity(entity).insert(NeedsUpdate);
        }
    }
}

// Helper struct for reflection probes
#[derive(Component)]
struct ReflectionProbe {
    last_update_time: f32,
    update_interval: f32,
    sky_influence: f32,
    weather_influence: f32,
}

impl Default for ReflectionProbe {
    fn default() -> Self {
        Self {
            last_update_time: 0.0,
            update_interval: 1.0,  // Update every second by default
            sky_influence: 1.0,
            weather_influence: 1.0,
        }
    }
}

impl ReflectionProbe {
    fn update_sky_colors(&mut self, zenith: Color, horizon: Color, ground: Color) {
        // Implementation would update the probe's cubemap with new sky colors
    }
    
    fn update_light_directions(&mut self, sun_dir: Vec3, moon_dir: Vec3) {
        // Implementation would update the probe's lighting direction
    }
}

// Helper component to mark probes that need rendering update
#[derive(Component)]
struct NeedsUpdate;

// Helper function to determine if a probe needs updating
fn needs_probe_update(
    probe: &ReflectionProbe,
    time_of_day: &TimeOfDayManager,
    weather_system: &WeatherSystem,
) -> bool {
    let current_time = time_of_day.current_time;
    let time_since_update = current_time - probe.last_update_time;
    
    // Update if enough time has passed or if weather has changed significantly
    time_since_update >= probe.update_interval || weather_system.has_significant_change()
}

// Helper function to apply weather effects to reflection probe
fn apply_weather_to_probe(probe: &mut ReflectionProbe, weather_state: &WeatherState) {
    // Implementation would modify probe parameters based on weather:
    // - Reduce reflection intensity during rain/snow
    // - Add fog influence
    // - Modify color based on cloud coverage
    // - etc.
} 