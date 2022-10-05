use crate::{
    interact_mesh::TriangleMeshData,
    meshmerger::{MeshMerger, Polygon, Vertex},
};

pub struct ConvexPolygons {
    pub mesh_vertices: Vec<Vertex>,
    pub mesh_polygons: Vec<Polygon>,
}

impl From<MeshMerger> for ConvexPolygons {
    fn from(mesh_merger: MeshMerger) -> Self {
        ConvexPolygons {
            mesh_vertices: mesh_merger.mesh_vertices.clone(),
            mesh_polygons: mesh_merger.mesh_polygons.clone(),
        }
    }
}

impl From<ConvexPolygons> for TriangleMeshData {
    fn from(convex_polygons: ConvexPolygons) -> Self {
        TriangleMeshData(crate::tools::TriangleMesh {
            indices: convex_polygons
                .mesh_polygons
                .iter()
                .flat_map(|p| {
                    (2..p.vertices.len())
                        .flat_map(|i| [p.vertices[0], p.vertices[i], p.vertices[i - 1]])
                })
                .map(|v| v as u32)
                .collect(),
            positions: convex_polygons.mesh_vertices.iter().map(|v| v.p).collect(),
        })
    }
}

pub struct PolygonMeshData(pub ConvexPolygons);
