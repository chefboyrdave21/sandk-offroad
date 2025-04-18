use bevy::prelude::*;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use std::collections::VecDeque;
use std::collections::HashMap;

/// Component for tracking performance metrics
#[derive(Component)]
pub struct PerformanceMetrics {
    frame_times: VecDeque<f64>,
    max_samples: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(100),
            max_samples: 100,
        }
    }
}

/// Resource for storing performance statistics
#[derive(Resource, Default)]
pub struct PerformanceStats {
    pub current_frame_time: Option<f64>,
    pub base_frame_time: Option<f64>,
    pub active_effects: Vec<String>,
    pub effect_overheads: HashMap<String, f64>,
    pub frame_history: Vec<f64>,
    pub history_size: usize,
}

impl PerformanceStats {
    pub fn new(history_size: usize) -> Self {
        Self {
            history_size,
            frame_history: Vec::with_capacity(history_size),
            effect_overheads: HashMap::new(),
            ..Default::default()
        }
    }

    pub fn update_frame_time(&mut self, frame_time: f64) {
        self.current_frame_time = Some(frame_time);
        
        // Update frame history
        if self.frame_history.len() >= self.history_size {
            self.frame_history.remove(0);
        }
        self.frame_history.push(frame_time);
    }

    pub fn average_frame_time(&self) -> Option<f64> {
        if self.frame_history.is_empty() {
            return None;
        }
        Some(self.frame_history.iter().sum::<f64>() / self.frame_history.len() as f64)
    }

    pub fn calculate_effect_overhead(&mut self) {
        if let (Some(base), Some(current)) = (self.base_frame_time, self.current_frame_time) {
            let total_overhead = (current - base).max(0.0);
            let effect_count = self.active_effects.len().max(1) as f64;
            
            // Distribute overhead among active effects
            let overhead_per_effect = total_overhead / effect_count;
            for effect in &self.active_effects {
                self.effect_overheads.insert(effect.clone(), overhead_per_effect);
            }
        }
    }
}

/// Plugin for performance profiling
pub struct PostProcessProfilingPlugin;

impl Plugin for PostProcessProfilingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
           .init_resource::<PerformanceStats>()
           .add_systems(Startup, setup_profiling)
           .add_systems(Update, (
               update_metrics,
               update_performance_display,
               profile_frame_times,
           ));
    }
}

fn setup_profiling(mut commands: Commands) {
    // Spawn performance metrics entity
    commands.spawn((
        PerformanceMetrics::default(),
        // Add UI components for displaying metrics
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
            ..default()
        },
    ));
}

fn update_metrics(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    mut metrics: Query<&mut PerformanceMetrics>,
    mut stats: ResMut<PerformanceStats>,
    post_process_settings: Res<PostProcessSettings>,
) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(frame_time) = fps.average() {
            for mut metrics in metrics.iter_mut() {
                // Update frame time history
                metrics.frame_times.push_back(frame_time);
                if metrics.frame_times.len() > metrics.max_samples {
                    metrics.frame_times.pop_front();
                }

                // Calculate statistics
                let frame_times: Vec<f64> = metrics.frame_times.iter().copied().collect();
                stats.average_frame_time = frame_times.iter().sum::<f64>() / frame_times.len() as f64;
                stats.min_frame_time = *frame_times.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);
                stats.max_frame_time = *frame_times.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);

                // Calculate variance
                let squared_diff_sum: f64 = frame_times
                    .iter()
                    .map(|&t| (t - stats.average_frame_time).powi(2))
                    .sum();
                stats.frame_time_variance = squared_diff_sum / frame_times.len() as f64;

                // Track enabled effects
                stats.active_effects.clear();
                if post_process_settings.bloom_intensity > 0.0 {
                    stats.active_effects.push("Bloom".to_string());
                }
                if post_process_settings.chromatic_aberration > 0.0 {
                    stats.active_effects.push("Chromatic Aberration".to_string());
                }
                if post_process_settings.vignette_strength > 0.0 {
                    stats.active_effects.push("Vignette".to_string());
                }
                if post_process_settings.film_grain > 0.0 {
                    stats.active_effects.push("Film Grain".to_string());
                }
                if post_process_settings.lens_distortion > 0.0 {
                    stats.active_effects.push("Lens Distortion".to_string());
                }
            }
        }
    }
}

fn update_performance_display(
    mut commands: Commands,
    stats: Res<PerformanceStats>,
    metrics_query: Query<Entity, With<PerformanceMetrics>>,
) {
    for entity in metrics_query.iter() {
        commands.entity(entity).despawn_descendants();
        
        // Create text sections for each metric
        commands.entity(entity).with_children(|parent| {
            parent.spawn(TextBundle::from_sections([
                TextSection::new(
                    format!(
                        "Frame Time: {:.2}ms (min: {:.2}ms, max: {:.2}ms)\n\
                         Variance: {:.4}ms²\n\
                         Active Effects:\n{}",
                        stats.average_frame_time,
                        stats.min_frame_time,
                        stats.max_frame_time,
                        stats.frame_time_variance,
                        stats.active_effects.join("\n- ")
                    ),
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
            ]));
        });
    }
}

fn profile_frame_times(
    diagnostics: Res<Diagnostics>,
    mut perf_stats: ResMut<PerformanceStats>,
) {
    if let Some(frame_time) = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diagnostic| diagnostic.smoothed())
    {
        perf_stats.update_frame_time(frame_time);
        perf_stats.calculate_effect_overhead();

        // Log performance stats every 60 frames
        if perf_stats.frame_history.len() % 60 == 0 {
            if let Some(avg_frame_time) = perf_stats.average_frame_time() {
                info!(
                    "Performance Stats:\n\
                    Average Frame Time: {:.2}ms\n\
                    Active Effects: {:?}\n\
                    Effect Overheads: {:#?}",
                    avg_frame_time,
                    perf_stats.active_effects,
                    perf_stats.effect_overheads
                );
            }
        }
    }
}

/// Helper function to log performance impact of specific effects
pub fn log_effect_performance_impact(
    effect_name: &str,
    before_frame_time: f64,
    after_frame_time: f64,
) {
    let impact = after_frame_time - before_frame_time;
    info!(
        "Performance impact of {}: {:.2}ms ({:.1}% overhead)",
        effect_name,
        impact,
        (impact / before_frame_time) * 100.0
    );
} 