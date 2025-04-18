use bevy::prelude::*;
use std::f32::consts::PI;

/// Trait for particle emitter shapes
pub trait EmitterShape: Send + Sync + 'static {
    /// Generate a position for a new particle
    fn generate_position(&self, rng: &mut rand::rngs::ThreadRng) -> Vec3;
    /// Generate a direction for a new particle
    fn generate_direction(&self, position: Vec3, rng: &mut rand::rngs::ThreadRng) -> Vec3;
}

/// Point emitter that emits particles from a single point
#[derive(Component)]
pub struct PointEmitter {
    /// Direction of emission
    pub direction: Vec3,
    /// Spread angle in radians
    pub spread: f32,
}

impl Default for PointEmitter {
    fn default() -> Self {
        Self {
            direction: Vec3::Y,
            spread: PI / 4.0,
        }
    }
}

impl EmitterShape for PointEmitter {
    fn generate_position(&self, _rng: &mut rand::rngs::ThreadRng) -> Vec3 {
        Vec3::ZERO
    }

    fn generate_direction(&self, _position: Vec3, rng: &mut rand::rngs::ThreadRng) -> Vec3 {
        use rand::Rng;
        
        if self.spread <= 0.0 {
            return self.direction.normalize();
        }

        let up = self.direction.normalize();
        let right = if up.abs_diff_eq(Vec3::Y, 0.01) {
            Vec3::X
        } else {
            Vec3::Y.cross(up).normalize()
        };
        let forward = up.cross(right);

        // Generate random angles within spread
        let theta = rng.gen_range(-self.spread..=self.spread);
        let phi = rng.gen_range(0.0..2.0 * PI);

        // Convert spherical coordinates to direction vector
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        (right * sin_theta * cos_phi + forward * sin_theta * sin_phi + up * cos_theta).normalize()
    }
}

/// Sphere emitter that emits particles from a sphere surface or volume
#[derive(Component)]
pub struct SphereEmitter {
    /// Radius of the sphere
    pub radius: f32,
    /// Whether to emit from volume (true) or surface (false)
    pub emit_from_volume: bool,
    /// Whether to emit outward from center
    pub emit_outward: bool,
}

impl Default for SphereEmitter {
    fn default() -> Self {
        Self {
            radius: 1.0,
            emit_from_volume: false,
            emit_outward: true,
        }
    }
}

impl EmitterShape for SphereEmitter {
    fn generate_position(&self, rng: &mut rand::rngs::ThreadRng) -> Vec3 {
        use rand::Rng;

        if self.emit_from_volume {
            // Generate random point in sphere volume
            let theta = rng.gen_range(0.0..2.0 * PI);
            let phi = rng.gen_range(0.0..PI);
            let r = rng.gen_range(0.0..=self.radius);

            let sin_phi = phi.sin();
            Vec3::new(
                r * sin_phi * theta.cos(),
                r * sin_phi * theta.sin(),
                r * phi.cos(),
            )
        } else {
            // Generate random point on sphere surface
            let theta = rng.gen_range(0.0..2.0 * PI);
            let phi = rng.gen_range(0.0..PI);

            let sin_phi = phi.sin();
            Vec3::new(
                self.radius * sin_phi * theta.cos(),
                self.radius * sin_phi * theta.sin(),
                self.radius * phi.cos(),
            )
        }
    }

    fn generate_direction(&self, position: Vec3, rng: &mut rand::rngs::ThreadRng) -> Vec3 {
        if self.emit_outward {
            position.normalize()
        } else {
            use rand::Rng;
            
            let theta = rng.gen_range(0.0..2.0 * PI);
            let phi = rng.gen_range(0.0..PI);

            let sin_phi = phi.sin();
            Vec3::new(
                sin_phi * theta.cos(),
                sin_phi * theta.sin(),
                phi.cos(),
            ).normalize()
        }
    }
}

