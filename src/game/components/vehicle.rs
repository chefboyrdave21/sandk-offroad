use bevy::prelude::*;
use bevy::math::Vec3;

/// Component for core vehicle properties and behavior
#[derive(Component, Debug)]
pub struct Vehicle {
    // Movement properties
    pub speed: f32,
    pub acceleration: f32,
    pub max_speed: f32,
    pub turn_speed: f32,
    
    // Ground detection
    pub ground_check_ray: f32,
    pub is_grounded: bool,
    
    // Physics properties
    pub mass: f32,
    pub center_of_mass_offset: Vec3,
}

impl Default for Vehicle {
    fn default() -> Self {
        Self {
            speed: 0.0,
            acceleration: 15.0,
            max_speed: 30.0,
            turn_speed: 2.0,
            ground_check_ray: 0.5,
            is_grounded: false,
            mass: 1500.0, // kg
            center_of_mass_offset: Vec3::new(0.0, -0.5, 0.0), // Slightly lowered center of mass
        }
    }
}

/// Component for vehicle suspension system
#[derive(Component, Debug)]
pub struct Suspension {
    // Spring properties
    pub spring_strength: f32,
    pub damping: f32,
    pub rest_length: f32,
    pub min_length: f32,
    pub max_length: f32,
    pub max_force: f32,
    
    // Wheel configuration
    pub wheel_positions: Vec<Vec3>,
    pub wheel_radius: f32,
    
    // Runtime state
    pub previous_lengths: Vec<f32>,
    pub wheel_entities: Vec<Entity>,
}

impl Default for Suspension {
    fn default() -> Self {
        Self {
            spring_strength: 50000.0,
            damping: 4000.0,
            rest_length: 0.5,
            min_length: 0.2,
            max_length: 0.8,
            max_force: 100000.0,
            wheel_positions: vec![
                Vec3::new(-0.8, 0.0, 1.0),  // Front left
                Vec3::new(0.8, 0.0, 1.0),   // Front right
                Vec3::new(-0.8, 0.0, -1.0), // Rear left
                Vec3::new(0.8, 0.0, -1.0),  // Rear right
            ],
            wheel_radius: 0.4,
            previous_lengths: vec![0.5; 4],
            wheel_entities: Vec::new(),
        }
    }
}

/// Component for individual wheel properties
#[derive(Component, Debug)]
pub struct Wheel {
    pub index: usize,
    pub steering_angle: f32,
    pub angular_velocity: f32,
    pub torque: f32,
    pub can_steer: bool,
    pub can_drive: bool,
}

impl Default for Wheel {
    fn default() -> Self {
        Self {
            index: 0,
            steering_angle: 0.0,
            angular_velocity: 0.0,
            torque: 0.0,
            can_steer: false,
            can_drive: true,
        }
    }
} 