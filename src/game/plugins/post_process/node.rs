use bevy::{
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
    },
};

use super::{PostProcessPipeline, PostProcessSettings};

/// Node that handles post-processing effects in the render graph.
/// This node applies effects like tone mapping, bloom, and color adjustments
/// to the final rendered image.
pub struct PostProcessNode {
    settings_buffer: bevy::render::renderer::BufferVec<PostProcessSettings>,
}

impl PostProcessNode {
    pub fn new(device: &RenderDevice) -> Self {
        Self {
            settings_buffer: BufferVec::new(
                device,
                "post_process_settings_buffer",
                bevy::render::renderer::BufferUsages::UNIFORM | bevy::render::renderer::BufferUsages::COPY_DST,
            ),
        }
    }
}

impl Node for PostProcessNode {
    fn update(&mut self, world: &mut World) {
        let settings = world.resource::<PostProcessSettings>();
        self.settings_buffer.clear();
        self.settings_buffer.push(settings.clone());
        self.settings_buffer.write_buffer(world.resource::<RenderDevice>());
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<PostProcessPipeline>();
        let view_target = graph.get_input_target()?;
        let post_process_target = ViewTarget::new(view_target.size);

        // Create bind group for the post-process pass
        let bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group",
            &pipeline.bind_group_layout,
            &[
                bevy::render::render_resource::BindGroupEntry {
                    binding: 0,
                    resource: view_target.main_texture().as_entire_binding(),
                },
                bevy::render::render_resource::BindGroupEntry {
                    binding: 1,
                    resource: bevy::render::render_resource::BindingResource::Buffer(
                        self.settings_buffer.buffer().unwrap().as_entire_buffer_binding(),
                    ),
                },
            ],
        );

        // Begin the post-process render pass
        let mut render_pass = render_context.begin_tracked_render_pass(
            bevy::render::render_resource::RenderPassDescriptor {
                label: Some("post_process_pass"),
                color_attachments: &[Some(
                    bevy::render::render_resource::RenderPassColorAttachment {
                        view: post_process_target.main_texture(),
                        resolve_target: None,
                        ops: bevy::render::render_resource::Operations {
                            load: bevy::render::render_resource::LoadOp::Clear(
                                bevy::render::render_resource::Color::BLACK,
                            ),
                            store: true,
                        },
                    },
                )],
                depth_stencil_attachment: None,
            },
        );

        // Set pipeline and bind group
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);

        // Draw fullscreen quad
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::renderer::RenderDevice;

    #[test]
    fn test_node_creation() {
        let mut world = World::new();
        let device = RenderDevice::wgpu_create_test_device();
        world.insert_resource(device);

        let node = PostProcessNode::new(world.resource::<RenderDevice>());
        assert!(node.settings_buffer.buffer().is_some());
    }
} 