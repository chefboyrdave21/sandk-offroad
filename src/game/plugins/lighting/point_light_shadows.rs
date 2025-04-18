use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::camera::CameraProjection;
use bevy::render::view::RenderLayers;

/// Size of the shadow cube map faces
pub const POINT_SHADOW_MAP_SIZE: u32 = 1024;

/// Component for point light shadow mapping
#[derive(Component)]
pub struct PointLightShadowMap {
    pub texture: Handle<Image>,
    pub view_matrices: [Mat4; 6],
    pub proj_matrix: Mat4,
}

impl Default for PointLightShadowMap {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            view_matrices: [Mat4::IDENTITY; 6],
            proj_matrix: Mat4::perspective_infinite_rh(
                90.0_f32.to_radians(),
                1.0,
                0.1,
            ),
        }
    }
}

/// System to create and update point light shadow maps
pub fn update_point_light_shadows(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    point_lights: Query<(Entity, &GlobalTransform, &PointLight), Without<PointLightShadowMap>>,
    mut shadow_maps: Query<(&mut PointLightShadowMap, &GlobalTransform)>,
) {
    // Create shadow maps for new point lights
    for (entity, transform, light) in point_lights.iter() {
        if light.shadows_enabled {
            let texture = create_shadow_cube_map(&mut images);
            let shadow_map = PointLightShadowMap {
                texture,
                ..default()
            };
            commands.entity(entity).insert(shadow_map);
        }
    }

    // Update existing shadow maps
    for (mut shadow_map, transform) in shadow_maps.iter_mut() {
        update_shadow_matrices(&mut shadow_map, transform);
    }
}

/// Create a cube map texture for point light shadows
fn create_shadow_cube_map(images: &mut Assets<Image>) -> Handle<Image> {
    let size = POINT_SHADOW_MAP_SIZE;
    let mut image = Image::new_fill(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6, // Cube map faces
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Depth32Float,
    );
    image.texture_descriptor.usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
    image.texture_descriptor.sample_count = 1;
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    images.add(image)
}

/// Update view matrices for each cube map face
fn update_shadow_matrices(shadow_map: &mut PointLightShadowMap, transform: &GlobalTransform) {
    let pos = transform.translation();
    
    // Cube map face directions and up vectors
    let faces = [
        (Vec3::X, Vec3::Y),   // Right face  (+X)
        (-Vec3::X, Vec3::Y),  // Left face   (-X)
        (Vec3::Y, -Vec3::Z),  // Top face    (+Y)
        (-Vec3::Y, Vec3::Z),  // Bottom face (-Y)
        (Vec3::Z, Vec3::Y),   // Front face  (+Z)
        (-Vec3::Z, Vec3::Y),  // Back face   (-Z)
    ];

    for (i, (dir, up)) in faces.iter().enumerate() {
        shadow_map.view_matrices[i] = Mat4::look_at_rh(
            pos,
            pos + *dir,
            *up,
        );
    }
}

/// System to render point light shadows
pub fn render_point_light_shadows(
    mut commands: Commands,
    mut point_lights: Query<(Entity, &GlobalTransform, &PointLight, &PointLightShadowMap)>,
    mut pipelines: ResMut<PipelineCache>,
    mut pipeline: Local<Option<CachedRenderPipelineId>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    // Create pipeline if it doesn't exist
    if pipeline.is_none() {
        let shader = render_device.create_shader_module(ShaderModuleDescriptor {
            label: Some("point_light_shadow_shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/point_light_shadow.wgsl").into()),
        });

        let pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("point_light_shadow_pipeline".into()),
            layout: vec![],  // Will be filled with bind group layouts
            vertex: VertexState {
                shader: shader.clone(),
                entry_point: "vertex".into(),
                buffers: vec![],  // Will be filled with vertex buffer layouts
            },
            fragment: Some(FragmentState {
                shader,
                entry_point: "fragment".into(),
                targets: vec![],  // No color targets needed for shadow mapping
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: DepthBiasState {
                        constant: 2,  // Reduce shadow acne
                        slope_scale: 2.0,
                        clamp: 0.0,
                    },
                }),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState::default(),
        };

        *pipeline = Some(pipelines.queue_render_pipeline(pipeline_descriptor));
    }

    // Render shadows for each point light
    for (entity, transform, light, shadow_map) in point_lights.iter_mut() {
        for face in 0..6 {
            let view = ViewUniform {
                view_proj: shadow_map.view_matrices[face] * shadow_map.proj_matrix,
                view: shadow_map.view_matrices[face],
                inverse_view: shadow_map.view_matrices[face].inverse(),
                projection: shadow_map.proj_matrix,
                world_position: transform.translation().extend(1.0),
                near: 0.1,
                far: light.range,
            };

            // Create view bind group
            let view_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("point_light_shadow_view_buffer"),
                contents: bytemuck::cast_slice(&[view]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let view_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("point_light_shadow_view_bind_group"),
                layout: &shadow_map.bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_buffer.as_entire_binding(),
                }],
            });

            // Set up render pass
            commands.spawn(RenderPhase::<Shadow>::new()).with_children(|parent| {
                parent.spawn(ShadowPass {
                    target: shadow_map.texture.clone(),
                    layer: face,
                    pipeline: pipeline.unwrap(),
                    bind_group: view_bind_group,
                });
            });
        }
    }
}

// Shadow pass component
#[derive(Component)]
struct ShadowPass {
    target: Handle<Image>,
    layer: u32,
    pipeline: CachedRenderPipelineId,
    bind_group: BindGroup,
}

// View uniform for shadow pass
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct ViewUniform {
    view_proj: Mat4,
    view: Mat4,
    inverse_view: Mat4,
    projection: Mat4,
    world_position: Vec4,
    near: f32,
    far: f32,
} 