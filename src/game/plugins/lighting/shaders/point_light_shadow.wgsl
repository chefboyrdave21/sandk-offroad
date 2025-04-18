struct ViewUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    world_position: vec4<f32>,
    near: f32,
    far: f32,
}

@group(0) @binding(0)
var<uniform> view: ViewUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
}

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform vertex position to world space
    let world_pos = vec4<f32>(vertex.position, 1.0);
    out.world_position = world_pos.xyz;
    
    // Transform normal to world space
    out.world_normal = normalize((view.inverse_view * vec4<f32>(vertex.normal, 0.0)).xyz);
    
    // Transform position to clip space
    out.clip_position = view.view_proj * world_pos;
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) f32 {
    // Calculate distance from light to fragment
    let light_to_frag = length(in.world_position - view.world_position.xyz);
    
    // Normalize distance to [0, 1] range using near/far planes
    let depth = (light_to_frag - view.near) / (view.far - view.near);
    
    return depth;
} 