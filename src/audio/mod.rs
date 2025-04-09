use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_audio)
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

fn setup_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(AudioAssets {
        engine_sound: asset_server.load("sounds/engine.ogg"),
        crash_sound: asset_server.load("sounds/crash.ogg"),
        ambient_sound: asset_server.load("sounds/ambient.ogg"),
    });

    // Start ambient sound
    commands.spawn(AudioBundle {
        source: asset_server.load("sounds/ambient.ogg"),
        settings: PlaybackSettings::LOOP.with_volume(0.3),
    });
}

fn update_vehicle_sounds(
    mut commands: Commands,
    vehicle_query: Query<(&Vehicle, &Transform)>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    for (vehicle, transform) in vehicle_query.iter() {
        let volume = (vehicle.current_speed / vehicle.max_speed).min(1.0);
        let pitch = 0.8 + (volume * 0.4);
        
        audio.set_volume(audio_assets.engine_sound.clone(), volume as f64);
        audio.set_playback_rate(audio_assets.engine_sound.clone(), pitch as f64);
    }
}

fn handle_environment_sounds(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    for event in collision_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            // Play crash sound on collision
            audio.play(audio_assets.crash_sound.clone())
                .with_volume(0.5)
                .with_playback_rate(1.0);
        }
    }
} 