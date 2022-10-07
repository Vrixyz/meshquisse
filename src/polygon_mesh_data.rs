use bevy::{math::Vec3Swizzles, prelude::*};

use crate::{
    interact_mesh::*,
    meshmerger::{MeshMerger, Polygon, Vertex},
    tools::{self, TriangleMesh},
};
use polyanya::Mesh as PAMesh;

/// Meant to be used in correlation with `ShowAndUpdateMesh` and/or `EditableMesh`
#[derive(Component, Default)]
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

/// Optimized data structure to be closer to the navmesh.
#[derive(Default, Component)]
pub struct ConvexPolygonsMeshData {
    pub mesh_vertices: Vec<Vertex>,
    pub mesh_polygons: Vec<Polygon>,
}

impl From<&MeshMerger> for ConvexPolygonsMeshData {
    fn from(mesh_merger: &MeshMerger) -> Self {
        ConvexPolygonsMeshData {
            mesh_vertices: mesh_merger.mesh_vertices.clone(),
            mesh_polygons: mesh_merger.mesh_polygons.clone(),
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

impl IntoPAMesh for ConvexPolygonsMeshData {
    fn to_pa_mesh(&self) -> PAMesh {
        // TODO: we can make this more performant because data is very similar :)
        tools::navmesh_from_trimesh(&(TriangleMeshData::from(self)).0)
    }
}

impl UpdateVertex for ConvexPolygonsMeshData {
    fn update_vertex(&mut self, vertex_index: u32, position: Vec3) {
        self.mesh_vertices[vertex_index as usize].p = position.xz();
    }

    fn iter_positions(&self) -> Vec<Vec2> {
        // FIXME: horrible perf, but what would be the generic version of that iteration ?
        // we could pass a function to go through all positions, or leverge Into<Vec2> ?
        self.mesh_vertices.iter().map(|v| v.p).collect()
    }
}

impl IntoBevyMesh for ConvexPolygonsMeshData {
    fn to_bevy_mesh(&self) -> Mesh {
        tools::bevymesh_from_trimesh(&TriangleMeshData::from(self).0)
    }

    fn update_mesh(&self, mesh: &mut Mesh) {
        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(ref mut positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            let triangle_data = TriangleMeshData::from(self);
            positions
                .iter_mut()
                .enumerate()
                .for_each(|(index, position)| {
                    let pos_data = triangle_data.0.positions[index];
                    position[0] = pos_data.x;
                    position[1] = 0f32;
                    position[2] = pos_data.y;
                });
        }
    }
}
