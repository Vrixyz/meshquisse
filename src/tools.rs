use bevy::prelude::{Vec2, Vec3};
use polyanya::{Mesh as PAMesh, Polygon, Vertex};

#[derive(Debug, PartialEq)]
pub struct TriangleMesh {
    pub indices: Vec<u32>,
    pub positions: Vec<Vec2>,
}

/// Triangulates convex polygons, complexity is O(n)
pub fn triangulate(navmesh: &PAMesh) -> Vec<u32> {
    navmesh
        .polygons
        .iter()
        .flat_map(|p| {
            (2..p.vertices.len()).flat_map(|i| [p.vertices[0], p.vertices[i], p.vertices[i - 1]])
        })
        .map(|v| v as u32)
        .collect()
}

pub fn create_grid_trimesh(width: u32, height: u32, spacing: f32) -> TriangleMesh {
    let to_index = |x: u32, y: u32| y * height + x;
    let positions: Vec<Vec2> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                let position: Vec2 = [x as f32, y as f32].into();
                position * spacing
            })
        })
        .collect();
    let indices: Vec<u32> = (1..height)
        .flat_map(|y| {
            (1..width).flat_map(move |x| {
                let mut triangles = [
                    // bottom left triangle
                    to_index(x - 1, y - 1),
                    to_index(x, y - 1),
                    to_index(x - 1, y),
                    // top right triangle
                    to_index(x, y),
                    to_index(x - 1, y),
                    to_index(x, y - 1),
                ];
                triangles
            })
        })
        .collect();
    TriangleMesh { indices, positions }
}

/// Returns an polyanya::Mesh, without any complex transformations,
/// polygons are kept as triangles.
/// (not implemented) For a more optimal solution, consider calling trimesh_to_convex_polygon_mesh()
pub fn navmesh_from_trimesh(triangles_mesh: &TriangleMesh) -> PAMesh {
    let mut vertices: Vec<Vertex> = triangles_mesh
        .positions
        .iter()
        .map(|position| Vertex {
            coords: *position,
            is_corner: true,
            polygons: vec![],
        })
        .collect();
    let polygons: Vec<_> = (0..triangles_mesh.indices.len() / 3)
        .map(|i| {
            let i = i * 3;
            let indexes = vec![
                triangles_mesh.indices[i] as usize,
                triangles_mesh.indices[i + 1] as usize,
                triangles_mesh.indices[i + 2] as usize,
            ];
            Polygon::new(indexes, false)
        })
        .collect();
    for (vertex_index, mut vertex) in vertices.iter_mut().enumerate() {
        vertex.polygons = polygons
            .iter()
            .enumerate()
            .filter_map(|(polygon_index, p)| {
                p.vertices
                    .contains(&vertex_index)
                    .then_some(polygon_index as isize)
            })
            .collect::<Vec<isize>>();
    }
    PAMesh::new(vertices, polygons)
}

/// Returns an bevy::Mesh with triangles
pub fn bevymesh_from_trimesh(triangles_mesh: &TriangleMesh) -> bevy::prelude::Mesh {
    use bevy::render::{mesh::Indices, prelude::*, render_resource::PrimitiveTopology};

    let mut new_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    new_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        triangles_mesh
            .positions
            .iter()
            .map(|v| [v.x, 0.0, v.y])
            .collect::<Vec<[f32; 3]>>(),
    );
    new_mesh.set_indices(Some(Indices::U32(
        triangles_mesh.indices.iter().rev().copied().collect(),
    )));
    new_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        (0..triangles_mesh.positions.len())
            .into_iter()
            .map(|_| [0.0, 1.0, 0.0])
            .collect::<Vec<[f32; 3]>>(),
    );

    new_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        triangles_mesh
            .positions
            .iter()
            .map(|v| [v.x, v.y])
            .collect::<Vec<[f32; 2]>>(),
    );
    new_mesh
}

mod test {
    use bevy::prelude::Vec2;
    use polyanya::{Polygon, Vertex};

    use super::{create_grid_trimesh, navmesh_from_trimesh, TriangleMesh};

    fn trimesh_3_3_10() -> TriangleMesh {
        TriangleMesh {
            indices: vec![
                0, 3, 1, 4, 1, 3, 1, 4, 2, 5, 2, 4, 3, 6, 4, 7, 4, 6, 4, 7, 5, 8, 5, 7,
            ],
            positions: vec![
                Vec2 { x: 0.0, y: 0.0 },
                Vec2 { x: 10.0, y: 0.0 },
                Vec2 { x: 20.0, y: 0.0 },
                Vec2 { x: 0.0, y: 10.0 },
                Vec2 { x: 10.0, y: 10. },
                Vec2 { x: 20.0, y: 10. },
                Vec2 { x: 0.0, y: 20.0 },
                Vec2 { x: 10.0, y: 20. },
                Vec2 { x: 20.0, y: 20. },
            ],
        }
    }

    #[test]
    fn test_trimesh_3_3_10() {
        let trimesh = create_grid_trimesh(3, 3, 10f32);
        assert_eq!(trimesh, trimesh_3_3_10())
    }
    #[test]
    fn test_navmesh_from_trimesh_3_3_10() {
        let trimesh = trimesh_3_3_10();
        let navmesh = navmesh_from_trimesh(&trimesh);
        assert_eq!(
            navmesh.vertices,
            vec![
                Vertex {
                    coords: Vec2::new(0.0, 0.0,),
                    polygons: vec![0,],
                    is_corner: true,
                },
                Vertex {
                    coords: Vec2::new(10.0, 0.0,),
                    polygons: vec![0, 1, 2,],
                    is_corner: true,
                },
                Vertex {
                    coords: Vec2::new(20.0, 0.0,),
                    polygons: vec![2, 3,],
                    is_corner: true,
                },
                Vertex {
                    coords: Vec2::new(0.0, 10.0,),
                    polygons: vec![0, 1, 4,],
                    is_corner: true,
                },
                Vertex {
                    coords: Vec2::new(10.0, 10.0,),
                    polygons: vec![1, 2, 3, 4, 5, 6,],
                    is_corner: true,
                },
                Vertex {
                    coords: Vec2::new(20.0, 10.0,),
                    polygons: vec![3, 6, 7,],
                    is_corner: true,
                },
                Vertex {
                    coords: Vec2::new(0.0, 20.0,),
                    polygons: vec![4, 5,],
                    is_corner: true,
                },
                Vertex {
                    coords: Vec2::new(10.0, 20.0,),
                    polygons: vec![5, 6, 7,],
                    is_corner: true,
                },
                Vertex {
                    coords: Vec2::new(20.0, 20.0,),
                    polygons: vec![7,],
                    is_corner: true,
                },
            ]
        );
        assert_eq!(
            navmesh
                .polygons
                .into_iter()
                .map(|v| (v.vertices, v.is_one_way))
                .collect::<Vec<_>>(),
            [
                (vec![0, 3, 1,], false,),
                (vec![4, 1, 3,], false,),
                (vec![1, 4, 2,], false,),
                (vec![5, 2, 4,], false,),
                (vec![3, 6, 4,], false,),
                (vec![7, 4, 6,], false,),
                (vec![4, 7, 5,], false,),
                (vec![8, 5, 7,], false,),
            ]
        )
    }
}
