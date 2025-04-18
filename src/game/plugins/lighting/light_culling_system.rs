use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::RenderDevice,
        view::ViewUniform,
    },
};

pub struct LightCullingPlugin;

impl Plugin for LightCullingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightCullingPipeline>()
           .add_systems(Update, prepare_light_culling);
    }
}

#[derive(Resource)]
pub struct LightCullingPipeline {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for LightCullingPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        
        let shader = render_device.create_shader_module(include_wgsl!("shaders/light_culling.wgsl"));
        let bind_group_layout = create_bind_group_layout(render_device);
        let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("light_culling_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("light_culling_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });
        
        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

fn create_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("light_culling_bind_group_layout"),
        entries: &[
            // View uniform
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
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
    })
}

#[derive(Component)]
pub struct LightCullingView {
    bind_group: BindGroup,
    light_buffer: Buffer,
    light_indices_buffer: Buffer,
    tile_data_buffer: Buffer,
}

fn prepare_light_culling(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<LightCullingPipeline>,
    view_uniforms: Res<ViewUniforms>,
    lights: Query<&PointLight>,
    views: Query<(Entity, &Camera)>,
) {
    for (entity, camera) in views.iter() {
        if !camera.is_active {
            continue;
        }
        
        // Create light data
        let light_data: Vec<Light> = lights.iter()
            .map(|light| Light {
                position: light.position.extend(1.0),
                color: light.color.extend(light.intensity),
                range: light.range,
                padding: Vec3::ZERO,
            })
            .collect();
        
        // Create buffers
        let light_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("light_buffer"),
            contents: bytemuck::cast_slice(&light_data),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });
        
        let light_indices_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("light_indices_buffer"),
            size: std::mem::size_of::<LightIndex>() as u64 * 1024,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let tile_data_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("tile_data_buffer"),
            size: std::mem::size_of::<TileData>() as u64 * 64,
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
                    resource: view_uniforms.uniforms.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: light_indices_buffer.as_entire_binding(),
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
            light_indices_buffer,
            tile_data_buffer,
        });
    }
} 