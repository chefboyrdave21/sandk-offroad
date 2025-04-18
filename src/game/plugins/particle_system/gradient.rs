use bevy::prelude::*;
use std::f32::consts::PI;

/// Control point for custom Bezier curves
#[derive(Clone, Copy, Debug)]
pub struct ControlPoint {
    pub x: f32,
    pub y: f32,
}

impl ControlPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Easing function type for non-linear interpolation
#[derive(Clone, Debug)]
pub enum EaseFunction {
    /// Linear interpolation (no easing)
    Linear,
    /// Smooth step (cubic) interpolation
    SmoothStep,
    /// Quadratic ease in
    QuadIn,
    /// Quadratic ease out
    QuadOut,
    /// Sine wave based easing
    Sine,
    /// Elastic bounce effect
    Elastic,
    /// Custom curve defined by control points
    Custom(Vec<ControlPoint>),
    /// Cubic Bezier curve with two control points
    CubicBezier(ControlPoint, ControlPoint),
}

impl EaseFunction {
    /// Create a custom easing function from control points
    pub fn from_points(points: Vec<ControlPoint>) -> Self {
        Self::Custom(points)
    }

    /// Create a cubic Bezier easing function
    pub fn cubic_bezier(p1: ControlPoint, p2: ControlPoint) -> Self {
        Self::CubicBezier(p1, p2)
    }

    /// Apply the easing function to a value
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::SmoothStep => t * t * (3.0 - 2.0 * t),
            Self::QuadIn => t * t,
            Self::QuadOut => t * (2.0 - t),
            Self::Sine => (1.0 - (t * PI).cos()) * 0.5,
            Self::Elastic => {
                let p = 0.3;
                (-(2.0_f32.powf(-10.0 * t) * ((t - p / 4.0) * (2.0 * PI) / p).sin())).exp2()
            }
            Self::Custom(points) => {
                if points.is_empty() {
                    return t;
                }
                
                // Find the segment containing t
                let mut prev = points[0];
                for point in points.iter().skip(1) {
                    if point.x > t {
                        // Linear interpolation within segment
                        let segment_t = (t - prev.x) / (point.x - prev.x);
                        return prev.y + segment_t * (point.y - prev.y);
                    }
                    prev = *point;
                }
                
                // If t is beyond the last point
                points.last().unwrap().y
            }
            Self::CubicBezier(p1, p2) => {
                // Cubic Bezier curve calculation
                let cx = 3.0 * p1.x;
                let bx = 3.0 * (p2.x - p1.x) - cx;
                let ax = 1.0 - cx - bx;
                
                let cy = 3.0 * p1.y;
                let by = 3.0 * (p2.y - p1.y) - cy;
                let ay = 1.0 - cy - by;
                
                // Find t value using Newton's method
                let mut t_guess = t;
                for _ in 0..8 {
                    let x = ((ax * t_guess + bx) * t_guess + cx) * t_guess;
                    let dx = (3.0 * ax * t_guess + 2.0 * bx) * t_guess + cx;
                    if dx.abs() < 1e-6 {
                        break;
                    }
                    t_guess -= (x - t) / dx;
                }
                
                ((ay * t_guess + by) * t_guess + cy) * t_guess
            }
        }
    }
}

/// Preset color gradients for common effects
#[derive(Clone, Debug)]
pub enum GradientPreset {
    /// Fire effect (yellow -> orange -> red -> dark red)
    Fire,
    /// Smoke effect (white -> gray -> dark gray -> transparent)
    Smoke,
    /// Sparkle effect (bright white -> yellow -> transparent)
    Sparkle,
    /// Magic effect (purple -> pink -> blue -> transparent)
    Magic,
    /// Nature effect (green -> yellow-green -> transparent)
    Nature,
    /// Water effect (light blue -> blue -> dark blue -> transparent)
    Water,
    /// Lightning effect (electric effect)
    Lightning,
    /// Rainbow effect (rainbow colors)
    Rainbow,
    /// Acid effect (toxic/acid effect)
    Acid,
    /// Energy effect (energy/plasma effect)
    Energy,
    /// Dark effect (dark/void effect)
    Dark,
}

