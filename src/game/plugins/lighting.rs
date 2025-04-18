use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::pbr::{CascadedShadowConfig, DirectionalLightShadowMap};

/// Plugin for managing dynamic lighting in the game
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CascadedShadowConfig {
                maximum_distance: 200.0,
                minimum_distance: 0.1,
                num_cascades: 4,
                ..default()
            })
            .insert_resource(DirectionalLightShadowMap { size: 4096 })
            .add_systems(Startup, setup_lighting)
            .add_systems(Update, (
                update_directional_light,
                update_point_lights,
                update_spot_lights,
            ));
    }
}

/// Component for tracking light intensity changes over time
#[derive(Component)]
pub struct LightFlicker {
    pub base_intensity: f32,
    pub flicker_speed: f32,
    pub flicker_intensity: f32,
    pub time: f32,
}

impl Default for LightFlicker {
    fn default() -> Self {
        Self {
            base_intensity: 1000.0,
            flicker_speed: 5.0,
            flicker_intensity: 0.2,
            time: 0.0,
        }
    }
}

/// Component for tracking light color temperature
#[derive(Component)]
pub struct LightTemperature {
    pub kelvin: f32,
}

impl Default for LightTemperature {
    fn default() -> Self {
        Self {
            kelvin: 6500.0, // Daylight white
        }
    }
}

fn setup_lighting(mut commands: Commands) {
    // Main directional light (sun)
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 100000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(50.0, 50.0, 50.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        LightTemperature { kelvin: 5500.0 }, // Daylight temperature
    ));

    // Ambient light for fill
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.1, 0.1, 0.15),
        brightness: 0.3,
    });
}

fn update_directional_light(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), Without<LightFlicker>>,
) {
    for (mut transform, mut light) in query.iter_mut() {
        // Simulate day/night cycle
        let angle = time.elapsed_seconds() * 0.1;
        let height = 50.0;
        let radius = 100.0;
        
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        
        transform.translation = Vec3::new(x, height, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
        
        // Adjust light intensity based on height (day/night)
        let normalized_height = (transform.translation.y / height).clamp(0.0, 1.0);
        light.illuminance = 100000.0 * normalized_height;
    }
}

fn update_point_lights(
    time: Res<Time>,
    mut query: Query<(&mut PointLight, &mut LightFlicker)>,
) {
    for (mut light, mut flicker) in query.iter_mut() {
        flicker.time += time.delta_seconds();
        
        // Calculate flicker effect using noise
        let noise = (flicker.time * flicker.flicker_speed).sin();
        let intensity_mod = 1.0 + noise * flicker.flicker_intensity;
        
        light.intensity = flicker.base_intensity * intensity_mod;
    }
}

fn update_spot_lights(
    time: Res<Time>,
    mut query: Query<(&mut SpotLight, &Transform)>,
) {
    for (mut light, transform) in query.iter_mut() {
        // Adjust spot light properties based on height from ground
        let height = transform.translation.y;
        let range = (height * 2.0).clamp(5.0, 50.0);
        
        light.range = range;
        light.outer_angle = (height * 0.1).clamp(0.5, 1.2);
        light.inner_angle = light.outer_angle * 0.8;
    }
}

/// Helper function to create a point light with flicker effect
pub fn spawn_point_light(
    commands: &mut Commands,
    position: Vec3,
    color: Color,
    intensity: f32,
    range: f32,
    flicker: Option<LightFlicker>,
) -> Entity {
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                color,
                intensity,
                range,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        },
        flicker.unwrap_or_default(),
        LightTemperature::default(),
    )).id()
}

/// Helper function to create a spot light
pub fn spawn_spot_light(
    commands: &mut Commands,
    position: Vec3,
    target: Vec3,
    color: Color,
    intensity: f32,
    range: f32,
    inner_angle: f32,
    outer_angle: f32,
) -> Entity {
    let transform = Transform::from_translation(position)
        .looking_at(target, Vec3::Y);
        
    commands.spawn((
        SpotLightBundle {
            spot_light: SpotLight {
                color,
                intensity,
                range,
                inner_angle,
                outer_angle,
                shadows_enabled: true,
                ..default()
            },
            transform,
            ..default()
        },
        LightTemperature::default(),
    )).id()
}

/// Convert color temperature (in Kelvin) to RGB color
pub fn temperature_to_rgb(temperature: f32) -> Color {
    // Implementation based on approximate Planckian locus
    let temp = temperature / 100.0;
    
    let red = if temp <= 66.0 {
        1.0
    } else {
        let t = temp - 60.0;
        (1.29293618606274514 + 0.0001520143839000175 * t
            - 0.000000198301902438447 * t * t)
            .clamp(0.0, 1.0)
    };
    
    let green = if temp <= 66.0 {
        let t = temp;
        (0.39008157876901960784 + 0.0084817829697596027 * t
            - 0.000026807682491249 * t * t)
            .clamp(0.0, 1.0)
    } else {
        let t = temp - 60.0;
        (1.12989086506368430 - 0.0755148492424 * t
            + 0.0000972440846 * t * t)
            .clamp(0.0, 1.0)
    };
    
    let blue = if temp >= 66.0 {
        1.0
    } else if temp <= 19.0 {
        0.0
    } else {
        let t = temp - 10.0;
        (0.24317186530294098 + 0.07253635284412163 * t
            - 0.000794916641489033 * t * t)
            .clamp(0.0, 1.0)
    };
    
    Color::rgb(red, green, blue)
} 