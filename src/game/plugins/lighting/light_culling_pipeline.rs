use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::{ViewUniform, ViewUniforms},
        Extract, Render, RenderApp, RenderSet,
    },
};

const WORKGROUP_SIZE: u32 = 16;

#[derive(Resource)]
pub struct LightCullingPipeline {
    compute_pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for LightCullingPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // Create bind group layout
        let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("light_culling_bind_group_layout"),
            entries: &[
                // View uniform
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(ViewUniform::min_size()),
                    },
                    count: None,
                },
                // Lights storage buffer
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Light indices storage buffer
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Tile data storage buffer
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("light_culling_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Load and create shader module
        let shader = std::fs::read_to_string(
            "src/game/plugins/lighting/shaders/light_culling.wgsl",
        ).expect("Failed to load light culling shader");

        let shader_module = render_device.create_shader_module(ShaderModuleDescriptor {
            label: Some("light_culling_shader"),
            source: ShaderSource::Wgsl(shader.into()),
        });

        // Create compute pipeline
        let compute_pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("light_culling_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main",
        });

        Self {
            compute_pipeline,
            bind_group_layout,
        }
    }
}

#[derive(Component)]
pub struct LightCullingView {
    bind_group: BindGroup,
    light_buffer: Buffer,
    light_index_buffer: Buffer,
    tile_data_buffer: Buffer,
}

pub fn prepare_light_culling(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline: Res<LightCullingPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<(Entity, &ExtractedView)>,
    lights: Query<(&PointLight, &GlobalTransform)>,
) {
    let view_binding = match view_uniforms.uniforms.binding() {
        Some(binding) => binding,
        None => return,
    };

    // Create light data
    let mut light_data = Vec::new();
    for (light, transform) in lights.iter() {
        light_data.push(LightData {
            position: transform.translation().extend(1.0),
            color: light.color.as_linear_rgba_f32().into(),
            range: light.range,
            padding: [0.0; 3],
        });
    }

    for (entity, view) in views.iter() {
        let viewport = view.viewport.as_vec2();
        let tile_count_x = (viewport.x as u32 + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
        let tile_count_y = (viewport.y as u32 + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
        let tile_count = (tile_count_x * tile_count_y) as usize;

        // Create buffers
        let light_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("light_buffer"),
            contents: bytemuck::cast_slice(&light_data),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let light_index_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("light_index_buffer"),
            size: (tile_count * 256 * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let tile_data_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("tile_data_buffer"),
            size: (tile_count * std::mem::size_of::<TileData>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("light_culling_bind_group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: light_index_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: tile_data_buffer.as_entire_binding(),
                },
            ],
        });

        commands.entity(entity).insert(LightCullingView {
            bind_group,
            light_buffer,
            light_index_buffer,
            tile_data_buffer,
        });
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct LightData {
    position: Vec4,
    color: Vec4,
    range: f32,
    padding: [f32; 3],
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct TileData {
    min_depth: f32,
    max_depth: f32,
    light_count: u32,
    padding: f32,
}

pub struct LightCullingPlugin;

impl Plugin for LightCullingPlugin {
    fn build(&self, app: &mut App) {
        // Register the plugin with the render app
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<LightCullingPipeline>()
                .add_systems(Render, prepare_light_culling.in_set(RenderSet::Prepare));
        }
    }
} 