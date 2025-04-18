// Vertex shader inputs
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

// Instance data from particle buffer
struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    color: vec4<f32>,
    size_and_rot: vec3<f32>,
    lifetime: vec2<f32>,
    random_seed: f32,
    tex_params: vec2<f32>,
    _padding: vec2<f32>,
}

// Material parameters
struct MaterialParams {
    // View-projection matrix
    view_proj: mat4x4<f32>,
    // Camera position
    camera_pos: vec3<f32>,
    // Atlas dimensions (columns, rows)
    atlas_dimensions: vec2<f32>,
    // Emission strength
    emission_strength: f32,
    // Alpha threshold for discard
    alpha_threshold: f32,
    // Blend mode (0: alpha, 1: additive, 2: premultiplied)
    blend_mode: u32,
    // LOD bias (-16 to +16)
    lod_bias: f32,
    // Padding
    _padding: vec2<f32>,
}

// Vertex shader outputs / Fragment shader inputs
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) emission: f32,
}

// Bindings
@group(0) @binding(0)
var<uniform> material: MaterialParams;

@group(0) @binding(1)
var base_texture: texture_2d<f32>;

@group(0) @binding(2)
var base_sampler: sampler;

@group(0) @binding(3)
var<storage, read> particles: array<Particle>;

@group(0) @binding(4)
var<storage, read> particle_indices: array<u32>;

// Vertex shader
@vertex
fn vertex(
    vertex: Vertex,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let particle_index = particle_indices[instance_index];
    let particle = particles[particle_index];
    
    // Calculate particle size and rotation
    let size = particle.size_and_rot.xy;
    let rotation = particle.size_and_rot.z;
    
    // Build rotation matrix
    let cos_rot = cos(rotation);
    let sin_rot = sin(rotation);
    let rot_matrix = mat2x2<f32>(
        cos_rot, -sin_rot,
        sin_rot, cos_rot
    );
    
    // Transform vertex position
    let rotated_pos = rot_matrix * (vertex.position.xy * size);
    let world_pos = vec3<f32>(rotated_pos, 0.0) + particle.position;
    
    // Calculate UVs for texture atlas
    let atlas_x = particle.tex_params.x;
    let atlas_y = particle.tex_params.y;
    let uv_scale = 1.0 / material.atlas_dimensions;
    let uv_offset = vec2<f32>(atlas_x, atlas_y) * uv_scale;
    let final_uv = vertex.uv * uv_scale + uv_offset;
    
    // Transform normal
    let world_normal = normalize(
        mat3x3<f32>(cos_rot, -sin_rot, 0.0,
                    sin_rot, cos_rot, 0.0,
                    0.0, 0.0, 1.0) * vertex.normal
    );
    
    var output: VertexOutput;
    output.clip_position = material.view_proj * vec4<f32>(world_pos, 1.0);
    output.world_position = world_pos;
    output.world_normal = world_normal;
    output.uv = final_uv;
    output.color = particle.color;
    output.emission = material.emission_strength;
    
    return output;
}

// Fragment shader
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture with LOD bias for better filtering
    let tex_color = textureSampleBias(
        base_texture,
        base_sampler,
        in.uv,
        material.lod_bias
    );
    
    // Calculate final color based on blend mode
    var final_color = tex_color * in.color;
    
    // Apply emission
    final_color.rgb += final_color.rgb * in.emission;
    
    // Handle different blend modes
    switch material.blend_mode {
        case 0u: { // Alpha blending
            // No changes needed, using alpha blend state
        }
        case 1u: { // Additive blending
            final_color.rgb *= final_color.a;
            final_color.a = 0.0;
        }
        case 2u: { // Premultiplied alpha
            final_color.rgb *= final_color.a;
        }
        default: {}
    }
    
    // Discard fully transparent pixels
    if final_color.a < material.alpha_threshold {
        discard;
    }
    
    return final_color;
} 