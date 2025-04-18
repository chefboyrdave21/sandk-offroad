use bevy::{
    prelude::*,
    render::{
        render_resource::{ShaderType, ShaderRef},
        renderer::RenderDevice,
    },
};
use std::f32::consts::PI;

#[test]
fn test_view_uniform_layout() {
    // Verify ViewUniform struct matches shader expectations
    let view_uniform_size = std::mem::size_of::<ViewUniform>();
    assert_eq!(view_uniform_size % 16, 0, "ViewUniform size must be 16-byte aligned");
}

#[test]
fn test_vertex_transformation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let position = Vec3::new(1.0, 2.0, 3.0);
    let normal = Vec3::new(0.0, 1.0, 0.0);
    
    // Create test view matrix (looking down -Z axis)
    let view = Mat4::look_at_rh(
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::ZERO,
        Vec3::Y
    );
    
    // Create test projection matrix
    let projection = Mat4::perspective_rh(
        PI / 4.0,
        1.0,
        0.1,
        100.0
    );
    
    let view_proj = projection * view;
    
    // Transform vertex manually
    let world_pos = Vec4::new(position.x, position.y, position.z, 1.0);
    let clip_pos = view_proj * world_pos;
    
    // Verify position is correctly transformed
    assert!((clip_pos.x/clip_pos.w).abs() < 1.0);
    assert!((clip_pos.y/clip_pos.w).abs() < 1.0);
    assert!(clip_pos.z/clip_pos.w >= 0.0 && clip_pos.z/clip_pos.w <= 1.0);
}

#[test]
fn test_depth_calculation() {
    let world_pos = Vec3::new(2.0, 0.0, 0.0);
    let light_pos = Vec3::ZERO;
    let near = 0.1;
    let far = 10.0;
    
    // Calculate expected depth
    let light_to_frag = world_pos.distance(light_pos);
    let expected_depth = (light_to_frag - near) / (far - near);
    
    // Verify depth is in valid range
    assert!(expected_depth >= 0.0 && expected_depth <= 1.0);
    
    // Test edge cases
    let near_point = light_pos + Vec3::X * near;
    let far_point = light_pos + Vec3::X * far;
    
    let near_depth = (near_point.distance(light_pos) - near) / (far - near);
    let far_depth = (far_point.distance(light_pos) - near) / (far - near);
    
    assert!(near_depth <= 0.01); // Should be very close to 0
    assert!(far_depth >= 0.99);  // Should be very close to 1
}

#[test]
fn test_normal_transformation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let normal = Vec3::new(0.0, 1.0, 0.0);
    
    // Create a rotated view matrix
    let rotation = Quat::from_rotation_x(PI / 4.0);
    let view = Mat4::from_quat(rotation);
    let inverse_view = view.inverse();
    
    // Transform normal
    let transformed_normal = (inverse_view * Vec4::new(normal.x, normal.y, normal.z, 0.0)).xyz();
    let normalized = transformed_normal.normalize();
    
    // Verify normal is still unit length
    assert!((normalized.length() - 1.0).abs() < 0.0001);
    
    // Verify rotation was applied correctly
    let expected_normal = rotation.mul_vec3(normal);
    assert!((normalized - expected_normal).length() < 0.0001);
}

#[test]
fn test_view_projection_matrix() {
    // Test different projection parameters
    let fov = PI / 3.0; // 60 degrees
    let aspect = 1.0;
    let near = 0.1;
    let far = 50.0;
    
    let projection = Mat4::perspective_rh(fov, aspect, near, far);
    
    // Test perspective matrix properties
    assert_eq!(projection.w_axis.w, 0.0); // Perspective projection
    assert_eq!(projection.z_axis.w, -1.0); // RH coordinate system
    
    // Test view matrix with translation
    let eye = Vec3::new(1.0, 2.0, 3.0);
    let target = Vec3::ZERO;
    let up = Vec3::Y;
    
    let view = Mat4::look_at_rh(eye, target, up);
    
    // Verify camera position
    let inverse_view = view.inverse();
    let camera_pos = inverse_view.w_axis.xyz();
    assert!((camera_pos - eye).length() < 0.0001);
}

#[test]
fn test_multiple_light_positions() {
    let test_positions = [
        Vec3::new(0.0, 5.0, 0.0),  // Above
        Vec3::new(5.0, 0.0, 0.0),  // Right
        Vec3::new(0.0, 0.0, 5.0),  // Front
        Vec3::new(3.0, 4.0, 5.0),  // Diagonal
    ];
    
    let fragment_pos = Vec3::ZERO;
    let near = 0.1;
    let far = 10.0;
    
    for light_pos in test_positions.iter() {
        let light_to_frag = fragment_pos.distance(*light_pos);
        let depth = (light_to_frag - near) / (far - near);
        
        // Verify depth calculation for each light position
        assert!(depth >= 0.0 && depth <= 1.0);
        
        // Verify distance calculation
        let expected_distance = light_pos.length();
        assert!((light_to_frag - expected_distance).abs() < 0.0001);
    }
}

#[test]
fn test_edge_case_positions() {
    let near = 0.1;
    let far = 10.0;
    let light_pos = Vec3::ZERO;
    
    // Test positions
    let test_cases = vec![
        (Vec3::ZERO, 0.0),                    // At light position
        (Vec3::new(0.0, near, 0.0), 0.0),    // At near plane
        (Vec3::new(0.0, far, 0.0), 1.0),     // At far plane
        (Vec3::new(far, far, far), 1.0),     // Beyond far plane diagonal
        (Vec3::new(near/2.0, 0.0, 0.0), 0.0) // Inside near plane
    ];
    
    for (pos, expected_normalized_depth) in test_cases {
        let light_to_frag = pos.distance(light_pos);
        let depth = (light_to_frag - near) / (far - near);
        depth.clamp(0.0, 1.0);
        
        assert!((depth - expected_normalized_depth).abs() < 0.01,
                "Failed for position {:?}, got depth {}, expected {}",
                pos, depth, expected_normalized_depth);
    }
}

#[derive(ShaderType)]
struct ViewUniform {
    view_proj: Mat4,
    view: Mat4,
    inverse_view: Mat4,
    projection: Mat4,
    world_position: Vec4,
    near: f32,
    far: f32,
} 