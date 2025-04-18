use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;

use crate::game::plugins::particle_system::collision::{
    CollisionBoxComponent,
    CollisionPlaneComponent,
    CollisionSphereComponent,
    ParticleCollisionBuffers,
    spawn_collision_box,
    spawn_collision_plane,
    spawn_collision_sphere,
};
use crate::game::plugins::particle_system::collision_plugin::ParticleCollisionPlugin;

fn setup_test_env() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        ParticleCollisionPlugin,
    ));
    app
}

#[test]
fn test_collision_buffers_creation() {
    let mut app = setup_test_env();
    
    // Run startup systems to initialize buffers
    app.update();
    
    // Verify collision buffers were created
    assert!(app.world.get_resource::<ParticleCollisionBuffers>().is_some());
}

#[test]
fn test_spawn_collision_plane() {
    let mut app = setup_test_env();
    
    let plane_entity = app.world.spawn_empty().id();
    spawn_collision_plane(
        &mut app.world,
        plane_entity,
        Vec3::Y,  // normal
        0.0,      // distance
        0.5,      // friction
        0.7,      // restitution
    );
    
    let plane = app.world.get::<CollisionPlaneComponent>(plane_entity);
    assert!(plane.is_some());
    
    let plane = plane.unwrap();
    assert_eq!(plane.normal, Vec3::Y);
    assert_eq!(plane.distance, 0.0);
    assert_eq!(plane.friction, 0.5);
    assert_eq!(plane.restitution, 0.7);
}

#[test]
fn test_spawn_collision_sphere() {
    let mut app = setup_test_env();
    
    let sphere_entity = app.world.spawn_empty().id();
    spawn_collision_sphere(
        &mut app.world,
        sphere_entity,
        Vec3::ZERO,  // center
        1.0,         // radius
        0.3,         // friction
        0.8,         // restitution
    );
    
    let sphere = app.world.get::<CollisionSphereComponent>(sphere_entity);
    assert!(sphere.is_some());
    
    let sphere = sphere.unwrap();
    assert_eq!(sphere.center, Vec3::ZERO);
    assert_eq!(sphere.radius, 1.0);
    assert_eq!(sphere.friction, 0.3);
    assert_eq!(sphere.restitution, 0.8);
}

#[test]
fn test_spawn_collision_box() {
    let mut app = setup_test_env();
    
    let box_entity = app.world.spawn_empty().id();
    spawn_collision_box(
        &mut app.world,
        box_entity,
        Vec3::ZERO,      // center
        Vec3::ONE,       // half_extents
        Quat::IDENTITY,  // rotation
        0.4,             // friction
        0.6,             // restitution
    );
    
    let box_comp = app.world.get::<CollisionBoxComponent>(box_entity);
    assert!(box_comp.is_some());
    
    let box_comp = box_comp.unwrap();
    assert_eq!(box_comp.center, Vec3::ZERO);
    assert_eq!(box_comp.half_extents, Vec3::ONE);
    assert_eq!(box_comp.rotation, Quat::IDENTITY);
    assert_eq!(box_comp.friction, 0.4);
    assert_eq!(box_comp.restitution, 0.6);
}

#[test]
fn test_update_collision_objects() {
    let mut app = setup_test_env();
    
    // Spawn one of each collision type
    let plane_entity = app.world.spawn_empty().id();
    spawn_collision_plane(
        &mut app.world,
        plane_entity,
        Vec3::Y,
        0.0,
        0.5,
        0.7,
    );
    
    let sphere_entity = app.world.spawn_empty().id();
    spawn_collision_sphere(
        &mut app.world,
        sphere_entity,
        Vec3::new(1.0, 2.0, 3.0),
        1.0,
        0.3,
        0.8,
    );
    
    let box_entity = app.world.spawn_empty().id();
    spawn_collision_box(
        &mut app.world,
        box_entity,
        Vec3::new(-1.0, 0.0, 1.0),
        Vec3::ONE,
        Quat::IDENTITY,
        0.4,
        0.6,
    );
    
    // Run update to process collision objects
    app.update();
    
    // Verify all objects still exist
    assert!(app.world.get::<CollisionPlaneComponent>(plane_entity).is_some());
    assert!(app.world.get::<CollisionSphereComponent>(sphere_entity).is_some());
    assert!(app.world.get::<CollisionBoxComponent>(box_entity).is_some());
}

#[test]
fn test_collision_buffer_limits() {
    let mut app = setup_test_env();
    
    // Spawn more than the maximum number of each type
    for _ in 0..40 {  // Max planes is 32
        let entity = app.world.spawn_empty().id();
        spawn_collision_plane(
            &mut app.world,
            entity,
            Vec3::Y,
            0.0,
            0.5,
            0.7,
        );
    }
    
    for _ in 0..70 {  // Max spheres is 64
        let entity = app.world.spawn_empty().id();
        spawn_collision_sphere(
            &mut app.world,
            entity,
            Vec3::ZERO,
            1.0,
            0.3,
            0.8,
        );
    }
    
    for _ in 0..40 {  // Max boxes is 32
        let entity = app.world.spawn_empty().id();
        spawn_collision_box(
            &mut app.world,
            entity,
            Vec3::ZERO,
            Vec3::ONE,
            Quat::IDENTITY,
            0.4,
            0.6,
        );
    }
    
    // Run update to process collision objects
    app.update();
    
    // Count actual number of components
    let plane_count = app.world.query::<&CollisionPlaneComponent>().iter(&app.world).count();
    let sphere_count = app.world.query::<&CollisionSphereComponent>().iter(&app.world).count();
    let box_count = app.world.query::<&CollisionBoxComponent>().iter(&app.world).count();
    
    assert_eq!(plane_count, 40);   // Components exist even if buffer is full
    assert_eq!(sphere_count, 70);
    assert_eq!(box_count, 40);
} 