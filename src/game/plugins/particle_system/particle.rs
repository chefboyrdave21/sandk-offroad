use bevy::prelude::*;
use super::gradient::{EaseFunction, GradientPreset};

/// Color channels for particle effects
#[derive(Clone, Debug)]
pub struct ParticleColors {
    /// Base color gradient
    pub albedo: ParticleColorGradient,
    /// Emission color gradient for glow effects
    pub emission: ParticleColorGradient,
    /// Emission strength over lifetime (0.0 to 1.0)
    pub emission_strength: f32,
    /// Easing function for color interpolation
    pub ease_function: EaseFunction,
}

impl Default for ParticleColors {
    fn default() -> Self {
        Self {
            albedo: ParticleColorGradient::default(),
            emission: ParticleColorGradient::default(),
            emission_strength: 0.0,
            ease_function: EaseFunction::Linear,
        }
    }
}

/// Color keyframe for gradient interpolation
#[derive(Clone, Debug)]
pub struct ColorKeyframe {
    /// Time point in particle lifetime (0.0 to 1.0)
    pub time: f32,
    /// Color at this keyframe
    pub color: Color,
}

/// Color gradient for particle lifetime
#[derive(Clone, Debug)]
pub struct ParticleColorGradient {
    /// Keyframes defining the gradient
    pub keyframes: Vec<ColorKeyframe>,
}

impl Default for ParticleColorGradient {
    fn default() -> Self {
        Self {
            keyframes: vec![
                ColorKeyframe { time: 0.0, color: Color::WHITE },
                ColorKeyframe { time: 1.0, color: Color::WHITE },
            ],
        }
    }
}

impl ParticleColorGradient {
    /// Create a new gradient from keyframes
    pub fn new(keyframes: Vec<ColorKeyframe>) -> Self {
        let mut gradient = Self { keyframes };
        gradient.sort_keyframes();
        gradient
    }

    /// Create a gradient from a preset
    pub fn from_preset(preset: GradientPreset) -> Self {
        preset.create_gradient()
    }

    /// Sort keyframes by time
    fn sort_keyframes(&mut self) {
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// Sample the gradient at a specific time (0.0 to 1.0) with easing
    pub fn sample(&self, time: f32, ease_fn: EaseFunction) -> Color {
        let time = ease_fn.apply(time.clamp(0.0, 1.0));
        
        // Handle edge cases
        if time <= self.keyframes[0].time {
            return self.keyframes[0].color;
        }
        if time >= self.keyframes.last().unwrap().time {
            return self.keyframes.last().unwrap().color;
        }

        // Find the keyframes to interpolate between
        let mut prev_keyframe = &self.keyframes[0];
        for keyframe in self.keyframes.iter().skip(1) {
            if keyframe.time > time {
                let t = (time - prev_keyframe.time) / (keyframe.time - prev_keyframe.time);
                return lerp_color(&prev_keyframe.color, &keyframe.color, t);
            }
            prev_keyframe = keyframe;
        }

        self.keyframes.last().unwrap().color
    }
}

/// Linear interpolation between two colors
fn lerp_color(start: &Color, end: &Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    match (start, end) {
        (Color::Rgba { red: r1, green: g1, blue: b1, alpha: a1 }, 
         Color::Rgba { red: r2, green: g2, blue: b2, alpha: a2 }) => {
            Color::Rgba {
                red: r1 + (r2 - r1) * t,
                green: g1 + (g2 - g1) * t,
                blue: b1 + (b2 - b1) * t,
                alpha: a1 + (a2 - a1) * t,
            }
        }
        _ => *start, // Fallback for non-RGBA colors
    }
}

/// Parameters for particle simulation
#[derive(Clone, Debug)]
pub struct SimulationParams {
    /// Particle colors and gradients
    pub colors: ParticleColors,
    // ... rest of existing fields ...
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            colors: ParticleColors::default(),
            // ... rest of existing fields ...
        }
    }
}

/// A particle system component
#[derive(Component)]
pub struct ParticleSystem {
    // ... existing code ...
    /// Simulation parameters
    pub params: SimulationParams,
}

/// System to update particle parameters
pub fn update_particle_params(
    time: Res<Time>,
    mut query: Query<&mut ParticleSystem>,
) {
    for mut particle_system in query.iter_mut() {
        // Update particle colors based on lifetime
        for i in 0..particle_system.particle_count {
            let lifetime = particle_system.get_particle_lifetime(i);
            if lifetime > 0.0 {
                let life_fraction = lifetime / particle_system.params.lifetime;
                let time = 1.0 - life_fraction; // Reverse time for consistent behavior
                
                // Update base color
                let albedo = particle_system.params.colors.albedo.sample(
                    time,
                    particle_system.params.colors.ease_function
                );
                particle_system.set_particle_color(i, albedo);
                
                // Update emission color
                let emission = particle_system.params.colors.emission.sample(
                    time,
                    particle_system.params.colors.ease_function
                );
                particle_system.set_particle_emission(i, emission, particle_system.params.colors.emission_strength);
            }
        }
        
        // ... rest of update logic ...
    }
}