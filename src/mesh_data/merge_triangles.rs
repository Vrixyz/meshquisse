use std::i32;

use bevy::{math::Vec3Swizzles, prelude::*, utils::HashMap};

use crate::{
    interact_mesh::*,
    tools::{self, TriangleMesh},
    trianglemerger::{MeshMerger, Polygon, Vertex},
};
use polyanya as PA;
use polyanya::Mesh as PAMesh;

use super::only_triangles::TriangleMeshData;

/// Optimized data structure to be closer to the navmesh.
#[derive(Default, Debug, Component)]
pub struct ConvexPolygonsMeshData {
    pub mesh_vertices: Vec<Vertex>,
    pub mesh_polygons: Vec<Polygon>,
    pub invalid_polygon_ids: Vec<u32>,
}

impl From<&MeshMerger> for ConvexPolygonsMeshData {
    fn from(mesh_merger: &MeshMerger) -> Self {
        ConvexPolygonsMeshData {
            mesh_vertices: mesh_merger.mesh_vertices.clone(),
            mesh_polygons: mesh_merger.mesh_polygons.clone(),
            invalid_polygon_ids: mesh_merger
                .mesh_polygons
                .iter()
                .enumerate()
                .filter_map(|(p_index, p)| {
                    if mesh_merger.is_polygon_merged_into_other(p_index as u32) {
                        Some(p_index as u32)
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

impl From<&TriangleMeshData> for ConvexPolygonsMeshData {
    fn from(triangle_mesh_data: &TriangleMeshData) -> Self {
        let mut convex_polygons_data = ConvexPolygonsMeshData {
            mesh_vertices: triangle_mesh_data
                .0
                .positions
                .iter()
                .map(|p| Vertex {
                    p: *p,
                    polygons: Vec::new(),
                })
                .collect(),
            mesh_polygons: (0..triangle_mesh_data.0.indices.len() / 3)
                .map(|i| {
                    let i = i as u32;
                    Polygon {
                        num_traversable: 0,
                        area: 0.0,
                        vertices: [i * 3, i * 3 + 1, i * 3 + 2]
                            .iter()
                            .map(|local_index| triangle_mesh_data.0.indices[*local_index as usize])
                            .collect(),
                        polygons: Vec::new(),
                    }
                })
                .collect(),
            invalid_polygon_ids: default(),
        };
        // TODO: compute edges and associated polygons.
        // as all triangles are oriented the same way (clockwise or counter clockwise),
        // we can make a hashset<edge, polygon>, or <(vertex_id, vertex_id), polygon_id>
        // There's only one polygn possible in 2d because <v1, v2> is <v2, v1> for the other polygon.
        // In 3d there could be more than 1 polygons linked to a same edge
        let mut hash_edges: HashMap<[u32; 2], u32> = HashMap::new();

        for (polygon_index, polygon) in convex_polygons_data.mesh_polygons.iter().enumerate() {
            for local_vertex_id in 0..polygon.vertices.len() {
                let vertex_ids = [
                    polygon.vertices[local_vertex_id],
                    polygon.vertices[(local_vertex_id + 1) % polygon.vertices.len()],
                ];
                let edge = vertex_ids;
                hash_edges.insert(edge, polygon_index as u32);
            }
        }
        for (polygon_index, polygon) in convex_polygons_data.mesh_polygons.iter_mut().enumerate() {
            for local_vertex_id in 0..polygon.vertices.len() {
                // Get edge in reverse order (to be the order of the registered associated polygon)
                let vertex_ids = [
                    polygon.vertices[(local_vertex_id + 1) % polygon.vertices.len()],
                    polygon.vertices[local_vertex_id],
                ];
                // Update the vertex neighbour polygons data
                // FIXME: neighbour polygon should be ordered clockwise (or ccw?) according to documentation,
                // but it's not the case here.
                // At the time of writing, we don't use that data it doesn't matter much.
                let vertex1 = &mut convex_polygons_data.mesh_vertices[vertex_ids[1] as usize];
                vertex1.polygons.push(polygon_index as i32);

                // Update the polygon data
                if hash_edges.contains_key(&vertex_ids) {
                    polygon.polygons.push(hash_edges[&vertex_ids] as i32);
                    polygon.num_traversable += 1;
                } else {
                    polygon.polygons.push(-1);
                }
            }
            polygon.area =
                MeshMerger::get_area(&convex_polygons_data.mesh_vertices, &polygon.vertices);
        }
        convex_polygons_data
    }
}

impl IntoPAMesh for ConvexPolygonsMeshData {
    fn to_pa_mesh(&self) -> PAMesh {
        let pa_mesh = PAMesh::new(
            self.mesh_vertices
                .iter()
                .map(|v| PA::Vertex::new(v.p, v.polygons.iter().map(|p| *p as isize).collect()))
                .collect(),
            self.mesh_polygons
                .iter()
                .enumerate()
                .filter(|(p_index, p)| {
                    self.invalid_polygon_ids.contains(&(*p_index as u32)) == false
                })
                .map(|(_, p)| PA::Polygon::new(p.vertices.clone(), false))
                .collect(),
        );
        pa_mesh
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
        use bevy::render::{mesh::Indices, prelude::*, render_resource::PrimitiveTopology};

        let indices_polygons = self
            .mesh_polygons
            .iter()
            .enumerate()
            .filter(|(p_index, p)| self.invalid_polygon_ids.contains(&(*p_index as u32)) == false)
            .map(|(_, p)| {
                (2..p.vertices.len())
                    .flat_map(|i| [p.vertices[0], p.vertices[i], p.vertices[i - 1]])
            });
        let positions = indices_polygons.clone().map(|polygon_indices| {
            polygon_indices.map(|vertex_index| self.mesh_vertices[vertex_index as usize].p)
        });
        let nb_polygons = self.mesh_polygons.len() - self.invalid_polygon_ids.len();
        let nb_vertices = indices_polygons.clone().flatten().count();

        let mut new_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        new_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            positions
                .clone()
                .flatten()
                .map(|p| [p.x, 0.0, p.y])
                .collect::<Vec<[f32; 3]>>(),
        );
        new_mesh.set_indices(Some(Indices::U32(
            (0..nb_vertices).map(|v| v as u32).collect(),
        )));
        new_mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            (0..nb_vertices)
                .map(|_| [0.0, 1.0, 0.0])
                .collect::<Vec<[f32; 3]>>(),
        );
        new_mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            positions
                .clone()
                .flatten()
                .map(|v| [v.x, v.y])
                .collect::<Vec<[f32; 2]>>(),
        );
        let colors: Vec<[f32; 4]> = indices_polygons
            .clone()
            .enumerate()
            .flat_map(|(index, polygon_vertices)| {
                fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
                    ((1.0 - t) * v0) + (t * v1)
                }
                let saturation = lerp(0.3f32, 1f32, (index as f32 / nb_polygons as f32) % 1f32);
                let color = Color::hsl(
                    ((index as f32 / nb_polygons as f32) * 360f32 + index as f32 * 12313f32)
                        % 360f32,
                    saturation,
                    0.5f32,
                );
                let color = [color.r(), color.g(), color.b(), 1f32];
                (0..polygon_vertices.count()).map(move |_| color)
            })
            .collect();
        new_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        new_mesh
    }

    fn update_mesh(&self, mesh: &mut Mesh) {
        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(ref mut positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            let indices_polygons = self
                .mesh_polygons
                .iter()
                .enumerate()
                .filter(|(p_index, p)| {
                    self.invalid_polygon_ids.contains(&(*p_index as u32)) == false
                })
                .map(|(_, p)| {
                    (2..p.vertices.len())
                        .flat_map(|i| [p.vertices[0], p.vertices[i], p.vertices[i - 1]])
                });
            let new_positions = indices_polygons.clone().map(|polygon_indices| {
                polygon_indices.map(|vertex_index| self.mesh_vertices[vertex_index as usize].p)
            });
            *positions = new_positions
                .clone()
                .flatten()
                .map(|p| [p.x, 0.0, p.y])
                .collect::<Vec<[f32; 3]>>();
        }
    }
}
