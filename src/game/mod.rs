use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use crate::terrain::TerrainPlugin;
use bevy_rapier3d::prelude::*;
use bevy::app::PluginGroupBuilder;

pub mod states;
pub mod ui;
#[cfg(test)]
pub mod tests;

mod plugins;
mod systems;
mod components;
mod resources;

mod state;
mod debug;
mod input;
mod vehicle;
mod physics;
mod camera;

pub use plugins::*;
pub use systems::*;
pub use components::*;
pub use resources::*;

pub use state::GameState;
pub use debug::DebugInfo;
pub use input::InputState;
pub use vehicle::VehicleConfig;

// Constants
pub mod constants {
    // Vehicle dimensions (in meters)
    pub const JEEP_LENGTH: f32 = 4.17;
    pub const JEEP_WIDTH: f32 = 1.74;
    pub const JEEP_HEIGHT: f32 = 1.75;
    pub const WHEEL_RADIUS: f32 = 0.4;
    pub const WHEELBASE: f32 = 2.38;
    pub const TRACK_WIDTH: f32 = 1.5;
    pub const MAX_STEERING_ANGLE: f32 = 0.35;
}

use constants::*;

// Re-export commonly used components
pub use components::{MainCamera, CameraFollow, Vehicle, Player, Suspension};

/// Main plugin group for the game that sets up all core systems and resources
pub struct GamePluginGroup;

impl PluginGroup for GamePluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(GamePlugin)
            .add(state::StatePlugin)
            .add(debug::DebugPlugin) 
            .add(input::InputPlugin)
            .add(vehicle::VehiclePlugin)
            .add(physics::PhysicsPlugin)
            .add(camera::CameraPlugin)
            .add(ui::UiPlugin)
    }
}

/// Represents the different states the game can be in
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    Paused,
}

/// Main plugin for the SandK Offroad game
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Core game systems
            .add_plugins(plugins::CorePlugins)
            // Game state management
            .init_resource::<resources::GameState>()
            // Add core systems
            .add_systems(Startup, systems::setup)
            .add_systems(Update, (
                systems::handle_input,
                systems::update_game_state,
            ));
    }
}

/// Initializes the game with default configuration
pub fn init_game() -> App {
    let mut app = App::new();
    
    app.add_plugins(GamePluginGroup);
    
    #[cfg(debug_assertions)]
    {
        use bevy_inspector_egui::quick::WorldInspectorPlugin;
        app.add_plugins(WorldInspectorPlugin::new());
    }
    
    app
}

#[derive(Component)]
struct JeepPart;

#[derive(Component)]
struct Wheel;

#[derive(Component)]
pub struct Vehicle {
    pub speed: f32,
    pub acceleration: f32,
    pub max_speed: f32,
    pub turn_speed: f32,
    pub ground_check_ray: Ray,
    pub is_grounded: bool,
}

impl Default for Vehicle {
    fn default() -> Self {
        Self {
            speed: 0.0,
            acceleration: 15.0,
            max_speed: 25.0,
            turn_speed: 2.0,
            ground_check_ray: Ray {
                origin: Vec3::ZERO,
                direction: -Vec3::Y,
            },
            is_grounded: false,
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub health: f32,
}

#[derive(Component)]
pub struct Suspension {
    pub spring_strength: f32,
    pub damping: f32,
    pub rest_length: f32,
    pub min_length: f32,
    pub max_length: f32,
    pub max_force: f32,
    pub wheel_positions: Vec<Vec3>,
    pub wheel_radius: f32,
    pub prev_lengths: Vec<f32>,
}

impl Default for Suspension {
    fn default() -> Self {
        Self {
            spring_strength: 50000.0,
            damping: 2000.0,
            rest_length: 0.5,
            min_length: 0.2,
            max_length: 0.8,
            max_force: 50000.0,
            wheel_positions: vec![
                Vec3::new(-1.0, 0.0, -1.5),  // Front left
                Vec3::new(1.0, 0.0, -1.5),   // Front right
                Vec3::new(-1.0, 0.0, 1.5),   // Rear left
                Vec3::new(1.0, 0.0, 1.5),    // Rear right
            ],
            wheel_radius: 0.4,
            prev_lengths: vec![0.5; 4],
        }
    }
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    info!("Setting up game...");

    // Main light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 10.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Create camera
    let camera_entity = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        MainCamera::default(),
    )).id();

    // Create vehicle using a simple box for testing
    let vehicle_entity = commands.spawn((
        Name::new("Jeep TJ"),
        Player { health: 100.0 },
        Vehicle {
            speed: 0.0,
            max_speed: 30.0,
            acceleration: 15.0,
            turn_speed: 2.0,
            is_grounded: false,
            ground_check_ray: Ray {
                origin: Vec3::ZERO,
                direction: -Vec3::Y,
            },
        },
        RigidBody::Dynamic,
        Collider::cuboid(JEEP_WIDTH/2.0, JEEP_HEIGHT/2.0, JEEP_LENGTH/2.0),
        ColliderMassProperties::Mass(1000.0),
        Friction::coefficient(0.5),
        Restitution::coefficient(0.2),
        Damping {
            linear_damping: 0.2,
            angular_damping: 0.2,
        },
        Suspension::default(),
        Transform::from_xyz(0.0, 5.0, 0.0),
        GlobalTransform::default(),
    )).id();

