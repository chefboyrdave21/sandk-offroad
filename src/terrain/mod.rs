use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy_rapier3d::prelude::*;
use noise::{NoiseFn, Perlin};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_terrain);
    }
}

#[derive(Component)]
pub struct TerrainChunk;

fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunk_size = 100.0;
    let resolution = 100;
    let height_scale = 5.0;
    let noise = Perlin::new(0);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    // Generate vertices
    for z in 0..=resolution {
        for x in 0..=resolution {
            let px = (x as f32 / resolution as f32 - 0.5) * chunk_size;
            let pz = (z as f32 / resolution as f32 - 0.5) * chunk_size;
            
            let noise_x = px * 0.02;
            let noise_z = pz * 0.02;
            let height = noise.get([noise_x as f64, noise_z as f64]) as f32 * height_scale;
            
            vertices.push([px, height, pz]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([x as f32 / resolution as f32, z as f32 / resolution as f32]);
        }
    }

    // Generate indices
    for z in 0..resolution {
        for x in 0..resolution {
            let top_left = z * (resolution + 1) + x;
            let top_right = top_left + 1;
            let bottom_left = (z + 1) * (resolution + 1) + x;
            let bottom_right = bottom_left + 1;

            indices.extend_from_slice(&[
                top_left as u32,
                bottom_left as u32,
                top_right as u32,
                top_right as u32,
                bottom_left as u32,
                bottom_right as u32,
            ]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices.clone())));

    // Create the terrain entity
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.5, 0.3),
                perceptual_roughness: 0.9,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -2.0, 0.0),
            ..default()
        },
        TerrainChunk,
        RigidBody::Fixed,
        Collider::trimesh(
            vertices.into_iter().map(|v| Vec3::from(v)).collect(),
            indices.chunks(3).map(|i| [i[0], i[1], i[2]]).collect(),
        ),
        Friction::coefficient(0.3),
    ));
} 