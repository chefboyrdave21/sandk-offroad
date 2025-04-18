use bevy::prelude::*;
use bevy::render::render_resource::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferBindingType, ShaderStages,
    StorageTextureAccess, TextureFormat, TextureViewDimension,
};
use bevy::render::renderer::RenderDevice;

/// Size of the tile in pixels for light culling
const TILE_SIZE: u32 = 16;

/// Maximum number of lights per tile
const MAX_LIGHTS_PER_TILE: u32 = 64;

#[derive(Resource)]
pub struct LightCullingResources {
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

#[derive(Component)]
pub struct LightCullingConfig {
    pub enabled: bool,
    pub debug_view: bool,
}

impl Default for LightCullingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debug_view: false,
        }
    }
}

pub fn setup_light_culling(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
) {
    // Create bind group layout for light culling compute shader
    let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("light_culling_bind_group_layout"),
        entries: &[
            // Light data buffer
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: bevy::render::render_resource::BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Depth texture
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: bevy::render::render_resource::BindingType::StorageTexture {
                    access: StorageTextureAccess::ReadOnly,
                    format: TextureFormat::Depth32Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            // Light grid
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: bevy::render::render_resource::BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Light indices
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: bevy::render::render_resource::BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    // Create empty bind group (will be updated each frame)
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("light_culling_bind_group"),
        layout: &bind_group_layout,
        entries: &[],
    });

    commands.insert_resource(LightCullingResources {
        bind_group,
        bind_group_layout,
    });
}

pub fn update_light_culling(
    mut commands: Commands,
    config: Res<LightCullingConfig>,
    lights: Query<(Entity, &GlobalTransform, &PointLight)>,
    // Add other resources needed for culling
) {
    if !config.enabled {
        return;
    }

    // Calculate view frustum planes
    // Perform light culling using compute shader
    // Update light grid and indices
}

#[derive(Component)]
pub struct LightCullingDebugView;

pub fn debug_light_culling(
    mut commands: Commands,
    config: Res<LightCullingConfig>,
    query: Query<Entity, With<LightCullingDebugView>>,
) {
    if config.debug_view {
        if query.is_empty() {
            // Create debug visualization
            commands.spawn((
                LightCullingDebugView,
                // Add visualization components
            ));
        }
    } else {
        // Remove debug visualization
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Compute shader for light culling
pub const LIGHT_CULLING_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> lights: array<Light>;
@group(0) @binding(1) var depth_texture: texture_storage_2d<r32float, read>;
@group(0) @binding(2) var<storage, read_write> light_grid: array<u32>;
@group(0) @binding(3) var<storage, read_write> light_indices: array<u32>;

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
    range: f32,
}

@compute @workgroup_size(TILE_SIZE, TILE_SIZE, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tile_index = global_id.y * TILE_SIZE + global_id.x;
    var light_count = 0u;
    
    // Calculate tile frustum planes
    let tile_min = vec2<f32>(f32(global_id.x * TILE_SIZE), f32(global_id.y * TILE_SIZE));
    let tile_max = tile_min + vec2<f32>(f32(TILE_SIZE));
    
    // Test each light against tile frustum
    for (var i = 0u; i < arrayLength(&lights); i = i + 1u) {
        let light = lights[i];
        if (light_intersects_tile(light, tile_min, tile_max)) {
            if (light_count < MAX_LIGHTS_PER_TILE) {
                light_indices[tile_index * MAX_LIGHTS_PER_TILE + light_count] = i;
                light_count = light_count + 1u;
            }
        }
    }
    
    light_grid[tile_index] = light_count;
}

fn light_intersects_tile(light: Light, tile_min: vec2<f32>, tile_max: vec2<f32>) -> bool {
    let light_pos = light.position.xyz;
    let range = light.range;
    
    // Simple 2D AABB test for now
    let circle_center = light_pos.xy;
    let box_center = (tile_min + tile_max) * 0.5;
    let box_half_size = (tile_max - tile_min) * 0.5;
    
    let diff = abs(circle_center - box_center);
    let closest = min(diff, box_half_size);
    let distance = length(diff - closest);
    
    return distance <= range;
}
"#; 