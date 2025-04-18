// Particle sorting compute shader using bitonic sort
// This shader performs a parallel bitonic sort on particle indices based on depth

struct SortParams {
    // Camera position for depth calculation
    camera_pos: vec3<f32>,
    // Number of active particles
    particle_count: u32,
    // Current sort stage
    k: u32,
    // Current sort step
    j: u32,
    // Whether to sort ascending (0) or descending (1)
    sort_descending: u32,
}

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

@group(0) @binding(0)
var<storage, read> particles: array<Particle>;

@group(0) @binding(1)
var<storage, read_write> indices: array<u32>;

@group(0) @binding(2)
var<uniform> params: SortParams;

// Calculate particle depth relative to camera
fn calculate_depth(particle_pos: vec3<f32>) -> f32 {
    let to_camera = particle_pos - params.camera_pos;
    return dot(to_camera, to_camera); // Use squared distance for better precision
}

// Compare two particles based on depth
fn compare_particles(a: u32, b: u32) -> bool {
    let depth_a = calculate_depth(particles[a].position);
    let depth_b = calculate_depth(particles[b].position);
    
    if (params.sort_descending == 0u) {
        return depth_a < depth_b;
    } else {
        return depth_a > depth_b;
    }
}

// Swap two indices if they are in the wrong order
fn compare_and_swap(i: u32, j: u32) {
    if (i >= params.particle_count || j >= params.particle_count) {
        return;
    }
    
    if (compare_particles(indices[i], indices[j]) != (params.sort_descending == 0u)) {
        let temp = indices[i];
        indices[i] = indices[j];
        indices[j] = temp;
    }
}

// Main compute shader entry point
@compute @workgroup_size(256)
fn sort(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let thread_id = global_id.x;
    let i = thread_id;
    
    // Calculate indices for bitonic sort
    let ixj = i ^ params.j;
    
    // Only swap if ixj > i
    if (ixj > i) {
        compare_and_swap(i, ixj);
    }
} 