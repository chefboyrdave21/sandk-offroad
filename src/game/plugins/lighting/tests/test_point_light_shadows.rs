use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::camera::CameraProjection;
use bevy::render::view::RenderLayers;
use crate::game::plugins::lighting::point_light_shadows::*;

#[test]
fn test_point_light_shadow_map_creation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Add required resources
    app.world.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        GlobalTransform::default(),
    ));

    // Run the system
    app.add_systems(Update, update_point_light_shadows);
    app.update();

    // Verify shadow map was created
    let shadow_maps = app.world.query::<&PointLightShadowMap>().iter(&app.world).count();
    assert_eq!(shadow_maps, 1);
}

#[test]
fn test_shadow_matrices_update() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Spawn a point light at a specific position
    let transform = Transform::from_xyz(1.0, 2.0, 3.0);
    let entity = app.world.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform.clone(),
        GlobalTransform::from(transform),
    )).id();

    // Run the system to create and update shadow map
    app.add_systems(Update, update_point_light_shadows);
    app.update();

    // Get the shadow map and verify matrices
    let shadow_map = app.world.entity(entity).get::<PointLightShadowMap>().unwrap();
    
    // Check that view matrices are different for each face
    for i in 0..6 {
        for j in (i + 1)..6 {
            assert_ne!(shadow_map.view_matrices[i], shadow_map.view_matrices[j]);
        }
    }

    // Verify projection matrix properties
    let proj = shadow_map.proj_matrix;
    assert_eq!(proj.x_axis.w, 0.0); // No perspective translation
    assert_eq!(proj.y_axis.w, 0.0);
    assert_eq!(proj.z_axis.w, 0.0);
}

#[test]
fn test_shadow_map_texture_properties() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Add required resources
    let entity = app.world.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        GlobalTransform::default(),
    )).id();

    // Run the system
    app.add_systems(Update, update_point_light_shadows);
    app.update();

    // Get the shadow map
    let shadow_map = app.world.entity(entity).get::<PointLightShadowMap>().unwrap();
    let images = app.world.resource::<Assets<Image>>();
    let texture = images.get(&shadow_map.texture).unwrap();

    // Verify texture properties
    assert_eq!(texture.texture_descriptor.size.width, POINT_SHADOW_MAP_SIZE);
    assert_eq!(texture.texture_descriptor.size.height, POINT_SHADOW_MAP_SIZE);
    assert_eq!(texture.texture_descriptor.size.depth_or_array_layers, 6);
    assert_eq!(texture.texture_descriptor.format, TextureFormat::Depth32Float);
    assert_eq!(
        texture.texture_descriptor.usage,
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING
    );
    assert_eq!(
        texture.texture_view_descriptor.as_ref().unwrap().dimension,
        Some(TextureViewDimension::Cube)
    );
}

#[test]
fn test_disabled_shadows() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Add point light with shadows disabled
    app.world.spawn((
        PointLight {
            shadows_enabled: false,
            ..default()
        },
        GlobalTransform::default(),
    ));

    // Run the system
    app.add_systems(Update, update_point_light_shadows);
    app.update();

    // Verify no shadow map was created
    let shadow_maps = app.world.query::<&PointLightShadowMap>().iter(&app.world).count();
    assert_eq!(shadow_maps, 0);
} 