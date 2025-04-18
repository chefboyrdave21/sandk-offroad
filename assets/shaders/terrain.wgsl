struct TerrainMaterial {
    height_scale: f32,
    texture_scale: f32,
    normal_strength: f32,
};

@group(1) @binding(0)
var height_map: texture_2d<f32>;
@group(1) @binding(1)
var height_sampler: sampler;

@fragment
fn fragment(
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    let height = textureSample(height_map, height_sampler, uv).r;
    let color = mix(
        vec3<f32>(0.2, 0.5, 0.1), // grass color
        vec3<f32>(0.6, 0.5, 0.4), // dirt color
        smoothstep(0.3, 0.7, height)
    );
    
    return vec4<f32>(color, 1.0);
} 