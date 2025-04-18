struct ViewUniform {
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    position: vec3<f32>,
    viewport: vec2<f32>,
}

struct Light {
    position: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
    intensity: f32,
}

struct TileData {
    min_depth: f32,
    max_depth: f32,
    light_count: u32,
    light_indices: array<u32, 256>,
}

@group(0) @binding(0)
var<uniform> view: ViewUniform;

@group(0) @binding(1)
var<storage> lights: array<Light>;

@group(0) @binding(2)
var<storage, read_write> light_indices: array<u32>;

@group(0) @binding(3)
var<storage, read_write> tile_data: array<TileData>;

const TILE_SIZE: u32 = 16u;
const MAX_LIGHTS_PER_TILE: u32 = 256u;

// Helper function to transform screen space coordinates to view space
fn screen_to_view(screen_pos: vec2<f32>, depth: f32) -> vec3<f32> {
    let clip_space = vec4<f32>(
        screen_pos.x * 2.0 - 1.0,
        1.0 - screen_pos.y * 2.0,
        depth,
        1.0
    );
    let view_pos = view.inverse_view_proj * clip_space;
    return view_pos.xyz / view_pos.w;
}

// Helper function to calculate frustum planes from corners
fn calculate_frustum_planes(corners: array<vec3<f32>, 4>) -> array<vec4<f32>, 6> {
    var planes: array<vec4<f32>, 6>;
    
    // Calculate normals for the four side planes
    let v0 = normalize(corners[1] - corners[0]);
    let v1 = normalize(corners[2] - corners[0]);
    let v2 = normalize(corners[3] - corners[1]);
    let v3 = normalize(corners[3] - corners[2]);
    
    // Left plane
    let left_normal = normalize(cross(v1, corners[0]));
    planes[0] = vec4<f32>(left_normal, -dot(left_normal, corners[0]));
    
    // Right plane
    let right_normal = normalize(cross(corners[1], v2));
    planes[1] = vec4<f32>(right_normal, -dot(right_normal, corners[1]));
    
    // Top plane
    let top_normal = normalize(cross(v0, corners[0]));
    planes[2] = vec4<f32>(top_normal, -dot(top_normal, corners[0]));
    
    // Bottom plane
    let bottom_normal = normalize(cross(corners[2], v3));
    planes[3] = vec4<f32>(bottom_normal, -dot(bottom_normal, corners[2]));
    
    // Near and far planes (simplified for performance)
    planes[4] = vec4<f32>(0.0, 0.0, 1.0, 0.0);  // Near plane at z=0
    planes[5] = vec4<f32>(0.0, 0.0, -1.0, -1.0); // Far plane at z=1
    
    return planes;
}

// Helper function to test if a sphere intersects with the frustum
fn sphere_intersects_frustum(center: vec3<f32>, radius: f32, planes: array<vec4<f32>, 6>) -> bool {
    for (var i = 0u; i < 6u; i = i + 1u) {
        let plane = planes[i];
        let distance = dot(vec4<f32>(center, 1.0), plane);
        if (distance < -radius) {
            return false;
        }
    }
    return true;
}

@compute @workgroup_size(16, 16, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    // Calculate tile index and get tile data
    let tile_index = workgroup_id.y * (view.viewport.x + TILE_SIZE - 1u) / TILE_SIZE + workgroup_id.x;
    var tile = &tile_data[tile_index];
    
    // Initialize tile data
    (*tile).min_depth = 1.0;
    (*tile).max_depth = 0.0;
    (*tile).light_count = 0u;
    
    // Calculate tile corners in screen space
    let tile_x = f32(workgroup_id.x * TILE_SIZE) / view.viewport.x;
    let tile_y = f32(workgroup_id.y * TILE_SIZE) / view.viewport.y;
    let tile_size_x = f32(TILE_SIZE) / view.viewport.x;
    let tile_size_y = f32(TILE_SIZE) / view.viewport.y;
    
    // Calculate tile frustum corners in view space
    let corners = array<vec3<f32>, 4>(
        screen_to_view(vec2<f32>(tile_x, tile_y), 1.0),
        screen_to_view(vec2<f32>(tile_x + tile_size_x, tile_y), 1.0),
        screen_to_view(vec2<f32>(tile_x, tile_y + tile_size_y), 1.0),
        screen_to_view(vec2<f32>(tile_x + tile_size_x, tile_y + tile_size_y), 1.0)
    );
    
    // Calculate frustum planes
    let frustum_planes = calculate_frustum_planes(corners);
    
    // Test each light against the tile frustum
    for (var i = 0u; i < arrayLength(&lights); i = i + 1u) {
        let light = lights[i];
        let light_pos = (view.view * vec4<f32>(light.position, 1.0)).xyz;
        
        // Test if light intersects with tile frustum
        if (sphere_intersects_frustum(light_pos, light.radius, frustum_planes)) {
            if ((*tile).light_count < MAX_LIGHTS_PER_TILE) {
                let index = atomicAdd(&(*tile).light_count, 1u);
                (*tile).light_indices[index] = i;
            }
        }
    }
}