/// Box emitter that emits particles from a box surface or volume
#[derive(Component)]
pub struct BoxEmitter {
    /// Box dimensions (width, height, depth)
    pub size: Vec3,
    /// Whether to emit from volume (true) or surface (false)
    pub emit_from_volume: bool,
    /// Whether to emit outward from center
    pub emit_outward: bool,
}

impl Default for BoxEmitter {
    fn default() -> Self {
        Self {
            size: Vec3::ONE,
            emit_from_volume: false,
            emit_outward: true,
        }
    }
}

impl EmitterShape for BoxEmitter {
    fn generate_position(&self, rng: &mut rand::rngs::ThreadRng) -> Vec3 {
        use rand::Rng;

        if self.emit_from_volume {
            // Generate random point in box volume
            Vec3::new(
                rng.gen_range(-self.size.x..=self.size.x) * 0.5,
                rng.gen_range(-self.size.y..=self.size.y) * 0.5,
                rng.gen_range(-self.size.z..=self.size.z) * 0.5,
            )
        } else {
            // Generate random point on box surface
            let face = rng.gen_range(0..6);
            match face {
                0 => Vec3::new( self.size.x * 0.5, rng.gen_range(-0.5..=0.5) * self.size.y, rng.gen_range(-0.5..=0.5) * self.size.z), // Right
                1 => Vec3::new(-self.size.x * 0.5, rng.gen_range(-0.5..=0.5) * self.size.y, rng.gen_range(-0.5..=0.5) * self.size.z), // Left
                2 => Vec3::new(rng.gen_range(-0.5..=0.5) * self.size.x,  self.size.y * 0.5, rng.gen_range(-0.5..=0.5) * self.size.z), // Top
                3 => Vec3::new(rng.gen_range(-0.5..=0.5) * self.size.x, -self.size.y * 0.5, rng.gen_range(-0.5..=0.5) * self.size.z), // Bottom
                4 => Vec3::new(rng.gen_range(-0.5..=0.5) * self.size.x, rng.gen_range(-0.5..=0.5) * self.size.y,  self.size.z * 0.5), // Front
                _ => Vec3::new(rng.gen_range(-0.5..=0.5) * self.size.x, rng.gen_range(-0.5..=0.5) * self.size.y, -self.size.z * 0.5), // Back
            }
        }
    }

    fn generate_direction(&self, position: Vec3, rng: &mut rand::rngs::ThreadRng) -> Vec3 {
        if self.emit_outward {
            // Find closest face and emit outward from it
            let abs_pos = position.abs();
            let max_component = abs_pos.max_element();
            
            if abs_pos.x == max_component {
                Vec3::new(position.x.signum(), 0.0, 0.0)
            } else if abs_pos.y == max_component {
                Vec3::new(0.0, position.y.signum(), 0.0)
            } else {
                Vec3::new(0.0, 0.0, position.z.signum())
            }
        } else {
            use rand::Rng;
            
            let theta = rng.gen_range(0.0..2.0 * PI);
            let phi = rng.gen_range(0.0..PI);

            let sin_phi = phi.sin();
            Vec3::new(
                sin_phi * theta.cos(),
                sin_phi * theta.sin(),
                phi.cos(),
            ).normalize()
        }
    }
}

/// System to update emitter transforms
pub fn update_emitter_transforms(
    mut emitters: Query<(&mut Transform, Option<&PointEmitter>, Option<&SphereEmitter>, Option<&BoxEmitter>)>,
    time: Res<Time>,
) {
    for (mut transform, point_emitter, sphere_emitter, box_emitter) in emitters.iter_mut() {
        // Add any custom transform updates here (e.g., rotation, movement patterns)
        if point_emitter.is_some() {
            // Point emitter specific updates
        } else if sphere_emitter.is_some() {
            // Sphere emitter specific updates
            transform.rotate_y(time.delta_seconds() * 0.5);
        } else if box_emitter.is_some() {
            // Box emitter specific updates
            transform.rotate_x(time.delta_seconds() * 0.3);
        }
    }
} 