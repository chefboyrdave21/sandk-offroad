use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_physics)
            .add_systems(Update, (
                update_vehicle_physics,
                handle_terrain_collisions,
            ));
    }
}

#[derive(Component)]
pub struct Vehicle {
    pub max_speed: f32,
    pub acceleration: f32,
    pub turn_rate: f32,
    pub current_speed: f32,
}

#[derive(Component)]
pub struct Terrain {
    pub height_map: Vec<f32>,
    pub width: usize,
    pub depth: usize,
}

fn setup_physics(mut commands: Commands) {
    // Configure physics world
    commands.insert_resource(RapierConfiguration {
        gravity: Vec3::new(0.0, -9.81, 0.0),
        ..default()
    });
}

fn update_vehicle_physics(
    mut query: Query<(&mut Vehicle, &mut Transform, &mut Velocity)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut vehicle, mut transform, mut velocity) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        
        if keyboard.pressed(KeyCode::W) {
            direction.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::S) {
            direction.z += 1.0;
        }
        if keyboard.pressed(KeyCode::A) {
            transform.rotate_y(time.delta_seconds() * vehicle.turn_rate);
        }
        if keyboard.pressed(KeyCode::D) {
            transform.rotate_y(-time.delta_seconds() * vehicle.turn_rate);
        }

        if direction != Vec3::ZERO {
            vehicle.current_speed = (vehicle.current_speed + vehicle.acceleration * time.delta_seconds())
                .min(vehicle.max_speed);
        } else {
            vehicle.current_speed = (vehicle.current_speed - vehicle.acceleration * time.delta_seconds())
                .max(0.0);
        }

        let forward = transform.forward();
        velocity.linvel = forward * vehicle.current_speed;
    }
}

fn handle_terrain_collisions(
    mut vehicle_query: Query<(&mut Transform, &mut Velocity), With<Vehicle>>,
    terrain_query: Query<&Terrain>,
) {
    // TODO: Implement terrain collision detection and response
} 