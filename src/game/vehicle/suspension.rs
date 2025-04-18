use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

/// Different types of suspension systems
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SuspensionType {
    Stock,              // Factory default suspension
    ShortArmLift,      // Short arm lift kit (2-3.5" lift)
    LongArmLift,       // Long arm lift kit (3.5-6" lift)
    CoiloverKit,       // Performance coilover suspension
    RockCrawler,       // Extreme off-road suspension
    PortalAxle,        // Portal axle lift (6-8" lift)
    AirSuspension,     // Adjustable air suspension
    CompetitionCrawler, // Competition rock crawler setup
    DesertRunner,      // High-speed desert racing setup
    ExpeditionLift,    // Overlanding/expedition setup
}

/// Suspension tuning profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspensionTuning {
    pub compression_damping: f32,  // Low-speed compression damping
    pub rebound_damping: f32,     // Low-speed rebound damping
    pub high_speed_compression: f32, // High-speed compression damping
    pub high_speed_rebound: f32,   // High-speed rebound damping
    pub preload: f32,             // Spring preload
}

impl Default for SuspensionTuning {
    fn default() -> Self {
        Self {
            compression_damping: 1.0,
            rebound_damping: 1.0,
            high_speed_compression: 1.0,
            high_speed_rebound: 1.0,
            preload: 0.0,
        }
    }
}

/// Lift kit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiftKitConfig {
    pub lift_height: f32,          // Total lift height in meters
    pub arm_length: f32,           // Control arm length
    pub arm_angle: f32,           // Control arm angle in radians
    pub track_width_increase: f32, // Increase in track width
    pub geometry_correction: f32,  // Correction factor for suspension geometry
}

impl Default for LiftKitConfig {
    fn default() -> Self {
        Self {
            lift_height: 0.0,
            arm_length: 0.5,
            arm_angle: 0.0,
            track_width_increase: 0.0,
            geometry_correction: 1.0,
        }
    }
}

/// Component for vehicle suspension
#[derive(Component)]
pub struct Suspension {
    /// Suspension type
    pub suspension_type: SuspensionType,
    /// Lift kit configuration if installed
    pub lift_kit: Option<LiftKitConfig>,
    /// Spring stiffness (N/m)
    pub spring_stiffness: f32,
    /// Damping coefficient (Ns/m)
    pub damping: f32,
    /// Rest length of the suspension (m)
    pub rest_length: f32,
    /// Current compression (m)
    pub compression: f32,
    /// Maximum compression (m)
    pub max_compression: f32,
    /// Maximum extension (m)
    pub max_extension: f32,
    /// Current suspension force (N)
    pub force: f32,
    /// Upper mounting point in chassis local space
    pub mount_point: Vec3,
    /// Lower mounting point (wheel connection) in chassis local space
    pub wheel_point: Vec3,
    /// Current velocity of compression/extension (m/s)
    pub velocity: f32,
    /// Current health percentage (0-100)
    pub health: f32,
    /// Maximum force before damage occurs
    pub damage_threshold: f32,
    /// Whether the suspension is currently broken
    pub is_broken: bool,
    /// Accumulated stress (for progressive damage)
    pub accumulated_stress: f32,
}

impl Default for Suspension {
    fn default() -> Self {
        Self {
            suspension_type: SuspensionType::Stock,
            lift_kit: None,
            spring_stiffness: 50000.0,
            damping: 5000.0,
            rest_length: 0.5,
            compression: 0.0,
            max_compression: 0.3,
            max_extension: 0.2,
            force: 0.0,
            mount_point: Vec3::ZERO,
            wheel_point: Vec3::new(0.0, -0.5, 0.0),
            velocity: 0.0,
            health: 100.0,
            damage_threshold: 60000.0,
            is_broken: false,
            accumulated_stress: 0.0,
        }
    }
}

impl Suspension {
    pub fn with_type(suspension_type: SuspensionType) -> Self {
        let mut suspension = Self::default();
        suspension.configure_type(suspension_type);
        suspension
    }

