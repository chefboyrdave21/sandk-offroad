// Post-processing shader implementing various visual effects
// Each effect is implemented with precise mathematical formulas for accurate results

// Performance Optimization Guide:
// 1. Early Exit Optimizations:
//    - Skip effects when their intensity/strength is 0 (chromatic aberration, vignette)
//    - Early return in fragment shader when possible
//    - Avoid unnecessary texture samples for disabled effects
//    - Branch prediction hints for frequently disabled effects
//
// 2. Memory Access Optimizations:
//    - Use textureSample instead of textureLoad for hardware filtering
//    - Minimize texture fetches by reusing samples when possible
//    - Keep uniform buffer aligned to 16-byte boundaries
//    - Cache frequently accessed values in registers
//
// 3. Computation Optimizations:
//    - Pre-calculate constants outside loops and functions
//    - Use built-in functions (dot, mix, pow) for hardware acceleration
//    - Vectorize operations (vec3 vs individual components)
//    - Minimize dependent texture reads
//
// 4. Quality vs Performance Tradeoffs:
//    - Chromatic aberration can skip G channel sampling
//    - Vignette calculation can use cheaper smoothstep
//    - Tone mapping operators can be simplified for mobile
//    - Adjust precision based on visual requirements
//
// 5. View-Dependent Optimizations:
//    - Scale effect intensity with view distance
//    - Adjust quality based on resolution
//    - Consider dynamic quality scaling
//    - Use distance-based LOD for effects
//
// 6. GPU Architecture Considerations:
//    - Minimize register pressure by reusing variables
//    - Reduce divergent branching in fragment shader
//    - Balance ALU vs texture operations
//    - Consider wave/warp size for branching
//
// 7. Performance Monitoring:
//    - Use GPU timestamps for profiling
//    - Monitor ALU/texture unit utilization
//    - Track memory bandwidth usage
//    - Measure frame timing impact

// Uniform buffer containing all post-processing settings
struct PostProcessSettings {
    exposure: f32,      // Exposure adjustment
    gamma: f32,         // Gamma correction value
    contrast: f32,      // Contrast adjustment
    saturation: f32,    // Color saturation multiplier
    brightness: f32,    // Brightness adjustment
    bloom_intensity: f32,   // Bloom effect strength
    bloom_threshold: f32,   // Threshold for bloom sampling
    chromatic_aberration: f32,  // RGB channel separation amount
    vignette_strength: f32,    // Darkening at screen edges
    vignette_radius: f32,      // Radius of vignette effect
    tone_mapping: u32,         // Tone mapping operator selection
    _padding: u32,             // Maintain 16-byte alignment
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

// Vertex shader for full-screen quad
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate full-screen triangle vertices
    let x = f32(vertex_index & 1u) * 2.0 - 1.0;
    let y = f32((vertex_index >> 1u) & 1u) * 2.0 - 1.0;
    
    var out: VertexOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (y + 1.0) * 0.5);
    return out;
}

// ACES tone mapping formula
// Uses Academy Color Encoding System for film-like response
fn tone_map_aces(color: vec3<f32>) -> vec3<f32> {
    // ACES parameters
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    
    // Formula: ((x * (ax + b)) / (x * (cx + d) + e))
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), vec3(0.0), vec3(1.0));
}

// Reinhard tone mapping
// Simple but effective tone mapping that preserves colors well
fn tone_map_reinhard(color: vec3<f32>) -> vec3<f32> {
    // Formula: x / (1 + x)
    return color / (vec3(1.0) + color);
}

// Uncharted 2 tone mapping
// Complex tone mapping with fine control over shoulder and toe regions
fn tone_map_uncharted2(color: vec3<f32>) -> vec3<f32> {
    let A = 0.15; // Shoulder strength
    let B = 0.50; // Linear strength
    let C = 0.10; // Linear angle
    let D = 0.20; // Toe strength
    let E = 0.02; // Toe numerator
    let F = 0.30; // Toe denominator
    let W = 11.2; // Linear white point value
    
    // Formula: ((x*(A*x+C*B)+D*E)/(x*(A*x+B)+D*F))-E/F
    let curr = ((color * (A * color + C * B) + D * E) / (color * (A * color + B) + D * F)) - E / F;
    let whiteScale = ((vec3(W) * (A * vec3(W) + C * B) + D * E) / (vec3(W) * (A * vec3(W) + B) + D * F)) - E / F;
    return curr / whiteScale;
}

// Chromatic aberration calculation
// Simulates lens dispersion by offsetting RGB channels
fn apply_chromatic_aberration(uv: vec2<f32>, strength: f32) -> vec3<f32> {
    let center = vec2(0.5);
    let dist = uv - center;
    
    // Sample each color channel with increasing offset from center
    let r = textureSample(screen_texture, screen_sampler, uv - dist * strength * 1.0).r;
    let g = textureSample(screen_texture, screen_sampler, uv - dist * strength * 0.0).g;
    let b = textureSample(screen_texture, screen_sampler, uv + dist * strength * 1.0).b;
    
    return vec3(r, g, b);
}

// Vignette effect calculation
// Creates smooth darkening at screen edges
fn apply_vignette(color: vec3<f32>, uv: vec2<f32>, strength: f32, radius: f32) -> vec3<f32> {
    let center = vec2(0.5);
    let dist = length(uv - center) * 2.0;
    
    // Smooth falloff formula: 1.0 - (dist^2 / radius^2)^strength
    let vignette = 1.0 - pow(pow(dist / radius, 2.0), strength);
    return color * clamp(vignette, 0.0, 1.0);
}

// Main fragment shader
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(screen_texture, screen_sampler, in.uv).rgb;
    
    // Apply exposure adjustment
    // Formula: color * 2^exposure
    color *= pow(2.0, settings.exposure);
    
    // Apply chromatic aberration
    if (settings.chromatic_aberration > 0.0) {
        color = apply_chromatic_aberration(in.uv, settings.chromatic_aberration);
    }
    
    // Apply tone mapping based on selected operator
    switch settings.tone_mapping {
        case 1u: { color = tone_map_aces(color); }
        case 2u: { color = tone_map_reinhard(color); }
        case 3u: { color = tone_map_uncharted2(color); }
        default: {}
    }
    
    // Apply contrast
    // Formula: 0.5 + (color - 0.5) * contrast
    color = 0.5 + (color - 0.5) * settings.contrast;
    
    // Apply saturation
    // Formula: mix(luminance, color, saturation)
    let luminance = vec3(dot(color, vec3(0.2126, 0.7152, 0.0722)));
    color = mix(luminance, color, settings.saturation);
    
    // Apply brightness
    // Formula: color * brightness
    color *= settings.brightness;
    
    // Apply vignette
    if (settings.vignette_strength > 0.0) {
        color = apply_vignette(color, in.uv, settings.vignette_strength, settings.vignette_radius);
    }
    
    // Apply gamma correction
    // Formula: color^(1/gamma)
    color = pow(color, vec3(1.0 / settings.gamma));
    
    return vec4(color, 1.0);
} 