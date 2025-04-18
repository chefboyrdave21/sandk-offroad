use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::ViewUniform,
        Extract, Render, RenderApp, RenderSet,
    },
};

/// Configuration for volumetric lighting effects
#[derive(Resource)]
pub struct VolumetricConfig {
    /// Global density of the participating media (fog, dust, etc.)
    pub density: f32,
    /// Scattering coefficient for the media
    pub scattering: f32,
    /// Absorption coefficient for the media
    pub absorption: f32,
    /// Number of steps for ray marching
    pub sample_count: u32,
    /// Maximum distance for volumetric effects
    pub max_distance: f32,
}

impl Default for VolumetricConfig {
    fn default() -> Self {
        Self {
            density: 0.02,
            scattering: 0.8,
            absorption: 0.2,
            sample_count: 64,
            max_distance: 100.0,
        }
    }
}

/// Component for entities that should cast volumetric shadows
#[derive(Component)]
pub struct VolumetricShadowCaster;

/// Resource holding volumetric lighting data for the GPU
#[derive(Resource)]
struct VolumetricResources {
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    pipeline: ComputePipeline,
    volume_texture: Texture,
    volume_texture_view: TextureView,
}

pub struct VolumetricPlugin;

impl Plugin for VolumetricPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VolumetricConfig>();

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<VolumetricResources>()
            .add_systems(Render, prepare_volumetrics.in_set(RenderSet::Prepare));
    }
}

impl FromWorld for VolumetricResources {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // Create bind group layout
        let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("volumetric_bind_group_layout"),
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
                // Volume texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D3,
                    },
                    count: None,
                },
                // Light data
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create compute pipeline
        let shader = render_device.create_shader_module(ShaderModuleDescriptor {
            label: Some("volumetric_shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/volumetric.wgsl").into()),
        });

        let pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("volumetric_pipeline"),
            layout: Some(&render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("volumetric_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            })),
            module: &shader,
            entry_point: "main",
        });

        // Create 3D volume texture
        let volume_texture = render_device.create_texture(&TextureDescriptor {
            label: Some("volumetric_texture"),
            size: Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 64,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let volume_texture_view = volume_texture.create_view(&TextureViewDescriptor::default());

        Self {
            bind_group_layout,
            bind_group: None,
            pipeline,
            volume_texture,
            volume_texture_view,
        }
    }
}

fn prepare_volumetrics(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut volumetric_resources: ResMut<VolumetricResources>,
    view_uniforms: Res<ViewUniforms>,
    config: Res<VolumetricConfig>,
    lights: Query<(&GlobalTransform, &PointLight)>,
) {
    // Skip if no lights or view uniforms
    if lights.is_empty() || view_uniforms.uniforms.is_empty() {
        return;
    }

    // Create light data buffer
    let light_data: Vec<_> = lights
        .iter()
        .map(|(transform, light)| {
            let position = transform.translation();
            [
                position.x,
                position.y,
                position.z,
                1.0,
                light.color.r(),
                light.color.g(),
                light.color.b(),
                light.intensity,
            ]
        })
        .collect();

    let light_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("volumetric_light_buffer"),
        contents: bytemuck::cast_slice(&light_data),
        usage: BufferUsages::STORAGE,
    });

    // Create bind group
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("volumetric_bind_group"),
        layout: &volumetric_resources.bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: view_uniforms.uniforms.binding().unwrap(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&volumetric_resources.volume_texture_view),
            },
            BindGroupEntry {
                binding: 2,
                resource: light_buffer.as_entire_binding(),
            },
        ],
    });

    volumetric_resources.bind_group = Some(bind_group);

    // Dispatch compute shader
    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    {
        let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor::default());
        compute_pass.set_pipeline(&volumetric_resources.pipeline);
        compute_pass.set_bind_group(0, volumetric_resources.bind_group.as_ref().unwrap(), &[]);
        compute_pass.dispatch_workgroups(8, 8, 4); // 128/16 = 8 workgroups in each dimension
    }
    render_queue.submit(vec![command_encoder.finish()]);
} 