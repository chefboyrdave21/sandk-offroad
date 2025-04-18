use bevy::prelude::*;
use std::time::Instant;
use crate::game::plugins::weather::{
    time_of_day::TimeOfDayManager,
    sky_system::SkySystem,
    weather_system::WeatherSystem,
};

#[test]
fn benchmark_time_of_day_updates() {
    let mut time_manager = TimeOfDayManager::default();
    let iterations = 10000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        time_manager.update(0.016); // Simulate 60 FPS
    }
    
    let duration = start.elapsed();
    println!(
        "Time of Day Updates: {} iterations in {:?} ({:?} per update)",
        iterations,
        duration,
        duration / iterations as u32
    );
}

#[test]
fn benchmark_sky_system() {
    let mut sky = SkySystem::default();
    let time_manager = TimeOfDayManager::default();
    let iterations = 10000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        sky.update(&time_manager);
    }
    
    let duration = start.elapsed();
    println!(
        "Sky System Updates: {} iterations in {:?} ({:?} per update)",
        iterations,
        duration,
        duration / iterations as u32
    );
}

#[test]
fn benchmark_reflection_probe_updates() {
    let mut sky = SkySystem::default();
    let mut time_manager = TimeOfDayManager::default();
    let iterations = 1000; // Fewer iterations as this is more intensive
    let start = Instant::now();
    
    for i in 0..iterations {
        // Simulate different times of day
        time_manager.set_time((i as f32 / iterations as f32) * 24.0);
        sky.update(&time_manager);
        let (zenith, horizon, ground) = sky.get_sky_colors();
        let sun_dir = time_manager.get_sun_direction();
        let moon_dir = time_manager.get_moon_direction();
        
        // Simulate probe updates
        let _ = (zenith, horizon, ground, sun_dir, moon_dir);
    }
    
    let duration = start.elapsed();
    println!(
        "Reflection Probe Updates: {} iterations in {:?} ({:?} per update)",
        iterations,
        duration,
        duration / iterations as u32
    );
}

#[test]
fn benchmark_full_environment_update() {
    let mut time_manager = TimeOfDayManager::default();
    let mut sky = SkySystem::default();
    let mut weather = WeatherSystem::default();
    let iterations = 1000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let delta = 0.016; // Simulate 60 FPS
        time_manager.update(delta);
        sky.update(&time_manager);
        weather.update_with_time(&time_manager, delta);
    }
    
    let duration = start.elapsed();
    println!(
        "Full Environment Updates: {} iterations in {:?} ({:?} per update)",
        iterations,
        duration,
        duration / iterations as u32
    );
} 