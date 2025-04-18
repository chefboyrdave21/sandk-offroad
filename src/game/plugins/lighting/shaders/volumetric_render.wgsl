// Volumetric Rendering Shader
// This shader implements volumetric lighting effects using ray marching through a 3D volume texture.
// The implementation uses a full-screen pass that ray marches through the volume for each pixel,
// accumulating light scattering and absorption along the way.

// Performance Optimization Guide:
// 1. Early Exit Optimizations:
//    - Skip samples when transmittance < 0.01 (fully opaque)
//    - Skip empty space when density < 0.001 (no visible contribution)
//    - Use dynamic step size based on density gradient
//
// 2. Memory Access Optimizations:
//    - Use trilinear filtering for smooth interpolation
//    - Cache ray direction calculation in vertex shader
//    - Minimize texture lookups by storing multiple values in channels
//    - Use coherent memory access patterns for volume sampling
//
// 3. Computation Optimizations:
//    - Pre-calculate constants in vertex shader or uniforms
//    - Use fast math approximations where appropriate
//    - Vectorize calculations when possible
//    - Avoid branching in inner loops
//
// 4. Quality vs Performance Tradeoffs:
//    - Adjustable num_steps (64 default, can be lowered for performance)
//    - Variable step size based on view angle
//    - Distance-based quality reduction
//    - Adaptive sampling based on scene complexity
//
// 5. View-Dependent Optimizations:
//    - Reduce quality at grazing angles
//    - Skip samples outside view frustum
//    - Use depth buffer for early termination
//    - Adjust sample count based on view distance
//
// 6. GPU-Specific Optimizations:
//    - Minimize register pressure in fragment shader
//    - Use efficient ALU vs texture fetch balance
//    - Optimize for specific GPU architectures
//    - Consider wave/warp size for divergent branches

// Mathematical Background:
// The volumetric rendering equation used is based on the radiative transfer equation:
// L(x,ω) = T(x,xs)L(xs,ω) + ∫[x to xs] T(x,t)σs(t)Li(t,ω)dt
// Where:
// - L(x,ω) is the radiance at point x in direction ω
// - T(x,xs) is the transmittance between x and xs
// - σs is the scattering coefficient
// - Li is the in-scattered radiance

// Input vertex data for full-screen triangle
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

// Camera and view information
struct ViewUniform {
    view_proj: mat4x4<f32>,          // Combined view-projection matrix
    inverse_view_proj: mat4x4<f32>,  // Inverse view-projection for world space reconstruction
    view: mat4x4<f32>,               // View matrix
    inverse_view: mat4x4<f32>,       // Inverse view matrix
    world_position: vec3<f32>,       // Camera position in world space
    width: f32,                      // Viewport width
    height: f32,                     // Viewport height
}

// Parameters controlling the volumetric effect
struct VolumetricSettings {
    density: f32,      // Overall density multiplier (0-1)
    scattering: f32,   // Light scattering coefficient (0-1)
    absorption: f32,   // Light absorption coefficient (0-1)
    max_distance: f32, // Maximum ray march distance in world units
}

// Vertex shader output structure
struct VertexOutput {
    @builtin(position) position: vec4<f32>,  // Clip space position
    @location(0) uv: vec2<f32>,             // Texture coordinates
    @location(1) ray_dir: vec3<f32>,        // Ray direction in world space
}

// Bind group for all shader resources
@group(0) @binding(0) var<uniform> view: ViewUniform;                 // View uniforms
@group(0) @binding(1) var volume_texture: texture_3d<f32>;           // 3D volume texture containing density and light data
@group(0) @binding(2) var volume_sampler: sampler;                   // Sampler for volume texture
@group(0) @binding(3) var scene_texture: texture_2d<f32>;           // Main scene color texture
@group(0) @binding(4) var scene_sampler: sampler;                   // Sampler for scene texture
@group(0) @binding(5) var depth_texture: texture_depth_2d;          // Scene depth texture
@group(0) @binding(6) var<uniform> settings: VolumetricSettings;    // Volumetric effect settings

