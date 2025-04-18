use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(Component, Clone, Default, AsBindGroup)]
pub struct TrailMaterial {
    #[uniform(0)]
    pub color_multiplier: Vec4,
    #[uniform(1)]
    pub use_texture: u32,
    #[uniform(2)]
    pub effect_strength: f32,
    #[uniform(3)]
    pub time: f32,
    #[texture(4)]
    #[sampler(5)]
    pub texture: Option<Handle<Image>>,
}

impl Material for TrailMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/particle_trail.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/particle_trail.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

impl TrailMaterial {
    pub fn new() -> Self {
        Self {
            color_multiplier: Vec4::ONE,
            use_texture: 0,
            effect_strength: 1.0,
            time: 0.0,
            texture: None,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color_multiplier = color;
        self
    }

    pub fn with_texture(mut self, texture: Handle<Image>) -> Self {
        self.texture = Some(texture);
        self.use_texture = 1;
        self
    }

    pub fn with_effect_strength(mut self, strength: f32) -> Self {
        self.effect_strength = strength;
        self
    }

    pub fn update(&mut self, time: f32) {
        self.time = time;
    }
} 