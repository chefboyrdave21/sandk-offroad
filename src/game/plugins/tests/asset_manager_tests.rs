use super::super::asset_manager::*;
use bevy::prelude::*;
use std::path::PathBuf;
use std::fs;

#[test]
fn test_asset_loading_state() {
    let mut state = AssetLoadingState::default();
    assert_eq!(state.total_assets, 0);
    assert_eq!(state.loaded_assets, 0);
    assert!(state.failed_assets.is_empty());
}

#[test]
fn test_loading_progress() {
    let mut state = AssetLoadingState::default();
    state.total_assets = 4;
    state.loaded_assets = 2;
    
    assert_eq!(state.get_loading_progress(), 0.5);
    assert!(!state.is_loading_complete());
    
    state.loaded_assets = 4;
    assert_eq!(state.get_loading_progress(), 1.0);
    assert!(state.is_loading_complete());
}

#[test]
fn test_vehicle_config_deserialization() {
    let json = r#"{
        "name": "Test Vehicle",
        "mass": 1500.0,
        "suspension_config": {
            "spring_stiffness": 50000.0,
            "damping": 5000.0,
            "travel": 0.3
        },
        "engine_config": {
            "max_power": 300.0,
            "max_torque": 400.0,
            "redline": 7000.0
        },
        "wheel_config": {
            "radius": 0.33,
            "width": 0.245,
            "mass": 20.0
        }
    }"#;
    
    let config: VehicleConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.name, "Test Vehicle");
    assert_eq!(config.mass, 1500.0);
    assert_eq!(config.suspension_config.travel, 0.3);
    assert_eq!(config.engine_config.max_power, 300.0);
    assert_eq!(config.wheel_config.radius, 0.33);
}

#[test]
fn test_asset_loading_plugin() {
    let mut app = App::new();
    app.add_plugin(AssetManagerPlugin);
    
    // Verify plugin initialization
    assert!(app.world.contains_resource::<AssetLoadingState>());
    assert!(app.world.contains_resource::<Assets<VehicleConfig>>());
}

#[test]
fn test_failed_asset_tracking() {
    let mut state = AssetLoadingState::default();
    state.failed_assets.push("test_vehicle.vehicle.json".to_string());
    
    assert_eq!(state.failed_assets.len(), 1);
    assert_eq!(state.failed_assets[0], "test_vehicle.vehicle.json");
}

#[test]
fn test_empty_directory_loading() {
    let mut state = AssetLoadingState::default();
    let app = App::new();
    let asset_server = app.world.resource::<AssetServer>();
    
    let temp_dir = std::env::temp_dir();
    let handles = state.load_vehicle_configs(asset_server, &temp_dir);
    
    assert!(handles.is_empty());
    assert_eq!(state.total_assets, 0);
    assert_eq!(state.get_loading_progress(), 1.0);
}

#[test]
fn test_multiple_vehicle_configs_loading() {
    let mut app = App::new();
    app.add_plugin(AssetManagerPlugin);
    
    let mut state = AssetLoadingState::default();
    let asset_server = app.world.resource::<AssetServer>();
    
    // Create temporary test directory
    let temp_dir = std::env::temp_dir().join("vehicle_test");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create test vehicle config files
    let test_configs = vec![
        ("test1.vehicle.json", r#"{"name": "Test1", "mass": 1000.0, "suspension_config": {"spring_stiffness": 50000.0, "damping": 5000.0, "travel": 0.3}, "engine_config": {"max_power": 200.0, "max_torque": 300.0, "redline": 6000.0}, "wheel_config": {"radius": 0.3, "width": 0.2, "mass": 15.0}}"#),
        ("test2.vehicle.json", r#"{"name": "Test2", "mass": 2000.0, "suspension_config": {"spring_stiffness": 60000.0, "damping": 6000.0, "travel": 0.4}, "engine_config": {"max_power": 400.0, "max_torque": 500.0, "redline": 7000.0}, "wheel_config": {"radius": 0.35, "width": 0.25, "mass": 20.0}}"#),
    ];
    
    for (filename, content) in test_configs {
        fs::write(temp_dir.join(filename), content).unwrap();
    }
    
    // Load configs
    let handles = state.load_vehicle_configs(asset_server, &temp_dir);
    
    assert_eq!(handles.len(), 2);
    assert_eq!(state.total_assets, 2);
    
    // Cleanup
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_asset_loading_error_handling() {
    let mut app = App::new();
    app.add_plugin(AssetManagerPlugin);
    
    let mut state = AssetLoadingState::default();
    let asset_server = app.world.resource::<AssetServer>();
    
    // Create temporary test directory
    let temp_dir = std::env::temp_dir().join("vehicle_test_error");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create invalid vehicle config file
    fs::write(
        temp_dir.join("invalid.vehicle.json"),
        "invalid json content"
    ).unwrap();
    
    // Load configs
    let handles = state.load_vehicle_configs(asset_server, &temp_dir);
    
    assert_eq!(handles.len(), 1); // Handle is created but loading will fail
    assert_eq!(state.total_assets, 1);
    
    // Cleanup
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_concurrent_asset_loading() {
    let mut app = App::new();
    app.add_plugin(AssetManagerPlugin);
    
    let mut state = AssetLoadingState::default();
    let asset_server = app.world.resource::<AssetServer>();
    
    // Create temporary test directories
    let temp_dir1 = std::env::temp_dir().join("vehicle_test_1");
    let temp_dir2 = std::env::temp_dir().join("vehicle_test_2");
    fs::create_dir_all(&temp_dir1).unwrap();
    fs::create_dir_all(&temp_dir2).unwrap();
    
    // Create test vehicle config files in both directories
    let test_config = r#"{"name": "Test", "mass": 1000.0, "suspension_config": {"spring_stiffness": 50000.0, "damping": 5000.0, "travel": 0.3}, "engine_config": {"max_power": 200.0, "max_torque": 300.0, "redline": 6000.0}, "wheel_config": {"radius": 0.3, "width": 0.2, "mass": 15.0}}"#;
    
    fs::write(temp_dir1.join("test1.vehicle.json"), test_config).unwrap();
    fs::write(temp_dir2.join("test2.vehicle.json"), test_config).unwrap();
    
    // Load configs from both directories
    let handles1 = state.load_vehicle_configs(asset_server, &temp_dir1);
    let handles2 = state.load_vehicle_configs(asset_server, &temp_dir2);
    
    assert_eq!(handles1.len() + handles2.len(), 2);
    assert_eq!(state.total_assets, 2);
    
    // Cleanup
    fs::remove_dir_all(temp_dir1).unwrap();
    fs::remove_dir_all(temp_dir2).unwrap();
}

#[test]
fn test_asset_loading_state_updates() {
    let mut app = App::new();
    app.add_plugin(AssetManagerPlugin);
    
    let mut state = app.world.resource_mut::<AssetLoadingState>();
    
    // Simulate loading progress
    state.total_assets = 3;
    state.loaded_assets = 1;
    assert_eq!(state.get_loading_progress(), 1.0/3.0);
    
    state.loaded_assets = 2;
    assert_eq!(state.get_loading_progress(), 2.0/3.0);
    
    state.loaded_assets = 3;
    assert_eq!(state.get_loading_progress(), 1.0);
    assert!(state.is_loading_complete());
}

#[test]
fn test_asset_loading_system() {
    let mut app = App::new();
    app.add_plugin(AssetManagerPlugin);
    
    // Add test system to check asset loading
    app.add_system(check_asset_loading);
    
    // Run one frame
    app.update();
    
    // Verify the system ran and the state is accessible
    assert!(app.world.contains_resource::<AssetLoadingState>());
} 