impl GradientPreset {
    /// Create a ColorGradient from this preset
    pub fn create_gradient(&self) -> super::particle::ParticleColorGradient {
        use super::particle::{ColorKeyframe, ParticleColorGradient};
        
        let (keyframes, ease_fn) = match self {
            Self::Fire => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(1.0, 0.9, 0.3, 1.0) },
                ColorKeyframe { time: 0.2, color: Color::rgba(1.0, 0.7, 0.0, 1.0) },
                ColorKeyframe { time: 0.6, color: Color::rgba(1.0, 0.3, 0.0, 0.8) },
                ColorKeyframe { time: 1.0, color: Color::rgba(0.5, 0.0, 0.0, 0.0) },
            ], EaseFunction::QuadOut),
            
            Self::Smoke => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(1.0, 1.0, 1.0, 0.8) },
                ColorKeyframe { time: 0.3, color: Color::rgba(0.8, 0.8, 0.8, 0.6) },
                ColorKeyframe { time: 0.7, color: Color::rgba(0.5, 0.5, 0.5, 0.3) },
                ColorKeyframe { time: 1.0, color: Color::rgba(0.3, 0.3, 0.3, 0.0) },
            ], EaseFunction::SmoothStep),
            
            Self::Sparkle => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(1.0, 1.0, 1.0, 1.0) },
                ColorKeyframe { time: 0.4, color: Color::rgba(1.0, 1.0, 0.6, 0.8) },
                ColorKeyframe { time: 0.7, color: Color::rgba(1.0, 0.8, 0.4, 0.4) },
                ColorKeyframe { time: 1.0, color: Color::rgba(1.0, 0.6, 0.2, 0.0) },
            ], EaseFunction::Elastic),
            
            Self::Magic => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(0.8, 0.2, 1.0, 1.0) },
                ColorKeyframe { time: 0.3, color: Color::rgba(1.0, 0.4, 0.8, 0.8) },
                ColorKeyframe { time: 0.7, color: Color::rgba(0.4, 0.4, 1.0, 0.4) },
                ColorKeyframe { time: 1.0, color: Color::rgba(0.2, 0.2, 0.8, 0.0) },
            ], EaseFunction::cubic_bezier(ControlPoint::new(0.4, 0.0), ControlPoint::new(0.6, 1.0))),
            
            Self::Nature => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(0.4, 0.8, 0.2, 1.0) },
                ColorKeyframe { time: 0.4, color: Color::rgba(0.6, 0.8, 0.2, 0.7) },
                ColorKeyframe { time: 1.0, color: Color::rgba(0.3, 0.5, 0.1, 0.0) },
            ], EaseFunction::Sine),
            
            Self::Water => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(0.6, 0.8, 1.0, 0.8) },
                ColorKeyframe { time: 0.4, color: Color::rgba(0.2, 0.5, 1.0, 0.6) },
                ColorKeyframe { time: 0.8, color: Color::rgba(0.1, 0.2, 0.8, 0.3) },
                ColorKeyframe { time: 1.0, color: Color::rgba(0.0, 0.1, 0.5, 0.0) },
            ], EaseFunction::SmoothStep),

            Self::Lightning => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(1.0, 1.0, 1.0, 1.0) },
                ColorKeyframe { time: 0.1, color: Color::rgba(0.8, 0.9, 1.0, 0.9) },
                ColorKeyframe { time: 0.2, color: Color::rgba(0.6, 0.8, 1.0, 0.7) },
                ColorKeyframe { time: 1.0, color: Color::rgba(0.2, 0.4, 0.8, 0.0) },
            ], EaseFunction::QuadIn),

            Self::Rainbow => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(1.0, 0.0, 0.0, 1.0) },
                ColorKeyframe { time: 0.2, color: Color::rgba(1.0, 1.0, 0.0, 0.8) },
                ColorKeyframe { time: 0.4, color: Color::rgba(0.0, 1.0, 0.0, 0.6) },
                ColorKeyframe { time: 0.6, color: Color::rgba(0.0, 1.0, 1.0, 0.4) },
                ColorKeyframe { time: 0.8, color: Color::rgba(0.0, 0.0, 1.0, 0.2) },
                ColorKeyframe { time: 1.0, color: Color::rgba(1.0, 0.0, 1.0, 0.0) },
            ], EaseFunction::from_points(vec![
                ControlPoint::new(0.0, 0.0),
                ControlPoint::new(0.2, 0.8),
                ControlPoint::new(0.8, 0.2),
                ControlPoint::new(1.0, 0.0),
            ])),

            Self::Acid => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(0.8, 1.0, 0.0, 1.0) },
                ColorKeyframe { time: 0.3, color: Color::rgba(0.6, 0.8, 0.0, 0.8) },
                ColorKeyframe { time: 0.7, color: Color::rgba(0.4, 0.6, 0.0, 0.4) },
                ColorKeyframe { time: 1.0, color: Color::rgba(0.2, 0.3, 0.0, 0.0) },
            ], EaseFunction::QuadOut),

            Self::Energy => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(0.0, 1.0, 1.0, 1.0) },
                ColorKeyframe { time: 0.4, color: Color::rgba(0.4, 0.8, 1.0, 0.8) },
                ColorKeyframe { time: 0.7, color: Color::rgba(0.8, 0.6, 1.0, 0.4) },
                ColorKeyframe { time: 1.0, color: Color::rgba(1.0, 0.4, 1.0, 0.0) },
            ], EaseFunction::Elastic),

            Self::Dark => (vec![
                ColorKeyframe { time: 0.0, color: Color::rgba(0.1, 0.0, 0.2, 1.0) },
                ColorKeyframe { time: 0.4, color: Color::rgba(0.2, 0.0, 0.3, 0.7) },
                ColorKeyframe { time: 0.8, color: Color::rgba(0.3, 0.0, 0.4, 0.3) },
                ColorKeyframe { time: 1.0, color: Color::rgba(0.0, 0.0, 0.0, 0.0) },
            ], EaseFunction::QuadIn),
        };
        
        let mut gradient = ParticleColorGradient::new(keyframes);
        gradient.set_ease_function(ease_fn);
        gradient
    }
} 