    // Update camera to follow vehicle
    if let Some(mut camera) = commands.get_entity(camera_entity) {
        camera.insert(CameraFollow {
            target: Some(vehicle_entity),
            offset: Vec3::new(0.0, 5.0, 10.0),
            smoothness: 0.1,
        });
    }

    info!("Setup complete!");
}

// Helper function to create the Jeep body mesh
fn create_jeep_body_mesh() -> Mesh {
    // Create a more complex mesh that resembles a Jeep TJ
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    
    // Main body box
    let body_height = JEEP_HEIGHT - 0.3; // Slightly lower for wheels
    let vertices = vec![
        // Bottom face
        [-JEEP_WIDTH/2.0, 0.0, -JEEP_LENGTH/2.0],  // 0
        [JEEP_WIDTH/2.0, 0.0, -JEEP_LENGTH/2.0],   // 1
        [JEEP_WIDTH/2.0, 0.0, JEEP_LENGTH/2.0],    // 2
        [-JEEP_WIDTH/2.0, 0.0, JEEP_LENGTH/2.0],   // 3
        // Top face
        [-JEEP_WIDTH/2.0, body_height, -JEEP_LENGTH/2.0],  // 4
        [JEEP_WIDTH/2.0, body_height, -JEEP_LENGTH/2.0],   // 5
        [JEEP_WIDTH/2.0, body_height, JEEP_LENGTH/2.0],    // 6
        [-JEEP_WIDTH/2.0, body_height, JEEP_LENGTH/2.0],   // 7
    ];

    let indices = vec![
        // Bottom face (facing down)
        0, 2, 1, 0, 3, 2,
        // Top face (facing up)
        4, 5, 6, 4, 6, 7,
        // Front face (facing forward)
        3, 7, 6, 3, 6, 2,
        // Back face (facing backward)
        0, 1, 5, 0, 5, 4,
        // Right face (facing right)
        1, 2, 6, 1, 6, 5,
        // Left face (facing left)
        0, 4, 7, 0, 7, 3,
    ];

    let normals = vec![
        // Bottom vertices
        [0.0, -1.0, 0.0],  // 0
        [0.0, -1.0, 0.0],  // 1
        [0.0, -1.0, 0.0],  // 2
        [0.0, -1.0, 0.0],  // 3
        // Top vertices
        [0.0, 1.0, 0.0],   // 4
        [0.0, 1.0, 0.0],   // 5
        [0.0, 1.0, 0.0],   // 6
        [0.0, 1.0, 0.0],   // 7
    ];

    let uvs = vec![
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],  // Bottom
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],  // Top
    ];

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    mesh
}

// Helper function to create wheel mesh
fn create_wheel_mesh() -> Mesh {
    // Create a more detailed wheel with proper orientation
    shape::Cylinder {
        radius: WHEEL_RADIUS,
        height: 0.25, // Slightly wider for better visibility
        resolution: 32, // More segments for smoother look
        segments: 32,  // More segments for smoother look
    }.into()
}

fn update_vehicle_movement(
    mut query: Query<(
        &mut ExternalForce,
        &mut Vehicle,
        &Transform,
        &Velocity,
        &Suspension,
    )>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut external_force, mut vehicle, transform, velocity, suspension) in query.iter_mut() {
        let forward = transform.forward();
        let right = transform.right();
        
        // Base drive forces
        let mut drive_force = Vec3::ZERO;
        let mut turn_force = Vec3::ZERO;
        
        // Forward/backward movement
        if keyboard.pressed(KeyCode::W) {
            drive_force += forward * 40000.0; // Increased forward force
        }
        if keyboard.pressed(KeyCode::S) {
            drive_force += -forward * 30000.0; // Increased reverse force
        }
        
        // Turning
        if keyboard.pressed(KeyCode::A) {
            turn_force += -right * 12000.0; // Increased turning force
        }
        if keyboard.pressed(KeyCode::D) {
            turn_force += right * 12000.0;
        }
        
        // Calculate suspension force
        let mut suspension_force = Vec3::ZERO;
        let ray_origin = transform.translation;
        let ray_direction = -Vec3::Y;
        
        // Cast ray for ground detection
        let max_toi = suspension.max_length;
        let solid = true;
        
        // Update vehicle grounded state based on suspension
        vehicle.is_grounded = false; // Will be set to true if ground is detected
        
        // Apply suspension forces
        let ground_height = ray_origin.y; // This should be replaced with actual raycast result
        let suspension_length = (suspension.rest_length - ground_height).clamp(
            suspension.min_length,
            suspension.max_length
        );
        
        if suspension_length < suspension.max_length {
            vehicle.is_grounded = true;
            let spring_force = suspension.spring_strength * (suspension.rest_length - suspension_length);
            let damping_force = -suspension.damping * velocity.linvel.y;
            suspension_force = Vec3::new(0.0, spring_force + damping_force, 0.0);
        }
        
        // Combine all forces
        let total_force = drive_force + turn_force + suspension_force;
        
        // Apply artificial drag when moving
        let drag_force = -velocity.linvel * 0.5; // Reduced drag coefficient
        
        // Set the final external force
        external_force.force = total_force + drag_force;
        
        // Update vehicle speed for reference
        vehicle.speed = velocity.linvel.length();
    }
}

