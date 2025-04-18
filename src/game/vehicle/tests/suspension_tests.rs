use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::game::vehicle::*;
use approx::assert_relative_eq;

#[test]
fn test_suspension_types() {
    // Test different suspension configurations
    let stock = Suspension::with_type(SuspensionType::Stock);
    let short_arm = Suspension::with_type(SuspensionType::ShortArmLift);
    let long_arm = Suspension::with_type(SuspensionType::LongArmLift);
    
    // Verify stock configuration
    assert_eq!(stock.suspension_type, SuspensionType::Stock);
    assert!(stock.lift_kit.is_none());
    assert_eq!(stock.spring_stiffness, 50000.0);
    
    // Verify short arm lift configuration
    assert_eq!(short_arm.suspension_type, SuspensionType::ShortArmLift);
    assert!(short_arm.lift_kit.is_some());
    let lift_kit = short_arm.lift_kit.unwrap();
    assert_eq!(lift_kit.lift_height, 0.075); // 3" lift
    
    // Verify long arm lift configuration
    assert_eq!(long_arm.suspension_type, SuspensionType::LongArmLift);
    assert!(long_arm.lift_kit.is_some());
    let lift_kit = long_arm.lift_kit.unwrap();
    assert_eq!(lift_kit.lift_height, 0.125); // 5" lift
}

#[test]
fn test_suspension_compression() {
    let mut suspension = Suspension::default();
    
    // Test normal compression
    suspension.compression = 0.1;
    assert!(suspension.compression <= suspension.max_compression);
    assert!(suspension.compression >= -suspension.max_extension);
    
    // Test maximum compression
    suspension.compression = 1.0;
    assert_eq!(suspension.compression.min(suspension.max_compression), suspension.max_compression);
    
    // Test maximum extension
    suspension.compression = -1.0;
    assert_eq!(suspension.compression.max(-suspension.max_extension), -suspension.max_extension);
}

#[test]
fn test_suspension_forces() {
    let mut suspension = Suspension::default();
    
    // Test spring force
    suspension.compression = 0.1;
    let spring_force = suspension.spring_stiffness * suspension.compression;
    assert!(spring_force > 0.0);
    
    // Test damping force
    suspension.velocity = 1.0;
    let damping_force = suspension.damping * suspension.velocity;
    assert!(damping_force > 0.0);
    
    // Test total force limits
    suspension.force = 100000.0; // Above limit
    suspension.force = suspension.force.clamp(-suspension.damage_threshold, suspension.damage_threshold);
    assert!(suspension.force <= suspension.damage_threshold);
}

#[test]
fn test_suspension_damage() {
    let mut suspension = Suspension::default();
    
    // Test damage accumulation
    suspension.force = suspension.damage_threshold * 1.5; // Excessive force
    update_suspension_damage(&mut suspension, 1.0);
    assert!(suspension.health < 100.0);
    assert!(suspension.accumulated_stress > 0.0);
    
    // Test damage recovery
    suspension.force = 0.0;
    update_suspension_damage(&mut suspension, 1.0);
    assert!(suspension.accumulated_stress < 2.0); // Should decrease under low stress
    
    // Test complete failure
    suspension.health = 0.0;
    update_suspension_damage(&mut suspension, 1.0);
    assert!(suspension.is_broken);
}

#[test]
fn test_suspension_geometry() {
    let mut suspension = Suspension::with_type(SuspensionType::LongArmLift);
    
    // Test lift kit geometry effects
    if let Some(lift_kit) = &suspension.lift_kit {
        // Verify mount point adjustments
        assert!(suspension.mount_point.y > 0.0); // Should be raised by lift
        
        // Test geometry correction
        let base_force = 1000.0;
        suspension.force = base_force;
        suspension.force *= lift_kit.geometry_correction;
        assert!(suspension.force > base_force); // Should be increased by correction
    }
}

#[test]
fn test_suspension_stress_calculation() {
    let mut suspension = Suspension::default();
    
    // Test force stress
    suspension.force = suspension.damage_threshold * 0.5;
    let dt = 0.016; // Typical frame time
    update_suspension_damage(&mut suspension, dt);
    assert!(suspension.accumulated_stress > 0.0);
    
    // Test compression stress
    suspension.compression = suspension.max_compression * 0.95; // Near limit
    update_suspension_damage(&mut suspension, dt);
    assert!(suspension.accumulated_stress > 0.0);
    
    // Test velocity stress
    suspension.velocity = 8.0; // High velocity
    update_suspension_damage(&mut suspension, dt);
    assert!(suspension.accumulated_stress > 0.0);
}

