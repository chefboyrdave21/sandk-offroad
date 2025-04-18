#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use std::time::Duration;

    fn setup_test_assets() -> (App, AssetServer, GameAssets) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default());
        
        let asset_server = app.world.resource::<AssetServer>().clone();
        let game_assets = GameAssets::default();
        
        // Create test asset directories
        std::fs::create_dir_all("assets/test/vehicles/models").unwrap();
        std::fs::create_dir_all("assets/test/vehicles/textures").unwrap();
        std::fs::create_dir_all("assets/test/audio/engine").unwrap();
        std::fs::create_dir_all("assets/test/audio/environment").unwrap();
        std::fs::create_dir_all("assets/test/ui/textures").unwrap();
        std::fs::create_dir_all("assets/test/ui/fonts").unwrap();
        std::fs::create_dir_all("assets/test/terrain/textures").unwrap();
        std::fs::create_dir_all("assets/test/terrain/heightmaps").unwrap();
        std::fs::create_dir_all("assets/test/effects/particles").unwrap();
        std::fs::create_dir_all("assets/test/effects/weather").unwrap();
        std::fs::create_dir_all("assets/test/shaders/custom").unwrap();
        std::fs::create_dir_all("assets/test/shaders/materials").unwrap();

        (app, asset_server, game_assets)
    }

    #[test]
    fn test_asset_loading() {
        let (mut app, asset_server, mut game_assets) = setup_test_assets();
        
        // Create test assets
        std::fs::write("assets/test/ui/textures/loading.png", &[0; 100]).unwrap();
        std::fs::write("assets/test/ui/fonts/main.ttf", &[0; 100]).unwrap();
        std::fs::write("assets/test/vehicles/models/truck.glb", &[0; 100]).unwrap();
        std::fs::write("assets/test/audio/engine/idle.ogg", &[0; 100]).unwrap();
        
        game_assets.load_all(&asset_server);
        
        // Run the app to process asset loading
        app.update();
        
        let loading_state = app.world.resource::<AssetLoadingState>();
        assert_eq!(loading_state.total_assets, 4);
        assert!(game_assets.ui_textures.contains_key("loading.png"));
        assert!(game_assets.ui_fonts.contains_key("main.ttf"));
        assert!(game_assets.vehicle_models.contains_key("truck.glb"));
        assert!(game_assets.audio_engine.contains_key("idle.ogg"));
    }

    #[test]
    fn test_asset_loading_priority() {
        let (mut app, asset_server, mut game_assets) = setup_test_assets();
        
        // Create test assets with different priorities
        std::fs::write("assets/test/ui/textures/critical.png", &[0; 100]).unwrap();
        std::fs::write("assets/test/vehicles/models/high.glb", &[0; 100]).unwrap();
        std::fs::write("assets/test/terrain/textures/medium.png", &[0; 100]).unwrap();
        std::fs::write("assets/test/audio/environment/low.ogg", &[0; 100]).unwrap();
        
        game_assets.load_all(&asset_server);
        
        // Run one update to start loading
        app.update();
        
        let loading_state = app.world.resource::<AssetLoadingState>();
        assert_eq!(loading_state.current_priority, LoadPriority::Critical);
        
        // Run until critical assets are loaded
        while app.world.resource::<AssetLoadingState>().current_priority == LoadPriority::Critical {
            app.update();
        }
        
        assert!(game_assets.ui_textures.contains_key("critical.png"));
        assert_eq!(app.world.resource::<AssetLoadingState>().current_priority, LoadPriority::High);
    }

    #[test]
    fn test_asset_loading_progress() {
        let (mut app, asset_server, mut game_assets) = setup_test_assets();
        
        // Create multiple test assets
        for i in 0..5 {
            std::fs::write(format!("assets/test/ui/textures/test{}.png", i), &[0; 100]).unwrap();
        }
        
        game_assets.load_all(&asset_server);
        
        // Initial state
        app.update();
        let initial_state = app.world.resource::<AssetLoadingState>();
        assert_eq!(initial_state.total_assets, 5);
        assert_eq!(initial_state.loaded_assets, 0);
        
        // Run until complete
        while !app.world.resource::<AssetLoadingState>().is_complete {
            app.update();
        }
        
        let final_state = app.world.resource::<AssetLoadingState>();
        assert_eq!(final_state.loaded_assets, 5);
        assert!(final_state.is_complete);
    }

    #[test]
    fn test_hot_reload() {
        let (mut app, asset_server, mut game_assets) = setup_test_assets();
        
        // Create initial asset
        std::fs::write("assets/test/ui/textures/reload_test.png", &[0; 100]).unwrap();
        
        game_assets.load_all(&asset_server);
        app.update();
        
        // Modify the asset
        std::fs::write("assets/test/ui/textures/reload_test.png", &[1; 100]).unwrap();
        
        // Trigger hot reload
        #[cfg(debug_assertions)]
        game_assets.hot_reload(&asset_server);
        
        app.update();
        
        // Verify the asset was reloaded
        #[cfg(debug_assertions)]
        assert!(game_assets.ui_textures.contains_key("reload_test.png"));
    }

    #[test]
    fn test_failed_asset_loading() {
        let (mut app, asset_server, mut game_assets) = setup_test_assets();
        
        // Create an invalid asset
        std::fs::write("assets/test/vehicles/models/invalid.glb", &[0; 1]).unwrap();
        
        game_assets.load_all(&asset_server);
        
        // Run until loading completes or fails
        while !app.world.resource::<AssetLoadingState>().is_complete {
            app.update();
        }
        
        let final_state = app.world.resource::<AssetLoadingState>();
        assert_eq!(final_state.failed_assets, 1);
        assert!(!game_assets.vehicle_models.contains_key("invalid.glb"));
    }

    #[test]
    fn test_loading_queue() {
        let (mut app, asset_server, mut game_assets) = setup_test_assets();
        
        // Create test assets
        std::fs::write("assets/test/ui/textures/queue1.png", &[0; 100]).unwrap();
        std::fs::write("assets/test/ui/textures/queue2.png", &[0; 100]).unwrap();
        
        game_assets.load_all(&asset_server);
        app.update();
        
        let loading_state = app.world.resource::<AssetLoadingState>();
        assert_eq!(loading_state.loading_queue.len(), 2);
        
        // Run until queue is empty
        while !app.world.resource::<AssetLoadingState>().loading_queue.is_empty() {
            app.update();
        }
        
        assert!(game_assets.ui_textures.contains_key("queue1.png"));
        assert!(game_assets.ui_textures.contains_key("queue2.png"));
    }

    #[test]
    fn test_current_loading_priority() {
        let (mut app, asset_server, mut game_assets) = setup_test_assets();
        
        // Create assets of different priorities
        std::fs::write("assets/test/ui/textures/ui.png", &[0; 100]).unwrap(); // Critical
        std::fs::write("assets/test/vehicles/models/car.glb", &[0; 100]).unwrap(); // High
        std::fs::write("assets/test/terrain/textures/ground.png", &[0; 100]).unwrap(); // Medium
        std::fs::write("assets/test/audio/environment/ambient.ogg", &[0; 100]).unwrap(); // Low
        
        game_assets.load_all(&asset_server);
        
        // Check priority transitions
        let priorities = [
            LoadPriority::Critical,
            LoadPriority::High,
            LoadPriority::Medium,
            LoadPriority::Low,
        ];
        
        for expected_priority in priorities.iter() {
            while app.world.resource::<AssetLoadingState>().current_priority == *expected_priority {
                app.update();
            }
        }
        
        assert!(app.world.resource::<AssetLoadingState>().is_complete);
    }

    #[test]
    fn test_partial_asset_loading() {
        let (mut app, asset_server, mut game_assets) = setup_test_assets();
        
        // Only create critical assets
        std::fs::write("assets/test/ui/textures/loading.png", &[0; 100]).unwrap();
        std::fs::write("assets/test/ui/fonts/critical.ttf", &[0; 100]).unwrap();
        
        game_assets.load_all(&asset_server);
        
        // Run until complete
        while !app.world.resource::<AssetLoadingState>().is_complete {
            app.update();
        }
        
        let final_state = app.world.resource::<AssetLoadingState>();
        assert_eq!(final_state.total_assets, 2);
        assert_eq!(final_state.loaded_assets, 2);
        assert_eq!(final_state.failed_assets, 0);
        assert!(game_assets.ui_textures.contains_key("loading.png"));
        assert!(game_assets.ui_fonts.contains_key("critical.ttf"));
    }

    // Cleanup after tests
    impl Drop for GameAssets {
        fn drop(&mut self) {
            std::fs::remove_dir_all("assets/test").unwrap_or_default();
        }
    }
} 