    pub fn configure_type(&mut self, suspension_type: SuspensionType) {
        self.suspension_type = suspension_type;
        match suspension_type {
            SuspensionType::Stock => {
                self.spring_stiffness = 50000.0;
                self.damping = 5000.0;
                self.max_compression = 0.3;
                self.max_extension = 0.2;
                self.damage_threshold = 60000.0;
            }
            SuspensionType::ShortArmLift => {
                self.spring_stiffness = 55000.0;
                self.damping = 5500.0;
                self.max_compression = 0.35;
                self.max_extension = 0.25;
                self.damage_threshold = 65000.0;
                self.lift_kit = Some(LiftKitConfig {
                    lift_height: 0.075, // 3" lift
                    arm_length: 0.55,
                    arm_angle: 0.1,
                    track_width_increase: 0.025,
                    geometry_correction: 1.1,
                });
            }
            SuspensionType::LongArmLift => {
                self.spring_stiffness = 60000.0;
                self.damping = 6000.0;
                self.max_compression = 0.4;
                self.max_extension = 0.3;
                self.damage_threshold = 70000.0;
                self.lift_kit = Some(LiftKitConfig {
                    lift_height: 0.125, // 5" lift
                    arm_length: 0.65,
                    arm_angle: 0.15,
                    track_width_increase: 0.05,
                    geometry_correction: 1.2,
                });
            }
            SuspensionType::CoiloverKit => {
                self.spring_stiffness = 70000.0;
                self.damping = 7000.0;
                self.max_compression = 0.45;
                self.max_extension = 0.35;
                self.damage_threshold = 80000.0;
                self.lift_kit = Some(LiftKitConfig {
                    lift_height: 0.1,
                    arm_length: 0.6,
                    arm_angle: 0.12,
                    track_width_increase: 0.04,
                    geometry_correction: 1.3,
                });
            }
            SuspensionType::RockCrawler => {
                self.spring_stiffness = 80000.0;
                self.damping = 8000.0;
                self.max_compression = 0.5;
                self.max_extension = 0.4;
                self.damage_threshold = 90000.0;
                self.lift_kit = Some(LiftKitConfig {
                    lift_height: 0.15,
                    arm_length: 0.7,
                    arm_angle: 0.2,
                    track_width_increase: 0.075,
                    geometry_correction: 1.4,
                });
            }
            SuspensionType::PortalAxle => {
                self.spring_stiffness = 85000.0;
                self.damping = 8500.0;
                self.max_compression = 0.45;
                self.max_extension = 0.35;
                self.damage_threshold = 95000.0;
                self.lift_kit = Some(LiftKitConfig {
                    lift_height: 0.175, // 7" lift
                    arm_length: 0.75,
                    arm_angle: 0.25,
                    track_width_increase: 0.1,
                    geometry_correction: 1.5,
                });
            }
            SuspensionType::AirSuspension => {
                self.spring_stiffness = 65000.0;
                self.damping = 6500.0;
                self.max_compression = 0.4;
                self.max_extension = 0.35;
                self.damage_threshold = 75000.0;
                self.lift_kit = Some(LiftKitConfig {
                    lift_height: 0.1, // Adjustable 0-4"
                    arm_length: 0.6,
                    arm_angle: 0.15,
                    track_width_increase: 0.03,
                    geometry_correction: 1.25,
                });
            }
            SuspensionType::CompetitionCrawler => {
                self.spring_stiffness = 90000.0;
                self.damping = 9000.0;
                self.max_compression = 0.55;
                self.max_extension = 0.45;
                self.damage_threshold = 100000.0;
                self.lift_kit = Some(LiftKitConfig {
                    lift_height: 0.2, // 8" lift
                    arm_length: 0.8,
                    arm_angle: 0.3,
                    track_width_increase: 0.15,
                    geometry_correction: 1.6,
                });
            }
            SuspensionType::DesertRunner => {
                self.spring_stiffness = 75000.0;
                self.damping = 8000.0;
                self.max_compression = 0.5;
                self.max_extension = 0.4;
                self.damage_threshold = 85000.0;
                self.lift_kit = Some(LiftKitConfig {
                    lift_height: 0.125, // 5" lift
                    arm_length: 0.7,
                    arm_angle: 0.2,
                    track_width_increase: 0.08,
                    geometry_correction: 1.35,
                });
            }
            SuspensionType::ExpeditionLift => {
                self.spring_stiffness = 70000.0;
                self.damping = 7500.0;
                self.max_compression = 0.45;
                self.max_extension = 0.35;
                self.damage_threshold = 80000.0;
                self.lift_kit = Some(LiftKitConfig {
                    lift_height: 0.15, // 6" lift
                    arm_length: 0.65,
                    arm_angle: 0.18,
                    track_width_increase: 0.06,
                    geometry_correction: 1.3,
                });
            }
            _ => {} // Handle existing types
        }
    }
}

