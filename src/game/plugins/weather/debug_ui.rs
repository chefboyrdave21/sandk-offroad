impl WeatherDebugUi {
    pub fn draw(&self, ui: &mut egui::Ui, weather_effects: &WeatherEffects) {
        // ... existing code ...

        ui.collapsing("Performance", |ui| {
            ui.label("Timing Information:");
            
            if let Some(debug_state) = weather_effects.debug_state() {
                // Frame time statistics
                ui.label("Frame Times:");
                ui.label(format!("Current: {:.2}ms", debug_state.last_update_time));
                ui.label(format!("Min: {:.2}ms", debug_state.frame_time_min));
                ui.label(format!("Max: {:.2}ms", debug_state.frame_time_max));
                ui.label(format!("Variance: {:.3}ms²", debug_state.frame_time_variance));
                
                // CPU/GPU split
                ui.separator();
                ui.label("Processing Split:");
                ui.label(format!("CPU Time: {:.2}ms", debug_state.cpu_time_ms));
                ui.label(format!("GPU Time: {:.2}ms", debug_state.gpu_time_ms));
                
                // Memory usage
                ui.separator();
                ui.label("Memory Usage:");
                ui.label(format!("Current: {:.1} MB", debug_state.memory_usage_mb));
                
                if !debug_state.memory_history.is_empty() {
                    let memory_plot = egui::plot::Plot::new("memory_usage")
                        .height(100.0)
                        .show_axes([false, true]);
                    
                    memory_plot.show(ui, |plot_ui| {
                        let memory_line = egui::plot::Line::new(
                            debug_state.memory_history.iter()
                                .map(|(t, m)| [*t as f64, *m as f64])
                                .collect::<Vec<[f64; 2]>>()
                        ).name("Memory (MB)");
                        plot_ui.line(memory_line);
                    });
                }
            }

            ui.separator();
            ui.label("Per-Effect Type Statistics:");
            
            if let Some(debug_state) = weather_effects.debug_state() {
                egui::Grid::new("effect_type_stats").show(ui, |ui| {
                    ui.label("Effect Type");
                    ui.label("Count");
                    ui.label("Particles");
                    ui.label("Spawn Rate");
                    ui.label("Lifetime");
                    ui.label("Memory");
                    ui.label("CPU ms");
                    ui.label("GPU ms");
                    ui.end_row();
                    
                    for (effect_type, stats) in &debug_state.effect_type_stats {
                        ui.label(format!("{:?}", effect_type));
                        ui.label(format!("{}", stats.active_count));
                        ui.label(format!("{}", stats.total_particles));
                        ui.label(format!("{:.1}", stats.spawn_rate));
                        ui.label(format!("{:.1}s", stats.avg_lifetime));
                        ui.label(format!("{:.1}MB", stats.memory_usage / 1024.0 / 1024.0));
                        ui.label(format!("{:.2}", stats.cpu_time));
                        ui.label(format!("{:.2}", stats.gpu_time));
                        ui.end_row();
                    }
                });
            }

            ui.separator();
            ui.label("CPU/GPU Time History:");
            
            if let Some(debug_state) = weather_effects.debug_state() {
                if !debug_state.cpu_gpu_history.is_empty() {
                    let plot = egui::plot::Plot::new("cpu_gpu_split")
                        .height(100.0)
                        .show_axes([false, true]);
                    
                    plot.show(ui, |plot_ui| {
                        let cpu_line = egui::plot::Line::new(
                            debug_state.cpu_gpu_history.iter()
                                .map(|(t, cpu, _)| [*t as f64, *cpu as f64])
                                .collect::<Vec<[f64; 2]>>()
                        ).name("CPU");
                        
                        let gpu_line = egui::plot::Line::new(
                            debug_state.cpu_gpu_history.iter()
                                .map(|(t, _, gpu)| [*t as f64, *gpu as f64])
                                .collect::<Vec<[f64; 2]>>()
                        ).name("GPU");
                        
                        plot_ui.line(cpu_line);
                        plot_ui.line(gpu_line);
                    });
                }
            }

            ui.separator();
            ui.label("Active Effects:");
            
            if let Some(debug_state) = weather_effects.debug_state() {
                ui.label(format!("Total Effects: {}", debug_state.total_effects));
                ui.label(format!("Ground Effects: {}", debug_state.ground_effects));
                ui.label(format!("Particle Count: {}", debug_state.particle_count));
            }
        });
    }
} 