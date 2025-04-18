use bevy::{
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::RenderContext,
        view::ViewTarget,
    },
};

use super::light_culling_pipeline::{LightCullingPipeline, LightCullingView};

pub struct LightCullingNode;

impl Node for LightCullingNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<LightCullingPipeline>();
        let view_entity = graph.view_entity();

        // Get the light culling view component
        if let Some(light_culling_view) = world.get::<LightCullingView>(view_entity) {
            let viewport = world.get::<ViewTarget>(view_entity)
                .unwrap()
                .main_texture_view
                .texture()
                .size();

            let tile_count_x = (viewport.width + 15) / 16;
            let tile_count_y = (viewport.height + 15) / 16;

            // Begin compute pass
            let mut compute_pass = render_context
                .command_encoder()
                .begin_compute_pass(&Default::default());

            // Set pipeline and bind group
            compute_pass.set_pipeline(&pipeline.compute_pipeline);
            compute_pass.set_bind_group(0, &light_culling_view.bind_group, &[]);

            // Dispatch compute shader
            compute_pass.dispatch_workgroups(tile_count_x, tile_count_y, 1);
        }

        Ok(())
    }
}

pub mod node {
    use bevy::render::render_graph::NodeLabel;

    #[derive(Debug, Hash, PartialEq, Eq, Clone)]
    pub enum LightCulling {
        LightCulling,
    }

    impl NodeLabel for LightCulling {
        fn debug_name(&self) -> std::borrow::Cow<'static, str> {
            "LightCulling".into()
        }
    }
} 