use std::i32;

use bevy::{math::Vec3Swizzles, prelude::*, utils::HashMap};

use crate::{
    interact_mesh::*,
    tools::{self, TriangleMesh},
    trianglemerger::{MeshMerger, Polygon, Vertex},
};
use polyanya as PA;
use polyanya::Mesh as PAMesh;

use super::merge_triangles::ConvexPolygonsMeshData;

/// Meant to be used in correlation with `ShowAndUpdateMesh` and/or `EditableMesh`
#[derive(Component, Debug, Default)]
pub struct TriangleMeshData(pub TriangleMesh);

impl IntoPAMesh for TriangleMeshData {
    fn to_pa_mesh(&self) -> PAMesh {
        tools::navmesh_from_trimesh(&self.0)
    }
}

impl UpdateVertex for TriangleMeshData {
    fn update_vertex(&mut self, vertex_index: u32, position: Vec3) {
        self.0.positions[vertex_index as usize] = position.xz();
    }

    fn iter_positions(&self) -> Vec<Vec2> {
        self.0.positions.clone()
    }
}

impl IntoBevyMesh for TriangleMeshData {
    fn to_bevy_mesh(&self) -> Mesh {
        tools::bevymesh_from_trimesh(&self.0)
    }

    fn update_mesh(&self, mesh: &mut Mesh) {
        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(ref mut positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            positions
                .iter_mut()
                .enumerate()
                .for_each(|(index, position)| {
                    let pos_data = self.0.positions[index];
                    position[0] = pos_data.x;
                    position[1] = 0f32;
                    position[2] = pos_data.y;
                });
        }
    }
}

impl From<&ConvexPolygonsMeshData> for TriangleMeshData {
    fn from(convex_polygons: &ConvexPolygonsMeshData) -> Self {
        TriangleMeshData(crate::tools::TriangleMesh {
            indices: convex_polygons
                .mesh_polygons
                .iter()
                .flat_map(|p| {
                    (2..p.vertices.len())
                        .flat_map(|i| [p.vertices[0], p.vertices[i - 1], p.vertices[i]])
                })
                .map(|v| v as u32)
                .collect(),
            positions: convex_polygons.mesh_vertices.iter().map(|v| v.p).collect(),
        })
    }
}
