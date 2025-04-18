use bevy::{
    prelude::*,
    render::{
        render_graph::RenderGraph,
        renderer::RenderDevice,
        view::ViewTarget,
    },
};

use super::{
    node::PostProcessNode,
    pipeline::PostProcessPipeline,
    settings::{PostProcessSettings, PostProcessSettingsRaw},
};

/// The main plugin for post-processing effects.
/// This plugin sets up the render pipeline, node, and systems needed for post-processing.
pub struct PostProcessPlugin;

impl Plugin for PostProcessPlugin {
    fn build(&self, app: &mut App) {
        // Add the settings resource with default values
        app.insert_resource(PostProcessSettings::default());

        // Add our render systems
        app.add_systems(Startup, setup_post_process_pipeline)
            .add_systems(Update, update_post_process_settings);
    }
}

/// Sets up the post-processing pipeline and adds the render node to the graph
fn setup_post_process_pipeline(
    mut render_graph: ResMut<RenderGraph>,
    mut commands: Commands,
    device: Res<RenderDevice>,
) {
    // Create the pipeline
    let pipeline = PostProcessPipeline::from_world(&commands.world);
    commands.insert_resource(pipeline);

    // Create the node
    let view_target = ViewTarget::default();
    let bind_group = pipeline.create_bind_group(&device, &view_target);
    let node = PostProcessNode::new(view_target, bind_group);

    // Add node to render graph
    render_graph.add_node("post_process", node);
    render_graph.add_node_edge("post_process", bevy::render::main_graph::node::CAMERA_DRIVER);
}

/// Updates the post-processing settings buffer with the current settings
fn update_post_process_settings(
    settings: Res<PostProcessSettings>,
    pipeline: Res<PostProcessPipeline>,
    device: Res<RenderDevice>,
) {
    if settings.is_changed() {
        let raw_settings: PostProcessSettingsRaw = settings.as_ref().into();
        pipeline.update_settings(&device, &raw_settings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_setup() {
        let mut app = App::new();
        app.add_plugins((DefaultPlugins, PostProcessPlugin));

        // Verify resources were added
        assert!(app.world.contains_resource::<PostProcessSettings>());
        
        // Run the app for one frame to ensure systems execute
        app.update();
    }

    #[test]
    fn test_settings_update() {
        let mut app = App::new();
        app.add_plugins((DefaultPlugins, PostProcessPlugin));

        // Modify settings
        let mut settings = PostProcessSettings::default();
        settings.exposure = 1.5;
        settings.contrast = 1.2;
        app.world.insert_resource(settings);

        // Run update to trigger the system
        app.update();
    }
} 