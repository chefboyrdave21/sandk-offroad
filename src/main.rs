use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

mod core;
mod game;
mod physics;
mod rendering;
mod audio;
mod ui;
mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(bevy_rapier3d::RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(bevy_rapier3d::RapierDebugRenderPlugin::default())
        .add_plugin(core::CorePlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(rendering::RenderingPlugin)
        .add_plugin(audio::AudioPlugin)
        .add_plugin(ui::UiPlugin)
        .run();
} 