fn update_camera_follow(
    mut query: Query<(&mut Transform, &CameraFollow)>,
    target_query: Query<&Transform>,
    time: Res<Time>,
) {
    for (mut camera_transform, follow) in query.iter_mut() {
        if let Some(target) = follow.target {
            if let Ok(target_transform) = target_query.get(target) {
                let target_pos = target_transform.translation;
                let desired_pos = target_pos + follow.offset;
                camera_transform.translation = camera_transform.translation.lerp(
                    desired_pos,
                    follow.smoothness * time.delta_seconds() * 60.0,
                );
                camera_transform.look_at(target_pos, Vec3::Y);
            }
        }
    }
}

// Add a debug system to monitor vehicle state
fn debug_vehicle_state(
    query: Query<(&Transform, &Vehicle, &Velocity, &ExternalForce, &Name), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((transform, vehicle, velocity, ext_force, name)) = query.get_single() {
        if keyboard.any_pressed([KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D]) {
            info!(
                "Vehicle Debug Info:\n\
                Name: {:?}\n\
                Position: {:?}\n\
                Rotation: {:?}\n\
                Speed: {:.2}\n\
                Velocity: {:?}\n\
                External Force: {:?}\n\
                Grounded: {}\n\
                Keys pressed: {:?}\n\
                Delta time: {:.4}",
                name,
                transform.translation,
                transform.rotation,
                vehicle.speed,
                velocity.linvel,
                ext_force.force,
                vehicle.is_grounded,
                keyboard.get_pressed().collect::<Vec<_>>(),
                time.delta_seconds()
            );
        }
    } else {
        warn!("Debug: No vehicle entity found with Player component!");
    }
}

// Add suspension system
fn update_suspension(
    mut vehicle_query: Query<(&mut Suspension, &Transform, &mut ExternalForce), With<Vehicle>>,
    rapier_context: Res<RapierContext>,
    mut gizmos: Gizmos,
) {
    for (mut suspension, transform, mut ext_force) in vehicle_query.iter_mut() {
        let mut total_force = Vec3::ZERO;
        
        for (i, wheel_pos) in suspension.wheel_positions.iter().enumerate() {
            // Transform wheel position to world space
            let world_wheel_pos = transform.transform_point(*wheel_pos);
            let ray_dir = -transform.up();
            
            // Cast ray for suspension
            let ray = Ray::new(world_wheel_pos, ray_dir);
            if let Some((_, hit_point)) = rapier_context.cast_ray(
                ray.origin,
                ray.direction,
                2.0,
                true,
                QueryFilter::default(),
            ) {
                // Calculate suspension force
                let hit_distance = hit_point * ray.direction.length();
                let compression = (suspension.rest_length - hit_distance).max(0.0);
                suspension.compression[i] = compression;
                
                // Spring force
                let spring_force = compression * suspension.spring_strength;
                
                // Damping force (using current compression)
                let damping_force = suspension.compression[i] * suspension.damping;
                
                // Total force for this wheel
                let force = (spring_force + damping_force).min(suspension.max_force);
                let force_vec = transform.up() * force;
                
                suspension.wheel_forces[i] = force_vec;
                total_force += force_vec;
                
                // Visualize suspension rays
                gizmos.ray(
                    world_wheel_pos,
                    ray.direction * 2.0,
                    if compression > 0.0 { Color::RED } else { Color::GREEN },
                );
            } else {
                suspension.compression[i] = 0.0;
                suspension.wheel_forces[i] = Vec3::ZERO;
                
                // Visualize suspension rays (no hit)
                gizmos.ray(
                    world_wheel_pos,
                    ray.direction * 2.0,
                    Color::BLUE,
                );
            }
        }
        
        // Apply total suspension force
        ext_force.force += total_force;
    }
}

// System implementations
fn setup_physics(mut commands: Commands) {
    // Initialize physics world and constraints
}

fn setup_camera(mut commands: Commands) {
    // Setup main game camera
}

fn update_game_state(
    mut state: ResMut<GameState>,
    time: Res<Time>,
) {
    // Update core game state
}

fn update_physics(
    mut physics_world: ResMut<PhysicsWorld>,
    time: Res<Time>,
) {
    // Update physics simulation
}

fn update_vehicle(
    mut vehicles: Query<(&mut Transform, &mut Vehicle)>,
    input: Res<InputState>,
    time: Res<Time>,
) {
    // Update vehicle physics and controls
}

fn update_camera(
    mut camera: Query<&mut Transform, With<Camera>>,
    target: Query<&Transform, With<Vehicle>>,
) {
    // Update camera position and orientation
}

fn update_ui(
    mut ui_state: ResMut<UiState>,
    state: Res<GameState>,
) {
    // Update UI elements based on game state
} 