use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

/// Component for vehicle wheels
#[derive(Component)]
pub struct Wheel {
    /// Wheel position (FL: 0, FR: 1, RL: 2, RR: 3)
    pub position: usize,
    /// Wheel radius in meters
    pub radius: f32,
    /// Wheel width in meters
    pub width: f32,
    /// Current angular velocity (rad/s)
    pub angular_velocity: f32,
    /// Current steering angle (rad)
    pub steering_angle: f32,
    /// Current drive torque (Nm)
    pub drive_torque: f32,
    /// Current brake torque (Nm)
    pub brake_torque: f32,
    /// Mass of the wheel (kg)
    pub mass: f32,
    /// Moment of inertia around rotation axis
    pub inertia: f32,
    /// Whether the wheel is in contact with the ground
    pub ground_contact: bool,
    /// Normal force from ground contact
    pub normal_force: f32,
    /// Lateral slip angle (rad)
    pub slip_angle: f32,
    /// Longitudinal slip ratio
    pub slip_ratio: f32,
}

impl Default for Wheel {
    fn default() -> Self {
        Self {
            position: 0,
            radius: 0.4,
            width: 0.25,
            angular_velocity: 0.0,
            steering_angle: 0.0,
            drive_torque: 0.0,
            brake_torque: 0.0,
            mass: 20.0,
            inertia: 2.5, // Approximated as mr²/2 for a solid cylinder
            ground_contact: false,
            normal_force: 0.0,
            slip_angle: 0.0,
            slip_ratio: 0.0,
        }
    }
}

/// Bundle for spawning a wheel with all necessary components
#[derive(Bundle)]
pub struct WheelBundle {
    pub wheel: Wheel,
    pub collider: Collider,
    pub mass_properties: ColliderMassProperties,
    pub friction: Friction,
    pub restitution: Restitution,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for WheelBundle {
    fn default() -> Self {
        let wheel = Wheel::default();
        Self {
            wheel,
            collider: Collider::cylinder(wheel.width / 2.0, wheel.radius),
            mass_properties: ColliderMassProperties::Mass(wheel.mass),
            friction: Friction::coefficient(1.0),
            restitution: Restitution::coefficient(0.3),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}

/// System to update wheel physics
pub fn update_wheel_physics(
    mut wheel_query: Query<(
        &mut Wheel,
        &mut Transform,
        &GlobalTransform,
        &Velocity,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for (mut wheel, mut transform, global_transform, velocity) in wheel_query.iter_mut() {
        // Update wheel rotation based on angular velocity
        let rotation_angle = wheel.angular_velocity * dt;
        transform.rotate_local_x(rotation_angle);

        // Update steering
        if wheel.position <= 1 { // Front wheels
            transform.rotation = Quat::from_rotation_y(wheel.steering_angle);
        }

        // Calculate slip values if in ground contact
        if wheel.ground_contact {
            // Get wheel's forward and right vectors in world space
            let forward = global_transform.forward();
            let right = global_transform.right();

            // Project velocity onto wheel's local axes
            let local_vel = Vec3::new(
                velocity.linvel.dot(right),
                0.0,
                velocity.linvel.dot(forward),
            );

            // Calculate slip angle (lateral)
            wheel.slip_angle = (local_vel.x / local_vel.z.abs().max(0.1)).atan();

            // Calculate slip ratio (longitudinal)
            let wheel_speed = wheel.angular_velocity * wheel.radius;
            let ground_speed = local_vel.z;
            wheel.slip_ratio = if ground_speed.abs() > 0.1 {
                (wheel_speed - ground_speed) / ground_speed.abs()
            } else {
                0.0
            };
        } else {
            wheel.slip_angle = 0.0;
            wheel.slip_ratio = 0.0;
            wheel.normal_force = 0.0;
        }

        // Apply drive and brake torques
        let total_torque = wheel.drive_torque - wheel.brake_torque.copysign(wheel.angular_velocity);
        wheel.angular_velocity += (total_torque / wheel.inertia) * dt;

        // Apply rolling resistance
        let rolling_resistance = -0.02 * wheel.normal_force * wheel.angular_velocity.signum();
        wheel.angular_velocity += (rolling_resistance * wheel.radius / wheel.inertia) * dt;
    }
} 