use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindGroup, BindGroupLayout, ComputePipeline,
            ShaderType, StorageBuffer,
        },
        renderer::RenderDevice,
    },
};

#[test]
fn test_compute_pipeline_creation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let render_device = app.world.resource::<RenderDevice>();
    
    let shader = render_device.create_shader_module(include_wgsl!("../shaders/light_culling.wgsl"));
    let pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("light_culling_pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });
    
    assert!(pipeline.is_some());
}

#[test]
fn test_bind_group_layout() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let render_device = app.world.resource::<RenderDevice>();
    
    let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
    });
    
    assert!(bind_group_layout.is_some());
}

#[test]
fn test_storage_buffer_creation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let render_device = app.world.resource::<RenderDevice>();
    
    // Create light buffer
    let light_buffer = StorageBuffer::new(
        &render_device,
        &[Light {
            position: Vec4::new(0.0, 5.0, 0.0, 1.0),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            range: 10.0,
            padding: Vec3::ZERO,
        }],
        Some("light_buffer"),
    );
    
    // Create light indices buffer
    let light_indices_buffer = StorageBuffer::new(
        &render_device,
        &vec![LightIndex { index: 0 }; 1024],
        Some("light_indices_buffer"),
    );
    
    // Create tile data buffer
    let tile_data_buffer = StorageBuffer::new(
        &render_device,
        &vec![TileData {
            min_depth: 0.0,
            max_depth: 1.0,
            light_count: 0,
            padding: 0.0,
        }; 64],
        Some("tile_data_buffer"),
    );
    
    assert!(light_buffer.buffer().is_some());
    assert!(light_indices_buffer.buffer().is_some());
    assert!(tile_data_buffer.buffer().is_some());
}

#[test]
fn test_bind_group_creation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let render_device = app.world.resource::<RenderDevice>();
    
    // Create buffers (reusing code from previous test)
    let view_uniform = ViewUniform::new();
    let view_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("view_uniform_buffer"),
        contents: bytemuck::cast_slice(&[view_uniform]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });
    
    let light_buffer = StorageBuffer::new(
        &render_device,
        &[Light::default()],
        Some("light_buffer"),
    );
    
    let light_indices_buffer = StorageBuffer::new(
        &render_device,
        &vec![LightIndex { index: 0 }; 1024],
        Some("light_indices_buffer"),
    );
    
    let tile_data_buffer = StorageBuffer::new(
        &render_device,
        &vec![TileData::default(); 64],
        Some("tile_data_buffer"),
    );
    
    // Create bind group
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("light_culling_bind_group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: view_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: light_buffer.buffer().unwrap().as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: light_indices_buffer.buffer().unwrap().as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: tile_data_buffer.buffer().unwrap().as_entire_binding(),
            },
        ],
    });
    
    assert!(bind_group.is_some());
} 