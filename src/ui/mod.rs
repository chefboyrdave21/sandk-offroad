use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                update_hud,
                handle_menu_interactions,
            ));
    }
}

#[derive(Resource)]
pub struct UiState {
    pub show_menu: bool,
    pub show_debug: bool,
}

fn setup_ui(mut commands: Commands) {
    commands.insert_resource(UiState {
        show_menu: false,
        show_debug: false,
    });
}

fn update_hud(
    mut egui_context: ResMut<EguiContext>,
    vehicle_query: Query<&Vehicle>,
    state: Res<State<GameState>>,
    ui_state: Res<UiState>,
) {
    if state.get() != &GameState::Playing || ui_state.show_menu {
        return;
    }

    egui::Window::new("HUD")
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            if let Ok(vehicle) = vehicle_query.get_single() {
                ui.label(format!("Speed: {:.1} km/h", vehicle.current_speed * 3.6));
                
                // Speedometer
                let speed_percentage = vehicle.current_speed / vehicle.max_speed;
                ui.add(egui::ProgressBar::new(speed_percentage)
                    .text(format!("{:.0}%", speed_percentage * 100.0)));
            }
        });
}

fn handle_menu_interactions(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State<GameState>>,
    mut ui_state: ResMut<UiState>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        ui_state.show_menu = !ui_state.show_menu;
    }

    if !ui_state.show_menu {
        return;
    }

    egui::Window::new("Menu")
        .show(egui_context.ctx_mut(), |ui| {
            ui.heading("SandK Offroad");
            ui.separator();
            
            if ui.button("Resume").clicked() {
                ui_state.show_menu = false;
            }
            
            if ui.button("Settings").clicked() {
                // TODO: Show settings menu
            }
            
            if ui.button("Quit").clicked() {
                std::process::exit(0);
            }
        });
} 