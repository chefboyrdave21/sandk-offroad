// Vertex shader inputs
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

// Vertex shader outputs / Fragment shader inputs
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

// View-projection uniform
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_position: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Material uniforms
struct TrailMaterial {
    fade_start: f32,
    fade_end: f32,
    emission_strength: f32,
    soft_particles: f32,
};
@group(1) @binding(0)
var<uniform> material: TrailMaterial;

// Texture bindings
@group(1) @binding(1)
var trail_texture: texture_2d<f32>;
@group(1) @binding(2)
var trail_sampler: sampler;

// Vertex shader
@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to clip space
    out.clip_position = camera.view_proj * vec4<f32>(vertex.position, 1.0);
    
    // Pass through world space data
    out.world_position = vertex.position;
    out.world_normal = vertex.normal;
    out.uv = vertex.uv;
    out.color = vertex.color;
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    var color = in.color;
    if (material.fade_start > 0.0) {
        let view_distance = length(camera.view_position.xyz - in.world_position);
        let fade_factor = smoothstep(material.fade_end, material.fade_start, view_distance);
        color = color * fade_factor;
    }
    
    // Apply texture if available
    let tex_color = textureSample(trail_texture, trail_sampler, in.uv);
    color = color * tex_color;
    
    // Apply emission
    color = color * material.emission_strength;
    
    // Apply soft particles
    if (material.soft_particles > 0.0) {
        // TODO: Implement soft particles using depth texture
    }
    
    return color;
} 