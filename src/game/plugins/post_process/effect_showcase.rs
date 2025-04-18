use bevy::prelude::*;
use bevy::render::mesh::shape;
use std::collections::HashSet;

/// Component for tracking showcase state
#[derive(Component)]
struct ShowcaseState {
    timer: Timer,
    current_scene: usize,
}

/// Different showcase scenes demonstrating specific effects
#[derive(Clone)]
enum ShowcaseScene {
    NightScene,
    Underwater,
    HeatDistortion,
    VintageFilm,
}

impl ShowcaseScene {
    fn setup(&self, commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) {
        match self {
            ShowcaseScene::NightScene => setup_night_scene(commands, meshes, materials),
            ShowcaseScene::Underwater => setup_underwater_scene(commands, meshes, materials),
            ShowcaseScene::HeatDistortion => setup_heat_scene(commands, meshes, materials),
            ShowcaseScene::VintageFilm => setup_vintage_scene(commands, meshes, materials),
        }
    }

    fn get_post_process_settings(&self) -> PostProcessSettings {
        match self {
            ShowcaseScene::NightScene => PostProcessSettings {
                tone_mapping_type: 1, // ACES
                exposure: 2.0,
                bloom_intensity: 0.7,
                bloom_threshold: 0.8,
                vignette_strength: 0.4,
                vignette_radius: 0.8,
                color_temperature: 0.3, // Cool/blue night lighting
                ..default()
            },
            ShowcaseScene::Underwater => PostProcessSettings {
                tone_mapping_type: 2, // Reinhard
                exposure: 0.5,
                chromatic_aberration: 0.2,
                lens_distortion: 0.15,
                color_temperature: 0.4, // Cool/blue tint
                saturation: 1.2,
                ..default()
            },
            ShowcaseScene::HeatDistortion => PostProcessSettings {
                tone_mapping_type: 3, // Uncharted2
                exposure: 0.8,
                lens_distortion: 0.1,
                color_temperature: 0.7, // Warm tint
                bloom_intensity: 0.5,
                bloom_threshold: 0.7,
                ..default()
            },
            ShowcaseScene::VintageFilm => PostProcessSettings {
                tone_mapping_type: 2, // Reinhard
                exposure: 0.2,
                contrast: 1.3,
                saturation: 0.7,
                vignette_strength: 0.3,
                vignette_radius: 0.7,
                film_grain: 0.15,
                color_temperature: 0.6, // Slightly warm
                ..default()
            },
        }
    }
}

fn setup_night_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Moonlit environment with glowing elements
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 100.0,
            color: Color::rgb(0.8, 0.8, 1.0),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });

    // Glowing crystals
    for i in 0..5 {
        let angle = i as f32 * std::f32::consts::TAU / 5.0;
        let radius = 3.0;
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.3, subdivisions: 3 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.2, 0.4, 1.0),
                emissive: Color::rgb(0.4, 0.8, 2.0),
                ..default()
            }),
            transform: Transform::from_xyz(
                angle.cos() * radius,
                1.0,
                angle.sin() * radius,
            ),
            ..default()
        });
    }
}

fn setup_underwater_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Ambient blue lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 500.0,
            color: Color::rgb(0.5, 0.7, 1.0),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });

    // Coral formations
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let radius = 4.0;
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Torus { radius: 0.5, ring_radius: 0.2, ..default() })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 0.5, 0.3),
                metallic: 0.0,
                roughness: 0.8,
                ..default()
            }),
            transform: Transform::from_xyz(
                angle.cos() * radius,
                0.5 + (i as f32 * 0.3),
                angle.sin() * radius,
            ),
            ..default()
        });
    }
}

fn setup_heat_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Intense sunlight
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 2000.0,
            color: Color::rgb(1.0, 0.95, 0.8),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });

    // Hot metal objects
    for i in 0..3 {
        let z = (i as f32 - 1.0) * 2.0;
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule { radius: 0.3, rings: 3, depth: 1.0, ..default() })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.3, 0.1),
                emissive: Color::rgb(0.5, 0.2, 0.0),
                metallic: 1.0,
                roughness: 0.2,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.0, z),
            ..default()
        });
    }
}

fn setup_vintage_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Soft lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 800.0,
            color: Color::rgb(1.0, 0.98, 0.95),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });

    // Old-style objects
    let vintage_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.6, 0.5, 0.4),
        metallic: 0.0,
        roughness: 0.9,
        ..default()
    });

    // Arrange objects in a nostalgic composition
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::FRAC_PI_2;
        let radius = 2.0;
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.8, 1.5, 0.8))),
            material: vintage_material.clone(),
            transform: Transform::from_xyz(
                angle.cos() * radius,
                0.75,
                angle.sin() * radius,
            ),
            ..default()
        });
    }
}

/// Plugin for the effect showcase system
pub struct EffectShowcasePlugin;

impl Plugin for EffectShowcasePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PostProcessProfilingPlugin)
            .init_resource::<ShowcaseState>()
            .add_systems(Startup, setup_showcase)
            .add_systems(Update, (
                update_showcase_scene,
                profile_active_effects.after(update_showcase_scene),
            ));
    }
}

fn setup_showcase(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera setup
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ShowcaseState {
            timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            current_scene: 0,
        },
    ));

    // Setup initial scene
    ShowcaseScene::NightScene.setup(&mut commands, &mut meshes, &mut materials);
}

