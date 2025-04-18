struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) velocity: vec3<f32>,
    @location(5) custom_data: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) velocity: vec3<f32>,
    @location(5) custom_data: vec4<f32>,
};

struct TrailMaterial {
    @location(0) color_multiplier: vec4<f32>,
    @location(1) use_texture: u32,
    @location(2) effect_strength: f32,
    @location(3) time: f32,
};

@group(0) @binding(0) var<uniform> view: mat4x4<f32>;
@group(0) @binding(1) var<uniform> projection: mat4x4<f32>;
@group(1) @binding(0) var<uniform> material: TrailMaterial;
@group(1) @binding(1) var trail_texture: texture_2d<f32>;
@group(1) @binding(2) var trail_sampler: sampler;

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Apply any vertex deformation based on velocity or custom data
    var modified_position = input.position;
    
    // Example: Add subtle wave effect based on velocity
    let wave_strength = length(input.velocity) * 0.1;
    modified_position += input.normal * sin(input.custom_data.x * 5.0 + input.uv.x * 10.0) * wave_strength;
    
    let world_position = modified_position;
    output.clip_position = projection * view * vec4<f32>(world_position, 1.0);
    output.world_position = world_position;
    output.world_normal = input.normal;
    output.uv = input.uv;
    output.color = input.color;
    output.velocity = input.velocity;
    output.custom_data = input.custom_data;
    
    return output;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = input.color * material.color_multiplier;
    
    // Apply texture if enabled
    if material.use_texture == 1u {
        let tex_color = textureSample(trail_texture, trail_sampler, input.uv);
        color *= tex_color;
    }
    
    // Apply velocity-based effects
    let speed = length(input.velocity);
    let speed_glow = smoothstep(0.5, 2.0, speed);
    
    // Add glow effect based on velocity
    color += vec4<f32>(1.0, 0.8, 0.4, 0.0) * speed_glow * material.effect_strength;
    
    // Add time-based effects using custom data
    let time_effect = sin(input.custom_data.x * 4.0 + material.time * 2.0) * 0.5 + 0.5;
    color += vec4<f32>(0.2, 0.4, 1.0, 0.0) * time_effect * material.effect_strength;
    
    // Ensure alpha doesn't exceed 1.0
    color.a = min(color.a, 1.0);
    
    return color;
} 