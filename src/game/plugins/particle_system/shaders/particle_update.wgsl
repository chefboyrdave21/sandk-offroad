// Particle data structure matching Rust definition
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

// System parameters
struct SimParams {
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
    atlas_config: vec2<f32>,
    anim_fps: f32,
    active_particles: u32,
    max_particles: u32,
    _padding: vec2<f32>,
}

// Buffers
@group(0) @binding(0)
var<storage, read> particles_in: array<Particle>;

@group(0) @binding(1)
var<storage, read_write> particles_out: array<Particle>;

@group(0) @binding(2)
var<uniform> params: SimParams;

// Random number generation
fn wang_hash(seed: u32) -> u32 {
    var s = seed;
    s = (s ^ 61u) ^ (s >> 16u);
    s *= 9u;
    s = s ^ (s >> 4u);
    s *= 0x27d4eb2du;
    s = s ^ (s >> 15u);
    return s;
}

fn rand(seed: ptr<function, u32>) -> f32 {
    *seed = wang_hash(*seed);
    return f32(*seed) / f32(0xFFFFFFFFu);
}

// Initialize a new particle
fn init_particle(index: u32, seed_base: u32) -> Particle {
    var seed = wang_hash(seed_base + index);
    var particle: Particle;
    
    // Generate random values
    let r1 = rand(&seed);
    let r2 = rand(&seed);
    let r3 = rand(&seed);
    let r4 = rand(&seed);
    let r5 = rand(&seed);
    
    // Set initial position (for now at origin, will be modified by emitter)
    particle.position = vec3<f32>(0.0);
    
    // Random velocity within range
    let vel_range = params.initial_velocity_max - params.initial_velocity_min;
    let vel_mag = params.initial_velocity_min + r1 * vel_range;
    let phi = r2 * 2.0 * 3.14159;
    let theta = r3 * 3.14159;
    particle.velocity = vec3<f32>(
        vel_mag * sin(theta) * cos(phi),
        vel_mag * sin(theta) * sin(phi),
        vel_mag * cos(theta)
    );
    
    // Random size within range
    let size_range = params.size_max - params.size_min;
    let size = params.size_min + r4 * size_range;
    let rot = params.rotation_min + r5 * (params.rotation_max - params.rotation_min);
    particle.size_and_rot = vec3<f32>(size, size, rot);
    
    // Set lifetime
    let life_range = params.lifetime_max - params.lifetime_min;
    particle.lifetime = vec2<f32>(0.0, params.lifetime_min + rand(&seed) * life_range);
    
    // Initial color
    particle.color = params.color_start;
    
    // Store random seed for future variation
    particle.random_seed = f32(seed);
    
    // Initial texture parameters (first frame)
    particle.tex_params = vec2<f32>(0.0, 0.0);
    
    // Padding
    particle._padding = vec2<f32>(0.0);
    
    return particle;
}

// Update an existing particle
fn update_particle(particle: Particle, dt: f32) -> Particle {
    var updated = particle;
    
    // Update position based on velocity
    updated.position = particle.position + particle.velocity * dt;
    
    // Apply forces
    updated.velocity = particle.velocity + params.forces * dt;
    
    // Update lifetime
    updated.lifetime.x = particle.lifetime.x + dt;
    
    // Interpolate color based on lifetime
    let life_factor = updated.lifetime.x / particle.lifetime.y;
    updated.color = mix(params.color_start, params.color_end, life_factor);
    
    // Update texture animation
    let frame_time = 1.0 / params.anim_fps;
    let total_frames = params.atlas_config.x * params.atlas_config.y;
    let current_frame = floor(updated.lifetime.x / frame_time) % total_frames;
    let frame_x = current_frame % params.atlas_config.x;
    let frame_y = floor(current_frame / params.atlas_config.x);
    updated.tex_params = vec2<f32>(frame_x, frame_y);
    
    return updated;
}

// Main compute shader
@compute @workgroup_size(256)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    
    // Don't process if beyond active particles
    if (index >= params.active_particles) {
        return;
    }
    
    var particle = particles_in[index];
    
    // Check if particle should die
    if (particle.lifetime.x >= particle.lifetime.y) {
        // Replace with new particle if we're still emitting
        if (index < params.emission_rate * params.delta_time) {
            particles_out[index] = init_particle(index, bitcast<u32>(particle.random_seed));
        }
        return;
    }
    
    // Update existing particle
    particles_out[index] = update_particle(particle, params.delta_time);
} 