fn log_effect_performance_impact(effect_name: &str, base_frame_time: f64, total_frame_time: f64) {
    let overhead_ms = (total_frame_time - base_frame_time) * 1000.0;
    let overhead_percent = (overhead_ms / (base_frame_time * 1000.0)) * 100.0;
    info!(
        "{} Performance Impact:\n  Base frame time: {:.2}ms\n  With effects: {:.2}ms\n  Overhead: {:.2}ms ({:.1}%)",
        effect_name,
        base_frame_time * 1000.0,
        total_frame_time * 1000.0,
        overhead_ms,
        overhead_percent
    );
}

#[derive(Resource)]
pub struct ShowcaseState {
    pub current_scene: ShowcaseScene,
    pub timer: Timer,
    pub active_effects: HashSet<String>,
    pub transition_progress: f32,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self {
            current_scene: ShowcaseScene::NightScene,
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
            active_effects: HashSet::new(),
            transition_progress: 0.0,
        }
    }
}

fn update_showcase_scene(
    time: Res<Time>,
    mut showcase_state: ResMut<ShowcaseState>,
    mut post_process_settings: ResMut<PostProcessSettings>,
    mut performance_stats: ResMut<PerformanceStats>,
) {
    showcase_state.timer.tick(time.delta());
    showcase_state.transition_progress += time.delta_seconds();

    if showcase_state.timer.just_finished() {
        // Reset performance stats for new scene
        performance_stats.base_frame_time = None;
        showcase_state.active_effects.clear();
        showcase_state.transition_progress = 0.0;

        // Cycle to next scene
        showcase_state.current_scene = match showcase_state.current_scene {
            ShowcaseScene::NightScene => ShowcaseScene::Underwater,
            ShowcaseScene::Underwater => ShowcaseScene::HeatDistortion,
            ShowcaseScene::HeatDistortion => ShowcaseScene::VintageFilm,
            ShowcaseScene::VintageFilm => ShowcaseScene::NightScene,
        };
    }

    // Update settings based on current scene and transition
    match showcase_state.current_scene {
        ShowcaseScene::NightScene => {
            let progress = (showcase_state.transition_progress * 2.0).min(1.0);
            post_process_settings.bloom_intensity = 1.5 * progress;
            post_process_settings.exposure = 1.2 * progress;
            post_process_settings.vignette_strength = 0.3 * progress;
            showcase_state.active_effects.insert("Bloom".to_string());
            showcase_state.active_effects.insert("Exposure".to_string());
            showcase_state.active_effects.insert("Vignette".to_string());
        }
        ShowcaseScene::Underwater => {
            let progress = (showcase_state.transition_progress * 2.0).min(1.0);
            post_process_settings.chromatic_aberration = 0.02 * progress;
            post_process_settings.color_tint = Vec4::new(0.0, 0.2, 0.4, 1.0);
            post_process_settings.blur_radius = 2.0 * progress;
            showcase_state.active_effects.insert("Chromatic Aberration".to_string());
            showcase_state.active_effects.insert("Color Tint".to_string());
            showcase_state.active_effects.insert("Blur".to_string());
        }
        ShowcaseScene::HeatDistortion => {
            let progress = (showcase_state.transition_progress * 2.0).min(1.0);
            post_process_settings.distortion_strength = 0.03 * progress;
            post_process_settings.distortion_scale = 30.0;
            post_process_settings.distortion_speed = 1.0;
            showcase_state.active_effects.insert("Heat Distortion".to_string());
        }
        ShowcaseScene::VintageFilm => {
            let progress = (showcase_state.transition_progress * 2.0).min(1.0);
            post_process_settings.grain_strength = 0.1 * progress;
            post_process_settings.sepia_strength = 0.8 * progress;
            post_process_settings.vignette_strength = 0.4 * progress;
            showcase_state.active_effects.insert("Film Grain".to_string());
            showcase_state.active_effects.insert("Sepia".to_string());
            showcase_state.active_effects.insert("Vignette".to_string());
        }
    }

    // Update performance stats active effects
    performance_stats.active_effects = showcase_state.active_effects.iter().cloned().collect();

    // Set base frame time if not set
    if performance_stats.base_frame_time.is_none() && showcase_state.transition_progress >= 0.5 {
        performance_stats.base_frame_time = performance_stats.current_frame_time;
    }
}

#[derive(Resource, Default)]
pub struct PerformanceStats {
    pub base_frame_time: Option<f64>,
    pub current_frame_time: Option<f64>,
    pub active_effects: Vec<String>,
}

pub struct PostProcessProfilingPlugin;

impl Plugin for PostProcessProfilingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerformanceStats>()
            .add_systems(Update, profile_frame_times);
    }
}

fn profile_frame_times(
    time: Res<Time>,
    mut stats: ResMut<PerformanceStats>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            stats.current_frame_time = Some(1.0 / value);
        }
    }
}

fn profile_active_effects(
    showcase_state: Res<ShowcaseState>,
    performance_stats: Res<PerformanceStats>,
) {
    // Log performance impact when scene changes or every 60 frames
    if showcase_state.timer.just_finished() {
        if let (Some(base_time), Some(current_time)) = (
            performance_stats.base_frame_time,
            performance_stats.current_frame_time,
        ) {
            info!("Active Effects: {:?}", performance_stats.active_effects);
            log_effect_performance_impact(
                &format!("{:?}", showcase_state.current_scene),
                base_time,
                current_time,
            );
        }
    }
} 