mod cloud_material;
mod noise_texture;
mod time_manager;
mod weather_manager;
mod weather_effects;

pub use cloud_material::{CloudMaterial, CloudParams};
pub use noise_texture::{NoiseTexturePlugin, CloudNoiseTextureHandles};
pub use time_manager::{TimeOfDay, TimeManager};
pub use weather_manager::{Weather, WeatherManager, WeatherState};
pub use weather_effects::{WeatherEffects, WeatherEffectType};

use bevy::prelude::*;

/// Plugin that handles all weather and time of day related systems
pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                MaterialPlugin::<CloudMaterial>::default(),
                NoiseTexturePlugin,
            ))
            .init_resource::<TimeManager>()
            .init_resource::<WeatherManager>()
            .init_resource::<WeatherEffects>()
            .add_systems(Update, (
                update_time_of_day,
                update_weather_state,
                update_weather_effects,
                update_environment_lighting,
            ));
    }
}

/// System that updates the time of day, including sun/moon position and lighting
fn update_time_of_day(
    mut time_manager: ResMut<TimeManager>,
    time: Res<Time>,
) {
    time_manager.update(time.delta_seconds());
}

/// System that handles weather state transitions and updates
fn update_weather_state(
    mut weather_manager: ResMut<WeatherManager>,
    time: Res<Time>,
) {
    weather_manager.update(time.delta_seconds());
}

/// System that updates active weather particle effects
fn update_weather_effects(
    mut effects: ResMut<WeatherEffects>,
    weather: Res<WeatherManager>,
    time: Res<TimeManager>,
) {
    effects.update(&weather.current_state(), time.time_of_day());
}

/// System that updates environment lighting based on time of day and weather
fn update_environment_lighting(
    time: Res<TimeManager>,
    weather: Res<WeatherManager>,
    mut query: Query<(&mut DirectionalLight, &mut Transform), With<DirectionalLight>>,
) {
    // Update main light (sun/moon) direction and intensity
    if let Ok((mut light, mut transform)) = query.get_single_mut() {
        let (direction, intensity) = time.get_main_light_params(&weather.current_state());
        
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction);
        light.illuminance = intensity;
    }
} 