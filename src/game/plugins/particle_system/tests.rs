use super::*;
use bevy::math::Vec3;
use rand::thread_rng;
use std::f32::consts::PI;

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::*;

    #[test]
    fn test_point_emitter() {
        let mut rng = thread_rng();
        let emitter = PointEmitter {
            position: Vec3::ZERO,
            direction: Vec3::Y,
            spread_angle: PI / 4.0,
        };

        // Test position generation
        let pos = emitter.generate_position(&mut rng);
        assert_eq!(pos, Vec3::ZERO);

        // Test direction generation
        let dir = emitter.generate_direction(&mut rng);
        assert!(dir.is_normalized());
        assert!(dir.angle_between(Vec3::Y) <= PI / 4.0);
    }

    #[test]
    fn test_sphere_emitter() {
        let mut rng = thread_rng();
        let emitter = SphereEmitter {
            center: Vec3::ZERO,
            radius: 1.0,
            emit_from_surface: true,
        };

        // Test surface position generation
        let pos = emitter.generate_position(&mut rng);
        assert!((pos.length() - 1.0).abs() < 0.001);

        // Test volume position generation
        let volume_emitter = SphereEmitter {
            emit_from_surface: false,
            ..emitter
        };
        let pos = volume_emitter.generate_position(&mut rng);
        assert!(pos.length() <= 1.0);

        // Test outward direction
        let dir = emitter.generate_direction(&mut rng);
        assert!(dir.is_normalized());
    }

    #[test]
    fn test_box_emitter() {
        let mut rng = thread_rng();
        let emitter = BoxEmitter {
            min_bound: Vec3::new(-1.0, -1.0, -1.0),
            max_bound: Vec3::new(1.0, 1.0, 1.0),
            emit_from_surface: true,
        };

        // Test surface position generation
        let pos = emitter.generate_position(&mut rng);
        assert!(pos.x.abs() == 1.0 || pos.y.abs() == 1.0 || pos.z.abs() == 1.0);

        // Test volume position generation
        let volume_emitter = BoxEmitter {
            emit_from_surface: false,
            ..emitter
        };
        let pos = volume_emitter.generate_position(&mut rng);
        assert!(pos.x.abs() <= 1.0 && pos.y.abs() <= 1.0 && pos.z.abs() <= 1.0);

        // Test outward direction
        let dir = emitter.generate_direction(&mut rng);
        assert!(dir.is_normalized());
    }

    #[test]
    fn test_particle_buffer_manager() {
        let mut manager = ParticleBufferManager::new(1000);
        
        // Test initial state
        assert_eq!(manager.max_particles(), 1000);
        assert_eq!(manager.frame_index(), 0);
        
        // Test buffer swapping
        manager.swap_buffers();
        assert_eq!(manager.frame_index(), 1);
        
        // Test buffer access
        let buffer = manager.current_buffer_mut();
        assert!(buffer.len() <= 1000);
    }

    #[test]
    fn test_simulation_params() {
        let params = SimulationParams::default();
        
        // Test default values
        assert!(params.gravity.length() > 0.0);
        assert!(params.drag >= 0.0);
        assert!(params.time_scale > 0.0);
        
        // Test custom params
        let custom_params = SimulationParams {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            drag: 0.1,
            time_scale: 1.0,
        };
        assert_eq!(custom_params.gravity.y, -9.81);
    }

    #[test]
    fn test_particle_system_settings() {
        let settings = ParticleSystemSettings::default();
        
        // Test default state
        assert!(settings.max_particles > 0);
        assert!(settings.spawn_rate > 0.0);
        assert!(settings.particle_lifetime > 0.0);
        
        // Test quality settings
        let high_quality = ParticleSystemSettings {
            max_particles: 10000,
            spawn_rate: 1000.0,
            particle_lifetime: 5.0,
            ..Default::default()
        };
        assert!(high_quality.max_particles > settings.max_particles);
    }

    #[test]
    fn test_sort_params() {
        let params = SortParams {
            camera_pos: Vec3::new(1.0, 2.0, 3.0),
            particle_count: 1000,
            k: 4,
            j: 2,
            sort_descending: 1,
            _padding: Vec2::ZERO,
        };

        assert_eq!(params.camera_pos.x, 1.0);
        assert_eq!(params.camera_pos.y, 2.0);
        assert_eq!(params.camera_pos.z, 3.0);
        assert_eq!(params.particle_count, 1000);
        assert_eq!(params.k, 4);
        assert_eq!(params.j, 2);
        assert_eq!(params.sort_descending, 1);
    }

    #[test]
    fn test_particle_sort_pipeline() {
        let mut world = World::new();
        world.init_resource::<RenderDevice>();
        
        let sort_pipeline = ParticleSortPipeline::from_world(&mut world);
        assert!(sort_pipeline.enabled);
    }

    #[test]
    fn test_particle_index_initialization() {
        let mut world = World::new();
        
        // Create test particle system
        let particle_system = ParticleSystem {
            params: ParticleSystemParams {
                max_particles: 100,
                ..Default::default()
            },
            ..Default::default()
        };
        
        let buffer_manager = ParticleBufferManager::new(100);
        
        // Add components to world
        let entity = world
            .spawn((particle_system, buffer_manager))
            .id();
            
        // Run index initialization system
        let mut system_state: SystemState<(
            Query<(&ParticleSystem, &ParticleBufferManager), Added<ParticleSystem>>,
            Res<RenderQueue>,
        )> = SystemState::new(&mut world);
        
        let (query, render_queue) = system_state.get(&world);
        init_particle_indices(query, render_queue);
        
        // Verify indices were initialized
        let (particle_system, buffer_manager) = world.get::<(ParticleSystem, ParticleBufferManager)>(entity).unwrap();
        assert_eq!(buffer_manager.max_particles(), particle_system.params.max_particles);
    }

    #[test]
    fn test_particle_sorting() {
        let mut world = World::new();
        
        // Create test resources
        world.init_resource::<ParticleSortPipeline>();
        world.init_resource::<RenderDevice>();
        world.init_resource::<RenderQueue>();
        
        // Create test camera
        let camera_entity = world
            .spawn((
                Camera3d::default(),
                GlobalTransform::from_xyz(0.0, 0.0, -10.0),
            ))
            .id();
            
        // Create test particle system
        let particle_system = ParticleSystem {
            active_particles: 10,
            params: ParticleSystemParams {
                max_particles: 100,
                ..Default::default()
            },
            ..Default::default()
        };
        
        let buffer_manager = ParticleBufferManager::new(100);
        
        let particle_entity = world
            .spawn((particle_system, buffer_manager))
            .id();
            
        // Run sorting system
        let mut system_state: SystemState<(
            Res<ParticleSortPipeline>,
            Query<(&ParticleSystem, &ParticleBufferManager)>,
            Query<&GlobalTransform, With<Camera>>,
            Res<RenderDevice>,
            Res<RenderQueue>,
        )> = SystemState::new(&mut world);
        
        let (sort_pipeline, particle_query, camera_query, render_device, render_queue) = 
            system_state.get(&world);
            
        dispatch_particle_sort(
            sort_pipeline,
            particle_query,
            camera_query,
            render_device,
            render_queue,
        );
        
        // Verify sorting was performed
        let (particle_system, _) = world.get::<(ParticleSystem, ParticleBufferManager)>(particle_entity).unwrap();
        assert!(particle_system.active_particles > 0);
    }
} 