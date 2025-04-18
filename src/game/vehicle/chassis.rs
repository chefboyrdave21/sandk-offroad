use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

/// Component for the vehicle chassis, representing the main body of the vehicle
#[derive(Component)]
pub struct Chassis {
    /// Attachment points for suspension components in local space
    pub suspension_points: [Vec3; 4],
    /// Current chassis angular velocity
    pub angular_velocity: Vec3,
    /// Current chassis linear velocity
    pub linear_velocity: Vec3,
    /// Total mass of the chassis (excluding wheels)
    pub mass: f32,
    /// Center of mass offset from origin
    pub center_of_mass_offset: Vec3,
    /// Inertia tensor for the chassis
    pub inertia_tensor: Mat3,
}

impl Default for Chassis {
    fn default() -> Self {
        Self {
            suspension_points: [
                Vec3::new(-0.75, -0.2, -1.19), // Front left
                Vec3::new(0.75, -0.2, -1.19),  // Front right
                Vec3::new(-0.75, -0.2, 1.19),  // Rear left
                Vec3::new(0.75, -0.2, 1.19),   // Rear right
            ],
            angular_velocity: Vec3::ZERO,
            linear_velocity: Vec3::ZERO,
            mass: 1500.0,
            center_of_mass_offset: Vec3::new(0.0, -0.2, 0.0),
            inertia_tensor: Mat3::IDENTITY * 1000.0, // Simplified inertia tensor
        }
    }
}

/// Bundle for spawning a chassis with all necessary components
#[derive(Bundle)]
pub struct ChassisBundle {
    pub chassis: Chassis,
    pub collider: Collider,
    pub mass_properties: ColliderMassProperties,
    pub friction: Friction,
    pub restitution: Restitution,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for ChassisBundle {
    fn default() -> Self {
        let chassis = Chassis::default();
        Self {
            chassis,
            collider: Collider::cuboid(1.0, 0.5, 2.0), // Basic box collider
            mass_properties: ColliderMassProperties::Mass(chassis.mass),
            friction: Friction::coefficient(0.5),
            restitution: Restitution::coefficient(0.2),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            global_transform: GlobalTransform::default(),
        }
    }
}

/// System to update chassis physics
pub fn update_chassis_physics(
    mut chassis_query: Query<(
        &mut Chassis,
        &mut Transform,
        &Velocity,
        &ExternalForce,
    )>,
    time: Res<Time>,
) {
    for (mut chassis, mut transform, velocity, external_force) in chassis_query.iter_mut() {
        // Update velocities from physics engine
        chassis.linear_velocity = velocity.linvel;
        chassis.angular_velocity = velocity.angvel;

        // Apply external forces through the physics engine
        // The actual force application is handled by bevy_rapier
    }
} 