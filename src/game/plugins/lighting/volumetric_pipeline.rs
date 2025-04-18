use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext, RenderGraph},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BufferBindingType, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, TextureFormat,
            TextureSampleType, TextureViewDimension, VertexState, Buffer, BufferUsages,
            BufferDescriptor, BufferInitDescriptor,
        },
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::{ViewUniforms, ViewUniformOffset},
        camera::CameraRenderGraph,
        Extract,
    },
};

use super::volumetric_texture::VolumetricTexture;

/// Resource that holds the render pipeline and bind group layout for volumetric rendering
/// This pipeline handles the rendering of volumetric effects like fog and clouds
#[derive(Resource)]
pub struct VolumetricPipeline {
    /// ID of the cached render pipeline
    pipeline_id: CachedRenderPipelineId,
    /// Layout for the bind group that provides shader resources
    bind_group_layout: BindGroupLayout,
}

/// Component that holds the bind group for volumetric rendering
/// Contains all the resources needed by the volumetric shader
#[derive(Component)]
pub struct VolumetricBindGroup {
    /// The actual bind group containing shader resources
    value: BindGroup,
}

/// Configuration settings for volumetric effects
/// These values control the appearance and behavior of volumetric lighting
/// 
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use sandk_offroad::plugins::lighting::VolumetricSettings;
/// 
/// fn setup_volumetrics(mut commands: Commands) {
///     // Create volumetric settings with default values
///     let settings = VolumetricSettings::default();
///     
///     // Or create custom settings
///     let custom_settings = VolumetricSettings::new(
///         0.5,  // density
///         0.8,  // scattering
///         0.2,  // absorption
///         100.0 // max_distance
///     );
///     
///     // Insert as a resource
///     commands.insert_resource(custom_settings);
/// }
/// ```
#[derive(Resource, ShaderType, Clone, Copy, Debug)]
pub struct VolumetricSettings {
    /// Overall density multiplier for the volumetric effect (0.0 - 1.0)
    pub density: f32,
    /// Amount of light scattering in the volume (0.0 - 1.0)
    pub scattering: f32,
    /// Light absorption factor in the volume (0.0 - 1.0)
    pub absorption: f32,
    /// Maximum distance for volumetric effects in world units
    pub max_distance: f32,
}

impl Default for VolumetricSettings {
    fn default() -> Self {
        Self {
            density: 0.1,     // Light fog by default
            scattering: 0.6,  // Medium scattering
            absorption: 0.1,   // Low absorption
            max_distance: 50.0, // 50 units max distance
        }
    }
}

impl VolumetricSettings {
    /// Create new volumetric settings with custom values
    /// 
    /// # Arguments
    /// * `density` - Overall density of the volumetric effect (0.0 - 1.0)
    /// * `scattering` - Amount of light scattering (0.0 - 1.0)
    /// * `absorption` - Light absorption factor (0.0 - 1.0)
    /// * `max_distance` - Maximum distance in world units
    pub fn new(density: f32, scattering: f32, absorption: f32, max_distance: f32) -> Self {
        Self {
            density: density.clamp(0.0, 1.0),
            scattering: scattering.clamp(0.0, 1.0),
            absorption: absorption.clamp(0.0, 1.0),
            max_distance: max_distance.max(0.0),
        }
    }

    /// Create settings for dense fog
    pub fn dense_fog() -> Self {
        Self::new(0.8, 0.3, 0.2, 30.0)
    }

    /// Create settings for light atmospheric haze
    pub fn light_haze() -> Self {
        Self::new(0.1, 0.7, 0.05, 100.0)
    }

    /// Create settings for volumetric clouds
    pub fn clouds() -> Self {
        Self::new(0.4, 0.9, 0.3, 200.0)
    }
}

/// Resource that holds the GPU buffer containing volumetric settings
#[derive(Resource)]
pub struct VolumetricSettingsBuffer {
    /// GPU buffer containing the settings data
    buffer: Buffer,
}

impl FromWorld for VolumetricPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // Create bind group layout with all necessary bindings for the volumetric shader:
        // - View uniforms for camera data
        // - Volume texture and sampler for the 3D volumetric data
        // - Scene texture and sampler for the main color buffer
        // - Depth texture for depth-aware fog
        // - Settings buffer for volumetric parameters
        let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("volumetric_bind_group_layout"),
            entries: &[
                // View uniform
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(ViewUniforms::min_size()),
                    },
                    count: None,
                },
                // Volume texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                // Volume sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Scene texture
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Scene sampler
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Depth texture
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Volumetric settings
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(VolumetricSettings::min_size()),
                    },
                    count: None,
                },
            ],
        });

        // Create the render pipeline that will render the volumetric effects
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/volumetric_render.wgsl");

        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("volumetric_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            vertex: VertexState {
                shader: shader.clone(),
                entry_point: "vertex".into(),
                buffers: vec![],  // No vertex buffers needed for full-screen quad
            },
            fragment: Some(FragmentState {
                shader,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,  // No depth testing needed for post-process
            multisample: MultisampleState::default(),
        });

        Self {
            pipeline_id,
            bind_group_layout,
        }
    }
}

impl FromWorld for VolumetricSettingsBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        
        // Create a uniform buffer to hold the volumetric settings
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("volumetric_settings_buffer"),
            size: std::mem::size_of::<VolumetricSettings>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { buffer }
    }
}

