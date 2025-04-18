use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::RenderDevice,
        view::ViewTarget,
    },
};

use super::{
    pipeline::PostProcessPipeline,
    settings::PostProcessSettings,
};

/// Holds the bind group and buffers needed for post-processing
#[derive(Resource)]
pub struct PostProcessBindGroup {
    bind_group: BindGroup,
    settings_buffer: Buffer,
}

impl PostProcessBindGroup {
    /// Creates a new bind group for post-processing
    pub fn new(
        render_device: &RenderDevice,
        pipeline: &PostProcessPipeline,
        view_target: &ViewTarget,
        settings: &PostProcessSettings,
    ) -> Self {
        // Create settings buffer
        let settings_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("post_process_settings_buffer"),
            contents: bytemuck::cast_slice(&[*settings]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("post_process_bind_group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(view_target.main_texture()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&view_target.sampler()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: settings_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            bind_group,
            settings_buffer,
        }
    }

    /// Updates the settings buffer with new settings
    pub fn update_settings(&self, render_device: &RenderDevice, settings: &PostProcessSettings) {
        render_device.queue().write_buffer(
            &self.settings_buffer,
            0,
            bytemuck::cast_slice(&[*settings]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_group_creation() {
        let mut app = App::new();
        
        // Setup render device and pipeline
        app.init_resource::<RenderDevice>();
        app.init_resource::<PostProcessPipeline>();
        
        let render_device = app.world.resource::<RenderDevice>();
        let pipeline = app.world.resource::<PostProcessPipeline>();
        let settings = PostProcessSettings::default();
        
        // Create view target (mock for testing)
        let view_target = ViewTarget::default();
        
        // Create bind group
        let bind_group = PostProcessBindGroup::new(
            render_device,
            pipeline,
            &view_target,
            &settings,
        );
        
        // Verify bind group and buffer are created
        assert!(bind_group.bind_group.as_ref() != std::ptr::null());
        assert!(bind_group.settings_buffer.as_ref() != std::ptr::null());
    }

    #[test]
    fn test_settings_update() {
        let mut app = App::new();
        app.init_resource::<RenderDevice>();
        app.init_resource::<PostProcessPipeline>();
        
        let render_device = app.world.resource::<RenderDevice>();
        let pipeline = app.world.resource::<PostProcessPipeline>();
        let mut settings = PostProcessSettings::default();
        
        let view_target = ViewTarget::default();
        let bind_group = PostProcessBindGroup::new(
            render_device,
            pipeline,
            &view_target,
            &settings,
        );
        
        // Update settings
        settings.exposure = 2.0;
        settings.contrast = 1.5;
        bind_group.update_settings(render_device, &settings);
        
        // Note: Can't verify buffer contents in test, but we can verify the call succeeds
    }
} 