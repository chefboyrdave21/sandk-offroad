// Particle Material Shader with GPU Instancing Support
//
// Features:
// - GPU instanced rendering
// - Batched draw calls
// - Dynamic LOD
// - Performance optimizations
// - Advanced effects

// Vertex shader inputs and outputs
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) emission: f32,
    @location(4) particle_age: f32,
};

// Material parameters
struct MaterialParams {
    view_proj: mat4x4<f32>,
    camera_position: vec3<f32>,
    camera_right: vec3<f32>,
    camera_up: vec3<f32>,
    atlas_size: vec2<f32>,
    frame_count: u32,
    current_time: f32,
    animation_fps: f32,
    alpha_threshold: f32,
    emission_strength: f32,
    depth_fade_distance: f32,
    color_tint: vec4<f32>,
    custom_params: vec4<f32>,
    distance_fade_range: vec2<f32>,
    gradient_color_start: vec4<f32>,
    gradient_color_end: vec4<f32>,
    effect_params: vec4<f32>,  // x: noise scale, y: noise speed, z: gradient blend, w: distortion
    performance_params: vec4<f32>,  // x: max particles, y: lod bias, z: quality, w: reserved
    _padding: vec2<f32>,
};

// Instance input data from vertex buffer
struct InstanceInput {
    @location(3) transform_0: vec4<f32>,
    @location(4) transform_1: vec4<f32>,
    @location(5) transform_2: vec4<f32>,
    @location(6) transform_3: vec4<f32>,
    @location(7) color: vec4<f32>,
    @location(8) uv_data: vec4<f32>,
    @location(9) custom_data: vec4<f32>,
};

struct WireframeSettings {
    color: vec4<f32>,
    thickness: f32,
    fade: f32,
}

struct TrailSettings {
    color: vec4<f32>,
    fade: f32,
    length: f32,
    spacing: f32,
    time: f32,
}

@group(0) @binding(0) var<uniform> material: MaterialParams;
@group(0) @binding(1) var base_texture: texture_2d<f32>;
@group(0) @binding(2) var base_sampler: sampler;
@group(0) @binding(3) var depth_texture: texture_depth_2d;
@group(2) @binding(0) var<uniform> wireframe: WireframeSettings;
@group(2) @binding(1) var<uniform> trail: TrailSettings;
@group(2) @binding(2) var trail_texture: texture_2d<f32>;
@group(2) @binding(3) var trail_sampler: sampler;

/// Helper functions for effects

/// Generate a pseudo-random number from a float seed
/// @param seed - Float value to generate random number from
/// @return Random value in range [0, 1]
fn rand(seed: f32) -> f32 {
    return fract(sin(seed * 78.233) * 43758.5453);
}

/// Generate 1D noise using linear interpolation
/// @param x - Input coordinate
/// @return Smooth noise value in range [0, 1]
fn noise(x: f32) -> f32 {
    let i = floor(x);
    let f = fract(x);
    return mix(rand(i), rand(i + 1.0), smoothstep(0.0, 1.0, f));
}

/// Apply heat distortion effect to UV coordinates
/// @param uv - Input UV coordinates
/// @param intensity - Strength of the distortion
/// @param time - Current time for animation
/// @return Distorted UV coordinates
fn heat_distortion(uv: vec2<f32>, intensity: f32, time: f32) -> vec2<f32> {
    let distortion = vec2<f32>(
        noise(uv.x * 10.0 + time),
        noise(uv.y * 10.0 + time)
    ) * 2.0 - 1.0;
    return uv + distortion * intensity;
}

/// Apply turbulent noise movement to UV coordinates
/// @param uv - Input UV coordinates
/// @param intensity - Strength of the turbulence
/// @param time - Current time for animation
/// @return Turbulent UV coordinates
fn turbulence(uv: vec2<f32>, intensity: f32, time: f32) -> vec2<f32> {
    let scale = vec2<f32>(2.0, 4.0);
    let offset = vec2<f32>(
        noise(time * 0.5) * 2.0 - 1.0,
        noise(time * 0.7) * 2.0 - 1.0
    );
    return uv + offset * intensity * scale;
}

/// Generate sparkle effect
/// @param uv - Input UV coordinates
/// @param intensity - Brightness of sparkles
/// @param time - Current time for animation
/// @return Sparkle intensity in range [0, 1]
fn sparkle(uv: vec2<f32>, intensity: f32, time: f32) -> f32 {
    let sparkle_pos = vec2<f32>(
        noise(uv.x * 20.0 + time * 2.0),
        noise(uv.y * 20.0 + time * 3.0)
    );
    let distance = length(fract(uv * 5.0) - sparkle_pos);
    return smoothstep(0.1, 0.0, distance) * intensity;
}

/// Generate Voronoi noise pattern
/// @param p - Input coordinates
/// @return Distance to nearest feature point
/// Used for cellular/organic patterns
fn voronoi_noise(p: vec2<f32>) -> f32 {
    let ip = floor(p);
    let fp = fract(p);
    
    var min_dist = 1.0;
    for(var y: i32 = -1; y <= 1; y++) {
        for(var x: i32 = -1; x <= 1; x++) {
            let offset = vec2<f32>(f32(x), f32(y));
            let h = rand(dot(ip + offset, vec2<f32>(127.1, 311.7)));
            let o = offset + vec2<f32>(h) - fp;
            min_dist = min(min_dist, dot(o, o));
        }
    }
    return sqrt(min_dist);
}