/// Bundle for spawning a suspension with all necessary components
#[derive(Bundle)]
pub struct SuspensionBundle {
    pub suspension: Suspension,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for SuspensionBundle {
    fn default() -> Self {
        Self {
            suspension: Suspension::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}

/// System to update suspension physics and handle damage
pub fn update_suspension_physics(
    mut suspension_query: Query<(&mut Suspension, &GlobalTransform)>,
    wheel_query: Query<(&Transform, &Wheel)>,
    chassis_query: Query<(&Transform, &Chassis)>,
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for (mut suspension, suspension_global) in suspension_query.iter_mut() {
        // Skip physics update if suspension is broken
        if suspension.is_broken {
            continue;
        }

        // Get chassis transform
        if let Ok((chassis_transform, chassis)) = chassis_query.get_single() {
            // Transform suspension mount points to world space
            let world_mount = chassis_transform.transform_point(suspension.mount_point);
            let world_wheel = chassis_transform.transform_point(suspension.wheel_point);

            // Calculate current length
            let current_length = (world_mount - world_wheel).length();
            
            // Update compression
            let prev_compression = suspension.compression;
            suspension.compression = suspension.rest_length - current_length;
            suspension.compression = suspension.compression.clamp(
                -suspension.max_extension,
                suspension.max_compression
            );

            // Calculate velocity
            suspension.velocity = (suspension.compression - prev_compression) / dt;

            // Calculate spring force
            let spring_force = suspension.spring_stiffness * suspension.compression;
            
            // Calculate damping force
            let damping_force = suspension.damping * suspension.velocity;
            
            // Total force
            suspension.force = spring_force + damping_force;

            // Handle damage
            update_suspension_damage(&mut suspension, dt);

            // Apply geometry correction from lift kit if present
            if let Some(lift_kit) = &suspension.lift_kit {
                suspension.force *= lift_kit.geometry_correction;
            }

            // Ensure force is within physical limits (adjusted by health)
            let max_force = if suspension.health < 100.0 {
                suspension.damage_threshold * (suspension.health / 100.0)
            } else {
                suspension.damage_threshold
            };
            suspension.force = suspension.force.clamp(-max_force, max_force);
        }
    }
}

/// Updates suspension damage based on forces and stress
fn update_suspension_damage(suspension: &mut Suspension, dt: f32) {
    // Calculate stress from force
    let force_stress = (suspension.force.abs() / suspension.damage_threshold).powf(2.0);
    
    // Add compression stress
    let compression_stress = if suspension.compression >= suspension.max_compression * 0.9 
        || suspension.compression <= -suspension.max_extension * 0.9 {
        0.5 // High stress when near limits
    } else {
        0.0
    };

    // Add velocity stress
    let velocity_stress = (suspension.velocity.abs() / 10.0).min(1.0) * 0.3;

    // Calculate total stress
    let total_stress = force_stress + compression_stress + velocity_stress;
    
    // Accumulate stress
    suspension.accumulated_stress += total_stress * dt;
    
    // Natural stress recovery when under low stress
    if total_stress < 0.1 {
        suspension.accumulated_stress = (suspension.accumulated_stress - dt * 0.5).max(0.0);
    }

    // Calculate damage based on accumulated stress
    if suspension.accumulated_stress > 1.0 {
        let damage = suspension.accumulated_stress * 5.0 * dt;
        suspension.health = (suspension.health - damage).max(0.0);
        suspension.accumulated_stress = 0.0;
    }

    // Check for complete failure
    if suspension.health <= 0.0 {
        suspension.is_broken = true;
        suspension.health = 0.0;
    }
}

/// System to apply suspension forces to chassis and wheels, accounting for damage
pub fn apply_suspension_forces(
    suspension_query: Query<(&Suspension, &GlobalTransform)>,
    mut chassis_query: Query<(&mut ExternalForce, &Transform), With<Chassis>>,
    mut wheel_query: Query<(&mut ExternalForce, &Transform), With<Wheel>>,
) {
    if let Ok((mut chassis_force, chassis_transform)) = chassis_query.get_single_mut() {
        for (suspension, suspension_global) in suspension_query.iter() {
            // Skip if suspension is broken
            if suspension.is_broken {
                continue;
            }

            // Calculate force direction
            let force_dir = (suspension.mount_point - suspension.wheel_point).normalize();
            let force_vec = force_dir * suspension.force;

            // Apply force to chassis (modified by health)
            let health_factor = suspension.health / 100.0;
            chassis_force.force += force_vec * health_factor;
            
            // Calculate torque on chassis
            let r = suspension.mount_point - chassis_transform.translation;
            chassis_force.torque += r.cross(force_vec) * health_factor;

            // Apply opposite force to wheel
            if let Ok((mut wheel_force, _)) = wheel_query.get_single_mut() {
                wheel_force.force -= force_vec * health_factor;
            }
        }
    }
}

/// Resource for controlling suspension debug visualization
#[derive(Resource)]
pub struct SuspensionDebugConfig {
    pub show_forces: bool,
    pub show_compression: bool,
    pub show_mount_points: bool,
    pub show_health: bool,
    pub show_stress: bool,
    pub show_travel_limits: bool,
    pub show_suspension_type: bool,
    pub show_geometry: bool,
    pub show_velocity: bool,
    pub show_metrics: bool,
    pub force_scale: f32,
}

impl Default for SuspensionDebugConfig {
    fn default() -> Self {
        Self {
            show_forces: true,
            show_compression: true,
            show_mount_points: true,
            show_health: true,
            show_stress: true,
            show_travel_limits: true,
            show_suspension_type: true,
            show_geometry: true,
            show_velocity: true,
            show_metrics: true,
            force_scale: 0.0001,
        }
    }
}

/// System to draw debug visualizations for suspension
pub fn draw_suspension_debug(
    mut gizmos: Gizmos,
    suspension_query: Query<(&Suspension, &GlobalTransform)>,
    chassis_query: Query<&Transform, With<Chassis>>,
    debug_config: Res<SuspensionDebugConfig>,
) {
    if let Ok(chassis_transform) = chassis_query.get_single() {
        for (suspension, suspension_global) in suspension_query.iter() {
            let world_mount = chassis_transform.transform_point(suspension.mount_point);
            let world_wheel = chassis_transform.transform_point(suspension.wheel_point);

            // Draw mount points
            if debug_config.show_mount_points {
                // Upper mount (red)
                gizmos.sphere(world_mount, Quat::IDENTITY, 0.05, Color::RED);
                // Lower mount (blue)
                gizmos.sphere(world_wheel, Quat::IDENTITY, 0.05, Color::BLUE);
                // Connection line
                gizmos.line(world_mount, world_wheel, Color::WHITE);
            }

            // Draw force vector
            if debug_config.show_forces {
                let force_dir = (suspension.mount_point - suspension.wheel_point).normalize();
                let force_vec = force_dir * suspension.force * debug_config.force_scale;
                let force_color = if suspension.force > 0.0 {
                    Color::GREEN
                } else {
                    Color::RED
                };
                gizmos.ray(
                    world_wheel,
                    force_vec,
                    force_color,
                );
            }

            // Draw compression indicator
            if debug_config.show_compression {
                let compression_percent = (suspension.compression / suspension.max_compression).abs();
                let compression_color = Color::rgb(
                    compression_percent,
                    1.0 - compression_percent,
                    0.0,
                );
                let indicator_pos = world_wheel + Vec3::new(0.1, 0.0, 0.0);
                gizmos.line(
                    indicator_pos,
                    indicator_pos + Vec3::Y * compression_percent * 0.2,
                    compression_color,
                );
            }

            // Draw health indicator
            if debug_config.show_health {
                let health_percent = suspension.health / 100.0;
                let health_color = Color::rgb(
                    1.0 - health_percent,
                    health_percent,
                    0.0,
                );
                let health_pos = world_mount + Vec3::new(0.1, 0.0, 0.0);
                gizmos.line(
                    health_pos,
                    health_pos + Vec3::Y * health_percent * 0.2,
                    health_color,
                );
            }

            // Draw stress indicator
            if debug_config.show_stress {
                let stress_percent = (suspension.accumulated_stress).min(1.0);
                let stress_color = Color::rgb(
                    stress_percent,
                    0.0,
                    1.0 - stress_percent,
                );
                let stress_pos = world_mount + Vec3::new(-0.1, 0.0, 0.0);
                gizmos.line(
                    stress_pos,
                    stress_pos + Vec3::Y * stress_percent * 0.2,
                    stress_color,
                );
            }

            // Draw travel limits
            if debug_config.show_travel_limits {
                let up_dir = (world_mount - world_wheel).normalize();
                let max_comp_point = world_wheel + up_dir * suspension.max_compression;
                let max_ext_point = world_wheel - up_dir * suspension.max_extension;
                
                // Max compression limit (red)
                gizmos.line(
                    world_wheel + Vec3::new(0.05, 0.0, 0.0),
                    max_comp_point + Vec3::new(0.05, 0.0, 0.0),
                    Color::RED,
                );
                
                // Max extension limit (blue)
                gizmos.line(
                    world_wheel - Vec3::new(0.05, 0.0, 0.0),
                    max_ext_point - Vec3::new(0.05, 0.0, 0.0),
                    Color::BLUE,
                );
            }

            // Draw suspension geometry
            if debug_config.show_geometry && suspension.lift_kit.is_some() {
                let lift_kit = suspension.lift_kit.as_ref().unwrap();
                let arm_dir = Vec3::new(
                    lift_kit.arm_angle.cos(),
                    lift_kit.arm_angle.sin(),
                    0.0,
                ).normalize();
                
                // Draw control arm
                gizmos.line(
                    world_mount,
                    world_mount + arm_dir * lift_kit.arm_length,
                    Color::YELLOW,
                );
                
                // Draw track width increase
                let track_point = world_wheel + Vec3::new(lift_kit.track_width_increase, 0.0, 0.0);
                gizmos.line(world_wheel, track_point, Color::GREEN);
            }

            // Draw velocity vector
            if debug_config.show_velocity {
                let velocity_dir = if suspension.velocity != 0.0 {
                    (world_mount - world_wheel).normalize()
                } else {
                    Vec3::ZERO
                };
                let velocity_vec = velocity_dir * suspension.velocity * 0.1;
                let velocity_color = if suspension.velocity > 0.0 {
                    Color::CYAN
                } else {
                    Color::ORANGE
                };
                gizmos.ray(
                    world_wheel,
                    velocity_vec,
                    velocity_color,
                );
            }

            // Draw metrics text
            if debug_config.show_metrics {
                let metrics_pos = world_mount + Vec3::new(0.2, 0.0, 0.0);
                let type_text = format!("Type: {:?}", suspension.suspension_type);
                let health_text = format!("Health: {:.1}%", suspension.health);
                let force_text = format!("Force: {:.1}N", suspension.force);
                let stress_text = format!("Stress: {:.2}", suspension.accumulated_stress);
                
                gizmos.text_3d(
                    metrics_pos,
                    Color::WHITE,
                    type_text,
                );
                gizmos.text_3d(
                    metrics_pos + Vec3::new(0.0, -0.05, 0.0),
                    Color::WHITE,
                    health_text,
                );
                gizmos.text_3d(
                    metrics_pos + Vec3::new(0.0, -0.1, 0.0),
                    Color::WHITE,
                    force_text,
                );
                gizmos.text_3d(
                    metrics_pos + Vec3::new(0.0, -0.15, 0.0),
                    Color::WHITE,
                    stress_text,
                );
            }

            // Draw broken indicator
            if suspension.is_broken {
                gizmos.sphere(
                    (world_mount + world_wheel) * 0.5,
                    Quat::IDENTITY,
                    0.1,
                    Color::RED,
                );
            }
        }
    }
}

// Add this to your plugin setup:
// app.init_resource::<SuspensionDebugConfig>()
//    .add_systems(Update, draw_suspension_debug); 