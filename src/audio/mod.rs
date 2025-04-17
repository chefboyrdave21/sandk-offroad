use bevy::prelude::*;
use bevy::audio::*;
use crate::game::Vehicle;
use bevy_rapier3d::prelude::CollisionEvent;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioAssets>()
            .add_systems(Update, (
                update_vehicle_sounds,
                handle_environment_sounds,
            ));
    }
}

#[derive(Resource)]
pub struct AudioAssets {
    pub engine_sound: Handle<AudioSource>,
    pub crash_sound: Handle<AudioSource>,
    pub ambient_sound: Handle<AudioSource>,
}

impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            engine_sound: asset_server.load("sounds/engine.ogg"),
            crash_sound: asset_server.load("sounds/crash.ogg"),
            ambient_sound: asset_server.load("sounds/ambient.ogg"),
        }
    }
}

fn update_vehicle_sounds(
    mut commands: Commands,
    vehicle_query: Query<(&Vehicle, &Transform)>,
    audio_assets: Res<AudioAssets>,
) {
    for (_vehicle, transform) in vehicle_query.iter() {
        let speed = transform.translation.length();
        let speed_percentage = (speed / 100.0).min(1.0);
        let volume = speed_percentage * 0.8 + 0.2;
        let pitch = speed_percentage * 0.5 + 0.75;

        commands.spawn(AudioBundle {
            source: audio_assets.engine_sound.clone(),
            settings: PlaybackSettings::LOOP
                .with_volume(Volume::new_relative(volume))
                .with_speed(pitch),
        });
    }
}

fn handle_environment_sounds(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    audio_assets: Res<AudioAssets>,
) {
    for event in collision_events.iter() {
        if let CollisionEvent::Started(_entity1, _entity2, _) = event {
            commands.spawn(AudioBundle {
                source: audio_assets.crash_sound.clone(),
                settings: PlaybackSettings::ONCE
                    .with_volume(Volume::new_relative(0.5))
                    .with_speed(1.0),
            });
        }
    }
} 