/// Generate Fractal Brownian Motion noise
/// @param p - Input coordinates
/// @param octaves - Number of noise layers to combine
/// @return Combined noise value
/// Creates natural-looking noise by layering multiple frequencies
fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    
    for(var i = 0; i < octaves; i++) {
        value += amplitude * noise(p * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    
    return value;
}

/// Blend between two colors using a gradient
/// @param t - Blend factor [0, 1]
/// @param start - Starting color
/// @param end - Ending color
/// @return Interpolated color
fn gradient_color(t: f32, start: vec4<f32>, end: vec4<f32>) -> vec4<f32> {
    return mix(start, end, smoothstep(0.0, 1.0, t));
}

fn calculate_trail(uv: vec2<f32>, trail_coord: f32) -> vec4<f32> {
    let trail_uv = vec2<f32>(
        uv.x,
        trail_coord / trail.length
    );
    
    let trail_sample = textureSample(trail_texture, trail_sampler, trail_uv);
    let trail_alpha = trail_sample.a * (1.0 - trail_coord / trail.length) * trail.fade;
    
    return vec4<f32>(trail.color.rgb, trail_alpha);
}

@vertex
fn vertex(
    input: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var output: VertexOutput;
    
    // Reconstruct instance transform matrix
    let instance_transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3
    );
    
    // Apply LOD with dynamic bias
    let lod_bias = material.performance_params.y;
    let size_scale = instance.custom_data.z * lod_bias; // size from instance data
    let scaled_position = input.position * size_scale;
    
    // Calculate world position using instance transform
    let world_pos = instance_transform * vec4<f32>(scaled_position, 1.0);
    
    // Early frustum culling optimization
    let clip_pos = material.view_proj * world_pos;
    
    // Frustum culling
    if (abs(clip_pos.x) > clip_pos.w * 1.1 || 
        abs(clip_pos.y) > clip_pos.w * 1.1 || 
        clip_pos.z < 0.0 || 
        clip_pos.z > clip_pos.w) {
        // Move particle off-screen
        output.clip_position = vec4<f32>(2.0, 2.0, 2.0, 1.0);
        return output;
    }
    
    // Calculate UV coordinates with instance data
    let uv_offset = instance.uv_data.xy;
    let uv_scale = instance.uv_data.zw;
    output.uv = input.uv * uv_scale + uv_offset;
    
    output.clip_position = clip_pos;
    output.world_position = world_pos.xyz;
    output.color = instance.color * material.color_tint;
    output.emission = material.emission_strength;
    output.particle_age = instance.custom_data.x;
    
    return output;
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    // Quality-based effect control
    let quality = material.performance_params.z;
    let effect_intensity = material.custom_params.y * quality;
    
    // Calculate base UV with optimized distortion
    var uv = input.uv;
    if (effect_intensity > 0.0 && quality > 0.5) {
        // Apply noise-based distortion
        let noise_uv = uv * material.effect_params.x + material.current_time * material.effect_params.y;
        let noise_value = fbm(noise_uv, i32(2.0 * quality));
        uv += noise_value * material.effect_params.w;
    }
    
    // Sample texture with gradient
    var color = textureSample(base_texture, base_sampler, uv);
    
    // Apply gradient coloring
    let gradient_t = input.particle_age * material.effect_params.z;
    let gradient = gradient_color(gradient_t, material.gradient_color_start, material.gradient_color_end);
    color *= gradient;
    
    // Apply instance color and emission
    color *= input.color;
    color.rgb *= input.emission;
    
    // Add voronoi noise pattern for high quality only
    if (quality > 0.8) {
        let voronoi = voronoi_noise(uv * material.effect_params.x + material.current_time * material.effect_params.y);
        color += vec4<f32>(voronoi * 0.2);
    }
    
    // Optimized soft particles with depth buffer
    if (material.depth_fade_distance > 0.0) {
        let scene_depth = textureSample(depth_texture, base_sampler, uv).r;
        let particle_depth = input.clip_position.z / input.clip_position.w;
        let depth_diff = scene_depth - particle_depth;
        let fade = smoothstep(0.0, material.depth_fade_distance, depth_diff);
        color.a *= fade;
    }
    
    // Distance-based fade with LOD
    let camera_distance = length(material.camera_position - input.world_position);
    let distance_fade = smoothstep(
        material.distance_fade_range.x,
        material.distance_fade_range.y,
        camera_distance
    );
    color.a *= 1.0 - distance_fade;
    
    // Alpha threshold with early discard
    if (color.a < material.alpha_threshold) {
        discard;
    }

    #ifdef TRAIL_MODE
    let trail_coord = fract(trail.time / trail.spacing);
    let trail_color = calculate_trail(uv, trail_coord);
    color = mix(color, trail_color, trail_color.a);
    #endif

    #ifdef WIREFRAME_MODE
    // Calculate distance to triangle edges for wireframe
    let barycentric = vec3<f32>(
        abs(dpdx(position.x)),
        abs(dpdy(position.y)),
        abs(dpdx(position.z))
    );
    let edge_dist = min(barycentric.x, min(barycentric.y, barycentric.z));
    let line_alpha = 1.0 - smoothstep(0.0, wireframe.thickness, edge_dist);
    let wire_color = mix(color, wireframe.color, line_alpha * wireframe.fade);
    color = wire_color;
    #endif

    return color;
} 