#[test]
fn test_suspension_health_effects() {
    let mut suspension = Suspension::default();
    
    // Test partial damage effects
    suspension.health = 50.0; // 50% health
    let base_force = 1000.0;
    suspension.force = base_force;
    
    // Force should be limited by health
    let max_force = suspension.damage_threshold * (suspension.health / 100.0);
    suspension.force = suspension.force.clamp(-max_force, max_force);
    assert!(suspension.force.abs() <= max_force);
    
    // Test complete failure effects
    suspension.health = 0.0;
    update_suspension_damage(&mut suspension, 1.0);
    assert!(suspension.is_broken);
    assert_eq!(suspension.force, 0.0); // No force when broken
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_lift_kit_effects() {
        let mut suspension = Suspension::default();
        suspension.suspension_type = SuspensionType::LongArmLift;
        suspension.lift_kit = Some(LiftKitConfig {
            lift_height: 0.1,
            arm_length: 0.5,
            arm_angle: 15.0_f32.to_radians(),
            track_width_increase: 0.05,
            geometry_correction: 0.02,
        });

        suspension.configure_type();
        
        assert_relative_eq!(suspension.rest_length, 0.4 + 0.1); // Base + lift height
        assert!(suspension.max_extension > 0.4); // Should be increased
        assert!(suspension.spring_stiffness < 50000.0); // Should be softer
    }

    #[test]
    fn test_portal_axle_configuration() {
        let mut suspension = Suspension::default();
        suspension.suspension_type = SuspensionType::PortalAxle;
        suspension.configure_type();

        assert!(suspension.rest_length > 0.4); // Should be higher
        assert!(suspension.max_compression < 0.2); // Should have less compression
        assert!(suspension.spring_stiffness > 50000.0); // Should be stiffer
    }

    #[test]
    fn test_air_suspension_adjustment() {
        let mut suspension = Suspension::default();
        suspension.suspension_type = SuspensionType::AirSuspension;
        suspension.configure_type();

        // Test height adjustment
        let initial_height = suspension.rest_length;
        suspension.adjust_air_pressure(1.2); // 20% increase
        assert!(suspension.rest_length > initial_height);
        
        // Test dynamic stiffness
        let initial_stiffness = suspension.spring_stiffness;
        suspension.adjust_air_pressure(0.8); // 20% decrease
        assert!(suspension.spring_stiffness < initial_stiffness);
    }

    #[test]
    fn test_extreme_terrain_behavior() {
        let mut suspension = Suspension::default();
        
        // Simulate rock crawling impact
        suspension.apply_force(100000.0); // High impact force
        assert!(suspension.accumulated_stress > 0.0);
        assert!(suspension.health < 100.0);

        // Test rapid compression/extension cycles
        for _ in 0..100 {
            suspension.velocity = 2.0;
            suspension.update_suspension_physics(0.016);
            suspension.velocity = -2.0;
            suspension.update_suspension_physics(0.016);
        }
        
        assert!(suspension.accumulated_stress > 0.5); // Should accumulate significant stress
    }

    #[test]
    fn test_suspension_tuning() {
        let mut suspension = Suspension::default();
        let tuning = SuspensionTuning {
            compression_damping: 5000.0,
            rebound_damping: 4000.0,
            high_speed_compression: 7000.0,
            high_speed_rebound: 6000.0,
            preload: 0.02,
        };
        
        suspension.apply_tuning(&tuning);
        
        // Test low speed damping
        suspension.velocity = 0.1;
        let low_speed_force = suspension.calculate_damping_force();
        
        // Test high speed damping
        suspension.velocity = 2.0;
        let high_speed_force = suspension.calculate_damping_force();
        
        assert!(high_speed_force.abs() > low_speed_force.abs());
    }

    #[test]
    fn test_broken_suspension_behavior() {
        let mut suspension = Suspension::default();
        
        // Break the suspension
        suspension.health = 0.0;
        suspension.is_broken = true;
        
        // Verify broken suspension behavior
        let force_before = suspension.force;
        suspension.apply_force(1000.0);
        assert_eq!(suspension.force, force_before); // Should not accumulate force
        
        // Verify no physics updates occur
        let pos_before = suspension.wheel_point;
        suspension.update_suspension_physics(0.016);
        assert_eq!(suspension.wheel_point, pos_before); // Position should not change
    }
} 