#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::utils

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    color: vec4<f32>,
    size: f32,
    rotation: f32,
    lifetime: f32,
    age: f32,
    atlas_index: u32,
}

struct SimulationParams {
    delta_time: f32,
    emission_rate: f32,
    initial_velocity_min: f32,
    initial_velocity_max: f32,
    size_min: f32,
    size_max: f32,
    rotation_min: f32,
    rotation_max: f32,
    lifetime_min: f32,
    lifetime_max: f32,
    color_start: vec4<f32>,
    color_end: vec4<f32>,
    forces: vec3<f32>,
    atlas_config: vec2<u32>, // x: cols, y: rows
    anim_fps: f32,
    active_particles: u32,
    max_particles: u32,
    _padding: vec2<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) instance_position: vec3<f32>,
    @location(4) instance_rotation: vec4<f32>,
    @location(5) instance_scale: vec3<f32>,
    @location(6) instance_color: vec4<f32>,
    @location(7) instance_velocity: vec3<f32>,
    @location(8) instance_age: f32,
    @location(9) instance_lifetime: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) velocity: vec3<f32>,
    @location(5) particle_age: f32,
    @location(6) particle_lifetime: f32>,
}

struct AtlasConfig {
    grid_size: vec2<f32>,
    frame_time: f32,
    total_frames: u32,
    current_time: f32,
}

struct LodSettings {
    start_distance: f32,
    end_distance: f32,
    min_size: f32,
    max_size: f32,
    enabled: u32,
}

struct ParticleProperties {
    emission_strength: f32,
    soft_particles: f32,
    distortion_amount: f32,
    normal_strength: f32,
    color_tint: vec4<f32>,
    custom_params: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> particles_in: array<Particle>;
@group(0) @binding(1) var<storage, read_write> particles_out: array<Particle>;
@group(0) @binding(2) var<uniform> params: SimulationParams;
@group(1) @binding(0) var<uniform> atlas_config: AtlasConfig;
@group(1) @binding(1) var<uniform> properties: ParticleProperties;
@group(1) @binding(2) var<uniform> lod_settings: LodSettings;
@group(1) @binding(3) var diffuse_texture: texture_2d<f32>;
@group(1) @binding(4) var diffuse_sampler: sampler;
@group(1) @binding(5) var normal_texture: texture_2d<f32>;
@group(1) @binding(6) var normal_sampler: sampler;

// Random number generation
fn rand_float(seed: vec2<f32>) -> f32 {
    return fract(sin(dot(seed, vec2(12.9898, 78.233))) * 43758.5453);
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + t * (b - a);
}

fn lerp_vec4(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    return a + t * (b - a);
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.active_particles) {
        return;
    }

    var particle = particles_in[index];

    // Update age
    particle.age += params.delta_time;
    let life_ratio = particle.age / particle.lifetime;

    // Check if particle should die
    if (life_ratio >= 1.0) {
        // Respawn particle
        let rand_seed = vec2<f32>(f32(index), params.delta_time);
        
        // Position (inherit from dead particle)
        particle.position = particle.position;
        
        // Velocity
        let speed = lerp(
            params.initial_velocity_min,
            params.initial_velocity_max,
            rand_float(rand_seed + vec2<f32>(1.0, 1.0))
        );
        let angle = rand_float(rand_seed + vec2<f32>(2.0, 2.0)) * 6.28318530718; // 2*PI
        particle.velocity = vec3<f32>(
            cos(angle) * speed,
            rand_float(rand_seed + vec2<f32>(3.0, 3.0)) * speed,
            sin(angle) * speed
        );
        
        // Other properties
        particle.size = lerp(
            params.size_min,
            params.size_max,
            rand_float(rand_seed + vec2<f32>(4.0, 4.0))
        );
        particle.rotation = lerp(
            params.rotation_min,
            params.rotation_max,
            rand_float(rand_seed + vec2<f32>(5.0, 5.0))
        );
        particle.lifetime = lerp(
            params.lifetime_min,
            params.lifetime_max,
            rand_float(rand_seed + vec2<f32>(6.0, 6.0))
        );
        particle.age = 0.0;
        particle.color = params.color_start;
        
        // Animation
        if (params.atlas_config.x > 1u || params.atlas_config.y > 1u) {
            let total_frames = params.atlas_config.x * params.atlas_config.y;
            particle.atlas_index = u32(rand_float(rand_seed + vec2<f32>(7.0, 7.0)) * f32(total_frames));
        } else {
            particle.atlas_index = 0u;
        }
    } else {
        // Update existing particle
        particle.velocity += params.forces * params.delta_time;
        particle.position += particle.velocity * params.delta_time;
        
        // Interpolate color
        particle.color = lerp_vec4(params.color_start, params.color_end, life_ratio);
        
        // Update animation
        if (params.atlas_config.x > 1u || params.atlas_config.y > 1u) {
            let total_frames = params.atlas_config.x * params.atlas_config.y;
            let frame = u32(particle.age * params.anim_fps) % total_frames;
            particle.atlas_index = frame;
        }
    }

    particles_out[index] = particle;
}

// Helper function to apply quaternion rotation
fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let qv = vec3<f32>(q.x, q.y, q.z);
    let uv = cross(qv, v);
    let uuv = cross(qv, uv);
    return v + ((uv * q.w) + uuv) * 2.0;
}