/// Node in the render graph that handles volumetric rendering
/// This node renders a full-screen quad with the volumetric shader
pub struct VolumetricNode {
    /// Query for getting view uniforms and bind groups
    query: QueryState<(
        &'static ViewUniformOffset,
        &'static VolumetricBindGroup,
    )>,
}

impl Node for VolumetricNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<VolumetricPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
            return Ok(());
        };

        let view_entity = graph.view_entity();
        let (view_uniform, bind_group) = self
            .query
            .get_manual(world, view_entity)
            .expect("View entity should have required components");

        // Create a render pass that renders the volumetric effects
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("volumetric_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: graph.view().target,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        });

        // Draw a full-screen triangle with the volumetric shader
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group.value, &[view_uniform.offset]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

/// System that prepares resources for volumetric rendering each frame
/// Creates bind groups and updates settings buffer
pub fn prepare_volumetrics(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<VolumetricPipeline>,
    view_uniforms: Res<ViewUniforms>,
    volumetric_texture: Res<VolumetricTexture>,
    settings_buffer: Res<VolumetricSettingsBuffer>,
    images: Res<RenderAssets<Image>>,
    mut volumetric_settings: ResMut<VolumetricSettings>,
    views: Query<(Entity, &ViewUniformOffset, &ExtractedView)>,
) {
    // Create a linear sampler for smooth texture sampling
    let sampler = render_device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..default()
    });

    // Update the settings buffer with current values
    render_device.queue().write_buffer(
        &settings_buffer.buffer,
        0,
        bytemuck::cast_slice(&[*volumetric_settings.as_ref()]),
    );

    // Create bind groups for each view
    for (entity, view_uniform, view) in views.iter() {
        let Some(volume_texture_view) = images.get(&volumetric_texture.texture) else {
            continue;
        };

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("volumetric_bind_group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                // View uniform
                BindGroupEntry {
                    binding: 0,
                    resource: view_uniforms.uniforms.binding().unwrap(),
                },
                // Volume texture
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&volume_texture_view.texture_view),
                },
                // Volume sampler
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
                // Scene texture
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(view.main_texture()),
                },
                // Scene sampler
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(&sampler),
                },
                // Depth texture
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(view.depth_texture()),
                },
                // Volumetric settings
                BindGroupEntry {
                    binding: 6,
                    resource: settings_buffer.buffer.as_entire_binding(),
                },
            ],
        });

        commands.entity(entity).insert(VolumetricBindGroup {
            value: bind_group,
        });
    }
}

/// Plugin that sets up the volumetric rendering system
pub struct VolumetricRenderPlugin;

impl Plugin for VolumetricRenderPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources and add systems
        app.init_resource::<VolumetricSettings>()
            .add_plugins(ExtractResourcePlugin::<VolumetricSettings>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<VolumetricPipeline>()
            .init_resource::<VolumetricSettingsBuffer>()
            .add_systems(Render, prepare_volumetrics.in_set(RenderSet::PrepareResources));

        // Add the volumetric node to the render graph after the main pass
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        
        render_graph.add_node("volumetric", VolumetricNode::default());
        
        render_graph.add_node_edge(
            "main_pass",
            "volumetric",
        );
    }
}

/// Example setup for the volumetric lighting system:
/// ```rust
/// use bevy::prelude::*;
/// use sandk_offroad::plugins::lighting::{VolumetricLightingPlugin, VolumetricSettings};
/// 
/// fn main() {
///     App::new()
///         // Add the volumetric lighting plugin
///         .add_plugins(VolumetricLightingPlugin)
///         // Configure initial settings
///         .insert_resource(VolumetricSettings::new(
///             0.5,   // density
///             0.8,   // scattering
///             0.2,   // absorption
///             100.0  // max_distance
///         ))
///         .add_systems(Startup, setup)
///         .run();
/// }
/// 
/// fn setup(mut commands: Commands) {
///     // Setup camera with volumetric effects
///     commands.spawn((
///         Camera3dBundle::default(),
///         // The plugin will automatically handle volumetric setup
///     ));
///     
///     // Create different atmospheric conditions
///     let fog_settings = VolumetricSettings::dense_fog();
///     let haze_settings = VolumetricSettings::light_haze();
///     let cloud_settings = VolumetricSettings::clouds();
///     
///     // System to update settings based on weather
///     fn update_weather(
///         time: Res<Time>,
///         mut settings: ResMut<VolumetricSettings>
///     ) {
///         // Smoothly transition between different atmospheric conditions
///         let t = (time.elapsed_seconds() * 0.1).sin() * 0.5 + 0.5;
///         *settings = VolumetricSettings::new(
///             0.2 + t * 0.3,  // varying density
///             0.6,            // constant scattering
///             0.1 + t * 0.2,  // varying absorption
///             100.0,          // constant max distance
///         );
///     }
/// }
/// ```
/// 
/// The volumetric lighting system provides realistic atmospheric effects by:
/// 1. Rendering a full-screen pass that ray marches through a 3D volume texture
/// 2. Applying physically-based light scattering and absorption
/// 3. Supporting dynamic updates to the volume texture for animated effects
/// 4. Providing easy-to-use presets for common atmospheric conditions
/// 
/// The system automatically integrates with Bevy's rendering pipeline and handles:
/// - Resource creation and cleanup
/// - Shader compilation and pipeline setup
/// - Per-frame updates and synchronization
/// - Camera integration and depth buffer handling 