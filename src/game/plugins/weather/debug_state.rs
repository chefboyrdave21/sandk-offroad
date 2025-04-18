#[derive(Default)]
pub struct WeatherDebugState {
    // ... existing code ...
    
    // Timing information
    pub last_update_time: f32,
    pub last_spawn_time: f32,
    pub last_render_time: f32,
    
    // Effect statistics
    pub total_effects: usize,
    pub ground_effects: usize,
    pub particle_count: usize,
    
    // Memory tracking
    pub memory_usage_mb: f32,
    pub memory_history: Vec<(f32, f32)>, // (timestamp, memory in MB)
    
    // Per-effect type statistics
    pub effect_type_stats: std::collections::HashMap<WeatherEffectType, EffectTypeStats>,
    
    // Frame time statistics
    pub frame_times: Vec<f32>,
    pub frame_time_variance: f32,
    pub frame_time_min: f32,
    pub frame_time_max: f32,
    
    // CPU/GPU time split
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
    pub cpu_gpu_history: Vec<(f32, f32, f32)>, // (timestamp, cpu_ms, gpu_ms)
    
    // Transition history
    pub transition_history: Vec<WeatherTransitionInfo>,
}

#[derive(Default)]
pub struct EffectTypeStats {
    pub active_count: usize,
    pub total_particles: usize,
    pub spawn_rate: f32,
    pub avg_lifetime: f32,
    pub memory_usage: f32,
    pub cpu_time: f32,
    pub gpu_time: f32,
}

pub struct WeatherTransitionInfo {
    pub from: String,
    pub to: String,
    pub duration: f32,
}

impl WeatherDebugState {
    pub fn record_timing(&mut self, update_time: f32, spawn_time: f32, render_time: f32) {
        self.last_update_time = update_time;
        self.last_spawn_time = spawn_time;
        self.last_render_time = render_time;
        
        // Update frame time statistics
        self.frame_times.push(update_time);
        if self.frame_times.len() > 100 {
            self.frame_times.remove(0);
        }
        
        // Calculate frame time variance and min/max
        if !self.frame_times.is_empty() {
            let mean = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            self.frame_time_variance = self.frame_times.iter()
                .map(|&t| (t - mean).powi(2))
                .sum::<f32>() / self.frame_times.len() as f32;
            self.frame_time_min = *self.frame_times.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            self.frame_time_max = *self.frame_times.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        }
    }
    
    pub fn record_memory_usage(&mut self, memory_mb: f32, timestamp: f32) {
        self.memory_usage_mb = memory_mb;
        self.memory_history.push((timestamp, memory_mb));
        
        // Keep last 100 samples
        if self.memory_history.len() > 100 {
            self.memory_history.remove(0);
        }
    }
    
    pub fn record_cpu_gpu_split(&mut self, cpu_time: f32, gpu_time: f32, timestamp: f32) {
        self.cpu_time_ms = cpu_time;
        self.gpu_time_ms = gpu_time;
        self.cpu_gpu_history.push((timestamp, cpu_time, gpu_time));
        
        // Keep last 100 samples
        if self.cpu_gpu_history.len() > 100 {
            self.cpu_gpu_history.remove(0);
        }
    }
    
    pub fn update_effect_type_stats(&mut self, effect_type: WeatherEffectType, stats: EffectTypeStats) {
        self.effect_type_stats.insert(effect_type, stats);
    }
    
    pub fn record_transition(&mut self, from: String, to: String, duration: f32) {
        self.transition_history.push(WeatherTransitionInfo {
            from,
            to,
            duration,
        });
        
        // Keep only the last 10 transitions
        if self.transition_history.len() > 10 {
            self.transition_history.remove(0);
        }
    }
    
    pub fn update_statistics(&mut self, total: usize, ground: usize, particles: usize) {
        self.total_effects = total;
        self.ground_effects = ground;
        self.particle_count = particles;
    }
} 