// Helper function to calculate UV coordinates for texture atlas
fn calculate_atlas_uv(base_uv: vec2<f32>, current_time: f32, frame_time: f32, grid_size: vec2<f32>, total_frames: u32) -> vec2<f32> {
    let frame = u32(current_time / frame_time) % total_frames;
    let frame_x = frame % u32(grid_size.x);
    let frame_y = frame / u32(grid_size.x);
    
    let frame_size = vec2<f32>(1.0) / grid_size;
    let frame_offset = vec2<f32>(
        f32(frame_x) * frame_size.x,
        f32(frame_y) * frame_size.y
    );
    
    return frame_offset + (base_uv * frame_size);
}

// Helper functions for effects
fn apply_heat_distortion(uv: vec2<f32>, intensity: f32, time: f32) -> vec2<f32> {
    let noise = sin(uv.x * 10.0 + time) * cos(uv.y * 10.0 + time) * 0.01;
    return uv + vec2<f32>(noise, noise) * intensity;
}

fn apply_sparkle(color: vec4<f32>, uv: vec2<f32>, intensity: f32, time: f32) -> vec4<f32> {
    let sparkle = sin(uv.x * 20.0 + time) * cos(uv.y * 20.0 + time) * 0.5 + 0.5;
    return color + vec4<f32>(sparkle * intensity);
}

fn apply_turbulence(uv: vec2<f32>, intensity: f32, time: f32) -> vec2<f32> {
    let noise_x = sin(uv.y * 5.0 + time) * 0.01;
    let noise_y = cos(uv.x * 5.0 + time) * 0.01;
    return uv + vec2<f32>(noise_x, noise_y) * intensity;
}

fn calculate_lod_factor(world_pos: vec3<f32>) -> f32 {
    if (lod_settings.enabled == 0u) {
        return 1.0;
    }
    
    let camera_pos = view.world_position;
    let distance = length(world_pos - camera_pos);
    
    if (distance <= lod_settings.start_distance) {
        return 1.0;
    }
    if (distance >= lod_settings.end_distance) {
        return 0.0;
    }
    
    return 1.0 - (distance - lod_settings.start_distance) / 
        (lod_settings.end_distance - lod_settings.start_distance);
}

fn calculate_lod_size(base_size: f32, lod_factor: f32) -> f32 {
    return mix(lod_settings.min_size, lod_settings.max_size, lod_factor) * base_size;
}

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Calculate LOD factor
    let lod_factor = calculate_lod_factor(input.instance_position);
    
    // Apply instance transform with LOD size adjustment
    let particle_size = calculate_lod_size(input.instance_scale.x, lod_factor);
    let scaled_position = input.position * vec3<f32>(particle_size);
    let rotated_position = quat_rotate(input.instance_rotation, scaled_position);
    let world_position = rotated_position + input.instance_position;
    
    // Transform to clip space
    output.clip_position = view.view_proj * vec4<f32>(world_position, 1.0);
    output.world_position = world_position;
    
    // Transform normal
    output.world_normal = quat_rotate(input.instance_rotation, input.normal);
    
    // Calculate UV coordinates for texture atlas
    var uv = calculate_atlas_uv(
        input.uv,
        atlas_config.current_time,
        atlas_config.frame_time,
        atlas_config.grid_size,
        atlas_config.total_frames
    );
    
    // Apply turbulence if enabled
    if (properties.custom_params.y > 0.0) {
        uv = apply_turbulence(uv, properties.custom_params.y, atlas_config.current_time);
    }
    
    output.uv = uv;
    
    // Pass through instance data with color tinting
    output.color = input.instance_color * properties.color_tint;
    output.velocity = input.instance_velocity;
    output.particle_age = input.instance_age;
    output.particle_lifetime = input.instance_lifetime;
    
    return output;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample textures
    var color = input.color;
    
    // Apply texture if using atlas
    if (atlas_config.total_frames > 1u) {
        var uv = input.uv;
        
        // Apply heat distortion if enabled
        if (properties.custom_params.x > 0.0) {
            uv = apply_heat_distortion(uv, properties.custom_params.x, atlas_config.current_time);
        }
        
        color *= textureSample(diffuse_texture, diffuse_sampler, uv);
    }
    
    // Apply normal mapping
    var normal = input.world_normal;
    if (properties.normal_strength > 0.0) {
        let tangent_normal = textureSample(normal_texture, normal_sampler, input.uv).xyz * 2.0 - 1.0;
        normal = normalize(mix(normal, tangent_normal, properties.normal_strength));
    }
    
    // Apply soft particles
    if (properties.soft_particles > 0.0) {
        let depth = input.clip_position.z / input.clip_position.w;
        let scene_depth = bevy_pbr::get_depth(input.clip_position.xy);
        let soft_factor = saturate((scene_depth - depth) / properties.soft_particles);
        color.a *= soft_factor;
    }
    
    // Apply sparkle effect if enabled
    if (properties.custom_params.z > 0.0) {
        color = apply_sparkle(color, input.uv, properties.custom_params.z, atlas_config.current_time);
    }
    
    // Apply emission
    color.rgb *= properties.emission_strength;
    
    // Apply distortion based on velocity
    if (properties.distortion_amount > 0.0) {
        let velocity_length = length(input.velocity);
        let distortion = velocity_length * properties.distortion_amount;
        // Apply screen-space distortion effect
        color.rgb += distortion * 0.1;
    }
    
    // Apply age-based fade
    let life_factor = input.particle_age / input.particle_lifetime;
    color.a *= 1.0 - life_factor;
    
    // Apply LOD-based fade
    let lod_factor = calculate_lod_factor(input.world_position);
    color.a *= lod_factor;
    
    return color;
} 