use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use super::trail_plugin::{Trail, TrailPoints, TrailPoint};

#[derive(Component)]
pub struct TrailMesh;

pub fn generate_trail_mesh(points: &[TrailPoint], trail: &Trail) -> Mesh {
    let point_count = points.len();
    if point_count < 2 {
        return Mesh::new(PrimitiveTopology::TriangleList);
    }

    let vertex_count = point_count * 2;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);

    for (i, point) in points.iter().enumerate() {
        let next = if i < point_count - 1 {
            points[i + 1].position
        } else {
            point.position + point.velocity.normalize()
        };

        let prev = if i > 0 {
            points[i - 1].position
        } else {
            point.position - point.velocity.normalize()
        };

        let forward = (next - prev).normalize();
        let right = forward.cross(Vec3::Y).normalize() * trail.width;

        positions.push((point.position + right).to_array());
        positions.push((point.position - right).to_array());

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        let u = i as f32 / (point_count - 1) as f32;
        uvs.push([u, 0.0]);
        uvs.push([u, 1.0]);

        let alpha = (1.0 - (points[i].creation_time - points[0].creation_time) / trail.fade_time)
            .clamp(0.0, 1.0);
        colors.push([1.0, 1.0, 1.0, alpha]);
        colors.push([1.0, 1.0, 1.0, alpha]);
    }

    let mut indices = Vec::with_capacity((point_count - 1) * 6);
    for i in 0..point_count - 1 {
        let i0 = i * 2;
        let i1 = i0 + 1;
        let i2 = i0 + 2;
        let i3 = i0 + 3;

        indices.extend_from_slice(&[i0, i1, i2]);
        indices.extend_from_slice(&[i1, i3, i2]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}

pub fn update_trail_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Trail, &TrailPoints, Option<&Handle<Mesh>>), With<TrailMesh>>,
) {
    for (entity, trail, points, mesh_handle) in query.iter() {
        let new_mesh = generate_trail_mesh(&points.0, trail);
        
        match mesh_handle {
            Some(handle) => {
                if let Some(mesh) = meshes.get_mut(handle) {
                    *mesh = new_mesh;
                }
            }
            None => {
                let handle = meshes.add(new_mesh);
                commands.entity(entity).insert(handle);
            }
        }
    }
} 