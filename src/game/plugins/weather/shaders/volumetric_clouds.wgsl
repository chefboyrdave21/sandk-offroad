#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

struct CloudParams {
    density: f32,
    coverage: f32,
    altitude: f32,
    thickness: f32,
    wind_direction: vec2<f32>,
    wind_speed: f32,
    precipitation_threshold: f32,
    time: f32,
}

@group(1) @binding(0)
var<uniform> params: CloudParams;

@group(1) @binding(1)
var base_shape_texture: texture_3d<f32>;
@group(1) @binding(2)
var base_shape_sampler: sampler;

@group(1) @binding(3)
var detail_texture: texture_3d<f32>;
@group(1) @binding(4)
var detail_sampler: sampler;

@group(1) @binding(5)
var weather_texture: texture_3d<f32>;
@group(1) @binding(6)
var weather_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_position = (mesh.model * vec4<f32>(vertex.position, 1.0)).xyz;
    out.world_normal = normalize((mesh.model * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.clip_position = view.view_proj * vec4<f32>(out.world_position, 1.0);
    out.uv = vertex.uv;
    return out;
}

// Helper function to sample 3D noise with wind offset
fn sample_noise(pos: vec3<f32>, scale: f32, offset: vec2<f32>) -> f32 {
    let wind_offset = vec3<f32>(offset * params.wind_speed * params.time, 0.0);
    let sample_pos = (pos * scale + wind_offset) * 0.001;
    return textureSample(base_shape_texture, base_shape_sampler, sample_pos).r;
}

// Calculate cloud density at a point
fn get_cloud_density(pos: vec3<f32>) -> f32 {
    // Apply wind displacement
    let wind_dir = vec3<f32>(params.wind_direction.x, 0.0, params.wind_direction.y);
    let wind_offset = wind_dir * params.wind_speed * params.time;
    
    // Sample base shape noise
    var density = sample_noise(pos + wind_offset, 1.0, params.wind_direction);
    
    // Add detail noise
    let detail = sample_noise(pos + wind_offset, 4.0, params.wind_direction * 2.0);
    density = density + detail * 0.2;
    
    // Sample weather influence
    let weather = textureSample(weather_texture, weather_sampler, pos * 0.0001).r;
    
    // Apply coverage and density controls
    density = smoothstep(1.0 - params.coverage, 1.0, density) * params.density;
    
    // Height falloff
    let height_percent = (pos.y - params.altitude) / params.thickness;
    let height_falloff = 1.0 - smoothstep(0.0, 1.0, abs(height_percent));
    
    return density * height_falloff * weather;
}

// Ray marching through cloud volume
fn ray_march(ray_origin: vec3<f32>, ray_dir: vec3<f32>, start_dist: f32, end_dist: f32) -> vec4<f32> {
    let steps = 64;
    let step_size = (end_dist - start_dist) / f32(steps);
    
    var total_density = 0.0;
    var transmittance = 1.0;
    var light_energy = vec3<f32>(0.0);
    
    // Sun light direction (from view.sun_direction if available)
    let sun_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    
    for(var i = 0; i < steps; i = i + 1) {
        let current_dist = start_dist + step_size * f32(i);
        let pos = ray_origin + ray_dir * current_dist;
        
        // Skip if outside cloud layer
        if (pos.y < params.altitude || pos.y > params.altitude + params.thickness) {
            continue;
        }
        
        let density = get_cloud_density(pos);
        if (density > 0.0) {
            // Light march towards sun
            let light_density = get_cloud_density(pos + sun_dir * 100.0);
            let light_transmittance = exp(-light_density * 0.1);
            
            // Accumulate light contribution
            let ambient_light = vec3<f32>(0.2, 0.2, 0.3);
            let sun_light = vec3<f32>(1.0, 0.9, 0.7) * light_transmittance;
            let cloud_light = ambient_light + sun_light;
            
            light_energy += cloud_light * density * transmittance * step_size;
            transmittance *= exp(-density * step_size);
            
            // Early exit if nearly opaque
            if (transmittance < 0.01) {
                break;
            }
        }
    }
    
    return vec4<f32>(light_energy, 1.0 - transmittance);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray_origin = view.world_position.xyz;
    let ray_dir = normalize(in.world_position - ray_origin);
    
    // Calculate intersection with cloud layer
    let height_above_clouds = ray_origin.y - params.altitude;
    let start_dist = max(0.0, (params.altitude - ray_origin.y) / ray_dir.y);
    let end_dist = min(10000.0, (params.altitude + params.thickness - ray_origin.y) / ray_dir.y);
    
    // Skip if ray doesn't intersect cloud layer
    if (start_dist >= end_dist) {
        return vec4<f32>(0.0);
    }
    
    // Ray march through clouds
    let cloud_color = ray_march(ray_origin, ray_dir, start_dist, end_dist);
    
    // Apply atmospheric scattering (simplified)
    let atmosphere = vec3<f32>(0.5, 0.6, 1.0);
    let final_color = mix(cloud_color.rgb, atmosphere, 1.0 - cloud_color.a);
    
    return vec4<f32>(final_color, cloud_color.a);
} 