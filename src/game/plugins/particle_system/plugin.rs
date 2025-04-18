use bevy::prelude::*;

use super::{
    buffer::ParticleBufferManager,
    compute::{ParticleComputePipeline, dispatch_particle_compute, update_particle_params},
    emitter::update_emitter_transforms,
    material::{ParticleMaterialPipeline, update_material_params, create_material_bind_groups},
    particle::{ParticleSystem, ParticleSystemSettings},
    sorting::{ParticleSortPipeline, dispatch_particle_sort, init_particle_indices},
};

/// Plugin for managing particle systems
pub struct ParticleSystemPlugin;

impl Plugin for ParticleSystemPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<ParticleSystemSettings>()
            .init_resource::<ParticleComputePipeline>()
            .init_resource::<ParticleSortPipeline>()
            .init_resource::<ParticleMaterialPipeline>();

        // Add systems
        app.add_systems(Update, (
            update_emitter_transforms,
            update_particle_params,
            init_particle_indices,
            update_material_params,
            create_material_bind_groups,
        ));

        // Add render systems
        app.add_systems(Render, (
            dispatch_particle_compute,
            dispatch_particle_sort,
            dispatch_particle_render,
        ).chain());
    }
}

/// System to update emitter transforms
fn update_emitter_transforms(mut emitters: Query<(&mut Transform, &ParticleSystem)>) {
    for (mut transform, _) in emitters.iter_mut() {
        // Update transform based on emitter settings
        // This is a placeholder - implement actual transform updates
    }
}

/// System to update particle parameters
fn update_particle_params(mut particles: Query<&mut ParticleSystem>) {
    for mut particle_system in particles.iter_mut() {
        // Update particle parameters based on system settings
        // This is a placeholder - implement actual parameter updates
    }
}

/// System to dispatch particle compute shader
fn dispatch_particle_compute(
    mut particles: Query<&mut ParticleSystem>,
    compute_pipeline: Res<ParticleComputePipeline>,
) {
    for mut particle_system in particles.iter_mut() {
        // Dispatch compute shader for particle simulation
        // This is a placeholder - implement actual compute dispatch
    }
}

/// System to dispatch particle render
fn dispatch_particle_render(
    particles: Query<(&ParticleSystem, &ParticleMaterial)>,
    material_pipeline: Res<ParticleMaterialPipeline>,
) {
    for (particle_system, material) in particles.iter() {
        // Dispatch render pipeline for particle rendering
        // This is a placeholder - implement actual render dispatch
    }
} 