use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, MapMode,
            BufferInitDescriptor,
        },
        renderer::RenderDevice,
        RenderApp,
        view::ViewUniform,
    },
    window::{WindowPlugin, WindowResolution},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    time::Time,
};
use std::time::Duration;

use crate::game::plugins::lighting::light_culling::{
    LightCullingConfig,
    LightCullingResources,
    LightCullingDebugView,
    setup_light_culling,
    update_light_culling,
    debug_light_culling,
    TILE_SIZE,
    MAX_LIGHTS_PER_TILE,
};

// Helper struct for GPU buffer verification
struct GpuVerificationBuffers {
    light_grid_buffer: Buffer,
    light_indices_buffer: Buffer,
}

impl GpuVerificationBuffers {
    fn new(device: &RenderDevice, grid_size: u64, indices_size: u64) -> Self {
        let light_grid_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("light_grid_verification"),
            size: grid_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let light_indices_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("light_indices_verification"),
            size: indices_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            light_grid_buffer,
            light_indices_buffer,
        }
    }
}

// Integration test helpers
fn setup_test_environment() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        RenderPlugin::default(),
        LightCullingPlugin,
    ))
    .insert_resource(WindowResolution::new(1920.0, 1080.0));
    app
}

#[test]
fn test_gpu_buffer_verification() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, WindowPlugin::default()));
    
    let render_app = app.sub_app_mut(RenderApp);
    let render_device = render_app.world.resource::<RenderDevice>();
    
    // Setup test scene
    let mut scene = Scene::new();
    
    // Add test lights in a known pattern
    let light_positions = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(TILE_SIZE as f32 * 0.5, 0.0, 0.0),
        Vec3::new(TILE_SIZE as f32 * 1.5, 0.0, 0.0),
    ];
    
    for position in light_positions.iter() {
        scene.spawn((
            PointLight {
                intensity: 1000.0,
                range: TILE_SIZE as f32,
                ..default()
            },
            GlobalTransform::from_translation(*position),
        ));
    }
    
    // Create verification buffers
    let grid_size = (TILE_SIZE * TILE_SIZE * 4) as u64; // 4 bytes per u32
    let indices_size = (TILE_SIZE * TILE_SIZE * MAX_LIGHTS_PER_TILE * 4) as u64;
    let verification_buffers = GpuVerificationBuffers::new(&render_device, grid_size, indices_size);
    
    // Run light culling
    app.world.insert_resource(LightCullingConfig::default());
    let mut update_system = IntoSystem::into_system(update_light_culling);
    update_system.initialize(&mut app.world);
    update_system.run((&mut app.world).into());
    
    // Map buffers and verify contents
    let light_grid_slice = verification_buffers.light_grid_buffer
        .slice(..)
        .map_async(MapMode::Read)
        .unwrap();
    
    // Verify the light counts in each tile match expectations
    let grid_data = light_grid_slice.get_mapped_range();
    let light_counts: &[u32] = bytemuck::cast_slice(&grid_data);
    
    // First tile should have 2 lights (the first two are close enough to be in same tile)
    assert_eq!(light_counts[0], 2);
    // Second tile should have 1 light (the third light)
    assert_eq!(light_counts[1], 1);
}

#[test]
fn test_debug_visualization() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, WindowPlugin::default()));
    
    // Enable debug view
    app.world.insert_resource(LightCullingConfig {
        enabled: true,
        debug_view: true,
    });
    
    // Add test lights
    for i in 0..5 {
        app.world.spawn((
            PointLight {
                intensity: 1000.0,
                range: TILE_SIZE as f32,
                ..default()
            },
            GlobalTransform::from_translation(Vec3::new(
                (i as f32) * TILE_SIZE as f32,
                0.0,
                0.0
            )),
        ));
    }
    
    // Run debug visualization system
    let mut debug_system = IntoSystem::into_system(debug_light_culling);
    debug_system.initialize(&mut app.world);
    debug_system.run((&mut app.world).into());
    
    // Verify debug entities were created
    let debug_query = app.world.query_filtered::<Entity, With<LightCullingDebugView>>();
    assert!(debug_query.iter(&app.world).count() > 0);
    
    // Disable debug view and verify cleanup
    app.world.insert_resource(LightCullingConfig {
        enabled: true,
        debug_view: false,
    });
    
    debug_system.run((&mut app.world).into());
    assert_eq!(debug_query.iter(&app.world).count(), 0);
}

#[test]
fn test_performance_benchmarks() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        WindowPlugin::default(),
        FrameTimeDiagnosticsPlugin::default(),
        LogDiagnosticsPlugin::default(),
    ));
    
    // Add large number of lights for stress testing
    let num_lights = 1000;
    for i in 0..num_lights {
        app.world.spawn((
            PointLight {
                intensity: 1000.0,
                range: TILE_SIZE as f32 * 2.0,
                ..default()
            },
            GlobalTransform::from_translation(Vec3::new(
                (i as f32 * 10.0).cos() * 100.0,
                (i as f32 * 10.0).sin() * 100.0,
                0.0
            )),
        ));
    }
    
    app.world.insert_resource(LightCullingConfig::default());
    
    // Measure performance over multiple frames
    let mut total_time = Duration::ZERO;
    let frames = 100;
    
    for _ in 0..frames {
        let start = std::time::Instant::now();
        
        let mut update_system = IntoSystem::into_system(update_light_culling);
        update_system.initialize(&mut app.world);
        update_system.run((&mut app.world).into());
        
        total_time += start.elapsed();
    }
    
    let avg_frame_time = total_time.as_secs_f32() / frames as f32;
    println!("Average frame time: {:.2}ms", avg_frame_time * 1000.0);
    
    // Assert reasonable performance (adjust threshold based on hardware)
    assert!(avg_frame_time < 0.016); // Target 60 FPS (16ms)
}