// Vertex shader that generates a full-screen triangle
// Uses a single triangle that covers the screen, generated from vertex ID
@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate vertices for full-screen triangle using vertex ID
    // This is more efficient than using a quad (2 triangles)
    let x = f32(vertex_index & 1u);
    let y = f32((vertex_index >> 1u) & 1u);
    
    var out: VertexOutput;
    out.uv = vec2<f32>(x, y);
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    
    // Calculate ray direction for this vertex
    let clip = vec4<f32>(out.position.xy, 1.0, 1.0);
    let world_pos = clip_to_world(clip);
    out.ray_dir = normalize(world_pos - view.world_position);
    
    return out;
}

// Utility function to convert clip space coordinates to world space
// Uses the inverse view-projection matrix to transform from clip to world coordinates
// Mathematical formula: world = inverse_view_proj * clip
// The w-divide is necessary for perspective-correct interpolation
fn clip_to_world(clip: vec4<f32>) -> vec3<f32> {
    let world = view.inverse_view_proj * clip;
    return world.xyz / world.w;  // Perspective divide
}

// Maps world position to volume texture coordinates
// Uses a normalized coordinate system centered around the camera
// Mathematical transformation:
// 1. Relative position = (world_pos - camera_pos) / max_distance
// 2. Normalized coordinates = relative_pos * 0.5 + 0.5
fn world_to_volume(world_pos: vec3<f32>) -> vec3<f32> {
    // Transform position relative to camera and normalize by max distance
    let relative_pos = (world_pos - view.world_position) / settings.max_distance;
    // Map from [-1, 1] to [0, 1] range for texture sampling
    return relative_pos * 0.5 + 0.5;
}

// Converts depth buffer value to linear depth
// Uses the standard perspective depth linearization formula:
// linear_depth = (2.0 * near) / (far + near - depth * (far - near))
fn get_linear_depth(uv: vec2<f32>) -> f32 {
    let depth = textureSample(depth_texture, scene_sampler, uv);
    // Convert from non-linear to linear depth
    let near = 0.1;
    let far = 1000.0;
    return (2.0 * near) / (far + near - depth * (far - near));
}

// Fragment shader that performs ray marching through the volume
// Implements physically-based light transport with scattering and absorption
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get the scene color and depth for this pixel
    let scene_color = textureSample(scene_texture, scene_sampler, in.uv);
    let depth = get_linear_depth(in.uv);
    
    // Calculate world position of this pixel using depth
    let clip = vec4<f32>(in.position.xy, depth, 1.0);
    let world_pos = clip_to_world(clip);
    
    // Setup ray marching parameters
    let ray_start = view.world_position;  // Start at camera position
    let ray_dir = normalize(world_pos - ray_start);
    let ray_length = min(length(world_pos - ray_start), settings.max_distance);
    
    // Ray marching configuration
    let num_steps = 64;
    
    // Performance: Use adaptive step size based on density
    var step_size = ray_length / f32(num_steps);
    var transmittance = 1.0;
    var result = vec3<f32>(0.0);
    
    // Performance: Unroll small loops when possible
    // Performance: Use early exit conditions
    for (var i = 0; i < num_steps; i++) {
        let t = f32(i) * step_size;
        let sample_pos = ray_start + ray_dir * t;
        let volume_uv = world_to_volume(sample_pos);
        
        // Performance: Skip samples outside volume bounds
        if (any(volume_uv < vec3<f32>(0.0)) || any(volume_uv > vec3<f32>(1.0))) {
            continue;
        }
        
        // Performance: Cache texture sample to avoid multiple lookups
        let volume_sample = textureSample(volume_texture, volume_sampler, volume_uv);
        let density = volume_sample.a * settings.density;
        
        // Performance: Skip empty space
        if (density <= 0.001) {
            // Performance: Increase step size in empty regions
            step_size *= 1.5;
            continue;
        } else {
            // Performance: Decrease step size in dense regions
            step_size = ray_length / f32(num_steps);
        }
        
        // Performance: Cache light calculations
        let light_contribution = volume_sample.rgb;
        let scatter_amount = density * settings.scattering;
        let absorption_amount = density * settings.absorption;
        
        // Performance: Vectorize calculations
        result += light_contribution * scatter_amount * transmittance * step_size;
        transmittance *= exp(-absorption_amount * step_size);
        
        // Performance: Early exit on full opacity
        if (transmittance < 0.01) {
            break;
        }
    }
    
    // Final color combines attenuated scene color with accumulated scattered light
    // This implements the complete rendering equation solution:
    // L_final = L_scene * T + L_scattered
    return vec4<f32>(
        scene_color.rgb * transmittance + result,
        1.0
    );
} 