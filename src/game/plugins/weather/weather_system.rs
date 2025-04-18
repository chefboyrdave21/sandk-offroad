use std::time::Instant;
use bevy::prelude::*;
use crate::game::plugins::weather::debug_state::{WeatherDebugState, EffectTypeStats};

// ... existing code ...

impl WeatherSystem {
    pub fn update(&mut self, commands: &mut Commands, time: Res<Time>, diagnostics: Res<Diagnostics>, debug_state: Option<&mut WeatherDebugState>) {
        let update_start = Instant::now();
        
        // Record start times for different phases
        let spawn_start = Instant::now();
        self.spawn_effects(commands);
        let spawn_duration = spawn_start.elapsed().as_secs_f32() * 1000.0;
        
        let cpu_start = Instant::now();
        self.update_effects(time.delta_seconds());
        let cpu_duration = cpu_start.elapsed().as_secs_f32() * 1000.0;
        
        let gpu_start = Instant::now();
        self.render_effects();
        let gpu_duration = gpu_start.elapsed().as_secs_f32() * 1000.0;
        
        let update_duration = update_start.elapsed().as_secs_f32() * 1000.0;
        
        // Update debug state if available
        if let Some(debug_state) = debug_state {
            // Record timing information
            debug_state.record_timing(
                update_duration,
                spawn_duration,
                gpu_duration
            );
            
            // Record CPU/GPU split
            debug_state.record_cpu_gpu_split(
                cpu_duration,
                gpu_duration,
                time.elapsed_seconds()
            );
            
            // Record memory usage
            if let Some(memory) = diagnostics.get(SystemInformationDiagnosticsPlugin::MEMORY_USAGE) {
                if let Some(value) = memory.value() {
                    let memory_mb = value / 1024.0 / 1024.0;
                    debug_state.record_memory_usage(memory_mb, time.elapsed_seconds());
                }
            }
            
            // Update per-effect type statistics
            self.update_effect_type_statistics(debug_state);
            
            // Update general statistics
            let stats = self.get_effect_statistics();
            debug_state.update_statistics(
                stats.total_effects,
                stats.ground_effects,
                stats.particle_count
            );
        }
    }
    
    fn update_effect_type_statistics(&self, debug_state: &mut WeatherDebugState) {
        for effect_type in WeatherEffectType::iter() {
            let effects = self.active_effects.iter()
                .filter(|(t, _)| *t == effect_type)
                .collect::<Vec<_>>();
            
            if !effects.is_empty() {
                let mut stats = EffectTypeStats::default();
                stats.active_count = effects.len();
                
                for (_, entity) in &effects {
                    if let Some(effect) = entity.get_component::<ParticleEffect>() {
                        stats.total_particles += effect.particle_count();
                        stats.spawn_rate += effect.emitter.spawn_rate;
                        stats.avg_lifetime += effect.emitter.lifetime;
                        
                        // Estimate memory usage (rough approximation)
                        stats.memory_usage += effect.particle_count() as f32 * 64.0; // 64 bytes per particle
                    }
                }
                
                // Calculate averages
                if stats.active_count > 0 {
                    stats.spawn_rate /= stats.active_count as f32;
                    stats.avg_lifetime /= stats.active_count as f32;
                }
                
                // Record CPU/GPU time per effect type (estimated from profiler data)
                if let Some(profiler) = &self.profiler {
                    stats.cpu_time = profiler.get_cpu_time_for_type(effect_type);
                    stats.gpu_time = profiler.get_gpu_time_for_type(effect_type);
                }
                
                debug_state.update_effect_type_stats(effect_type, stats);
            }
        }
    }
    
    fn get_effect_statistics(&self) -> EffectStatistics {
        let mut stats = EffectStatistics {
            total_effects: 0,
            ground_effects: 0,
            particle_count: 0,
        };
        
        for effect in &self.active_effects {
            stats.total_effects += 1;
            if effect.is_ground_effect() {
                stats.ground_effects += 1;
            }
            stats.particle_count += effect.particle_count();
        }
        
        stats
    }
}

struct EffectStatistics {
    total_effects: usize,
    ground_effects: usize,
    particle_count: usize,
}