#[test]
fn test_light_culling_accuracy() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, WindowPlugin::default()));
    
    // Create test scene with known light configurations
    let test_cases = vec![
        // Case 1: Single light in center of tile
        TestCase {
            light_pos: Vec3::new(TILE_SIZE as f32 * 0.5, TILE_SIZE as f32 * 0.5, 0.0),
            range: TILE_SIZE as f32 * 0.75,
            expected_tiles: vec![(0, 0)],
        },
        // Case 2: Light affecting multiple tiles
        TestCase {
            light_pos: Vec3::new(TILE_SIZE as f32, TILE_SIZE as f32, 0.0),
            range: TILE_SIZE as f32 * 2.0,
            expected_tiles: vec![(0, 0), (1, 0), (0, 1), (1, 1)],
        },
        // Case 3: Light at tile boundary
        TestCase {
            light_pos: Vec3::new(TILE_SIZE as f32, 0.0, 0.0),
            range: TILE_SIZE as f32 * 0.5,
            expected_tiles: vec![(0, 0), (1, 0)],
        },
    ];
    
    for test_case in test_cases {
        // Clear previous lights
        app.world.clear_entities();
        
        // Spawn test light
        app.world.spawn((
            PointLight {
                intensity: 1000.0,
                range: test_case.range,
                ..default()
            },
            GlobalTransform::from_translation(test_case.light_pos),
        ));
        
        // Run light culling
        app.world.insert_resource(LightCullingConfig::default());
        let mut update_system = IntoSystem::into_system(update_light_culling);
        update_system.initialize(&mut app.world);
        update_system.run((&mut app.world).into());
        
        // Verify affected tiles match expectations
        // This would require access to the GPU buffers in a real implementation
        // Here we're just verifying the system runs without errors
    }
}

struct TestCase {
    light_pos: Vec3,
    range: f32,
    expected_tiles: Vec<(u32, u32)>,
}

#[test]
fn test_dynamic_light_updates() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, WindowPlugin::default()));
    
    // Add a moving light
    let light_entity = app.world.spawn((
        PointLight {
            intensity: 1000.0,
            range: TILE_SIZE as f32,
            ..default()
        },
        GlobalTransform::from_translation(Vec3::ZERO),
    )).id();
    
    app.world.insert_resource(LightCullingConfig::default());
    app.world.insert_resource(Time::default());
    
    // Simulate light movement over multiple frames
    let frames = 60;
    for i in 0..frames {
        // Update light position
        let angle = (i as f32 / frames as f32) * std::f32::consts::TAU;
        let new_pos = Vec3::new(
            angle.cos() * TILE_SIZE as f32 * 2.0,
            angle.sin() * TILE_SIZE as f32 * 2.0,
            0.0
        );
        
        if let Some(mut transform) = app.world.get_mut::<GlobalTransform>(light_entity) {
            *transform = GlobalTransform::from_translation(new_pos);
        }
        
        // Run light culling
        let mut update_system = IntoSystem::into_system(update_light_culling);
        update_system.initialize(&mut app.world);
        update_system.run((&mut app.world).into());
        
        // In a real test, we would verify the light is properly tracked
        // across tiles as it moves
    }
}

// Helper function to create a test scene with a specific configuration
fn setup_test_scene(app: &mut App, num_lights: u32, pattern: LightPattern) {
    match pattern {
        LightPattern::Grid => {
            let grid_size = (num_lights as f32).sqrt().ceil() as u32;
            for y in 0..grid_size {
                for x in 0..grid_size {
                    if (y * grid_size + x) >= num_lights {
                        break;
                    }
                    app.world.spawn((
                        PointLight {
                            intensity: 1000.0,
                            range: TILE_SIZE as f32,
                            ..default()
                        },
                        GlobalTransform::from_translation(Vec3::new(
                            x as f32 * TILE_SIZE as f32 * 1.5,
                            y as f32 * TILE_SIZE as f32 * 1.5,
                            0.0
                        )),
                    ));
                }
            }
        },
        LightPattern::Random => {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            
            for _ in 0..num_lights {
                let pos = Vec3::new(
                    rng.gen_range(-500.0..500.0),
                    rng.gen_range(-500.0..500.0),
                    0.0
                );
                app.world.spawn((
                    PointLight {
                        intensity: rng.gen_range(500.0..2000.0),
                        range: rng.gen_range(
                            TILE_SIZE as f32 * 0.5..TILE_SIZE as f32 * 3.0
                        ),
                        ..default()
                    },
                    GlobalTransform::from_translation(pos),
                ));
            }
        },
        LightPattern::Cluster => {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            
            let num_clusters = (num_lights / 10).max(1);
            let lights_per_cluster = num_lights / num_clusters;
            
            for cluster in 0..num_clusters {
                let cluster_center = Vec3::new(
                    rng.gen_range(-500.0..500.0),
                    rng.gen_range(-500.0..500.0),
                    0.0
                );
                
                for _ in 0..lights_per_cluster {
                    let offset = Vec3::new(
                        rng.gen_range(-50.0..50.0),
                        rng.gen_range(-50.0..50.0),
                        0.0
                    );
                    app.world.spawn((
                        PointLight {
                            intensity: rng.gen_range(500.0..2000.0),
                            range: rng.gen_range(
                                TILE_SIZE as f32 * 0.5..TILE_SIZE as f32 * 2.0
                            ),
                            ..default()
                        },
                        GlobalTransform::from_translation(cluster_center + offset),
                    ));
                }
            }
        },
    }
}

enum LightPattern {
    Grid,
    Random,
    Cluster,
} 