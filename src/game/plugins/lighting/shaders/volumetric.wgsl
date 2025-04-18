struct ViewUniform {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    position: vec3<f32>,
    time: f32,
}

struct Light {
    position: vec3<f32>,
    _pad0: f32,
    color: vec3<f32>,
    intensity: f32,
}

@group(0) @binding(0)
var<uniform> view: ViewUniform;

@group(0) @binding(1)
var volume_texture: texture_storage_3d<rgba16float, read_write>;

@group(0) @binding(2)
var<storage, read> lights: array<Light>;

const VOLUME_SIZE = vec3<u32>(128u, 128u, 64u);
const WORKGROUP_SIZE = vec3<u32>(16u, 16u, 4u);
const PI = 3.14159265359;

// Phase function for light scattering (Henyey-Greenstein)
fn phase_function(cos_theta: f32, g: f32) -> f32 {
    let denom = 1.0 + g * g - 2.0 * g * cos_theta;
    return (1.0 - g * g) / (4.0 * PI * denom * sqrt(denom));
}

// Convert world position to volume texture coordinates
fn world_to_volume(world_pos: vec3<f32>, min_bounds: vec3<f32>, max_bounds: vec3<f32>) -> vec3<f32> {
    return (world_pos - min_bounds) / (max_bounds - min_bounds);
}

// Ray march through volume and accumulate light scattering
fn ray_march(origin: vec3<f32>, direction: vec3<f32>, min_bounds: vec3<f32>, max_bounds: vec3<f32>, num_steps: u32) -> vec4<f32> {
    let step_size = 1.0 / f32(num_steps);
    var result = vec4<f32>(0.0);
    var transmittance = 1.0;
    
    for(var i = 0u; i < num_steps; i = i + 1u) {
        let t = f32(i) * step_size;
        let pos = origin + direction * t;
        let volume_pos = world_to_volume(pos, min_bounds, max_bounds);
        
        // Skip if outside volume bounds
        if any(volume_pos < vec3<f32>(0.0)) || any(volume_pos > vec3<f32>(1.0)) {
            continue;
        }
        
        let volume_coords = vec3<u32>(
            u32(volume_pos.x * f32(VOLUME_SIZE.x)),
            u32(volume_pos.y * f32(VOLUME_SIZE.y)),
            u32(volume_pos.z * f32(VOLUME_SIZE.z))
        );
        
        // Get density at current position
        let density = textureLoad(volume_texture, volume_coords).a;
        if density <= 0.0 {
            continue;
        }
        
        // Calculate light contribution from each light
        var light_contribution = vec3<f32>(0.0);
        for(var light_idx = 0u; light_idx < arrayLength(&lights); light_idx = light_idx + 1u) {
            let light = lights[light_idx];
            let light_dir = normalize(light.position - pos);
            let dist_to_light = distance(light.position, pos);
            
            // Calculate phase function
            let cos_theta = dot(direction, light_dir);
            let phase = phase_function(cos_theta, 0.3); // g = 0.3 for forward scattering
            
            // Light attenuation
            let attenuation = 1.0 / (1.0 + 0.1 * dist_to_light + 0.01 * dist_to_light * dist_to_light);
            
            light_contribution += light.color * light.intensity * phase * attenuation;
        }
        
        // Accumulate light contribution with density
        let scatter_amount = density * step_size;
        result.rgb += light_contribution * transmittance * scatter_amount;
        transmittance *= 1.0 - scatter_amount;
        
        if transmittance < 0.01 {
            break;
        }
    }
    
    result.a = 1.0 - transmittance;
    return result;
}

@compute @workgroup_size(16, 16, 4)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Skip if outside volume dimensions
    if any(global_id >= VOLUME_SIZE) {
        return;
    }
    
    // Calculate world space position
    let volume_pos = vec3<f32>(
        f32(global_id.x) / f32(VOLUME_SIZE.x),
        f32(global_id.y) / f32(VOLUME_SIZE.y),
        f32(global_id.z) / f32(VOLUME_SIZE.z)
    );
    
    // Define volume bounds in world space
    let min_bounds = vec3<f32>(-50.0, -50.0, -25.0);
    let max_bounds = vec3<f32>(50.0, 50.0, 25.0);
    let world_pos = mix(min_bounds, max_bounds, volume_pos);
    
    // Calculate view direction
    let view_dir = normalize(world_pos - view.position);
    
    // Ray march through volume
    let result = ray_march(world_pos, view_dir, min_bounds, max_bounds, 32u);
    
    // Store result in volume texture
    textureStore(volume_texture, global_id, result);
} 