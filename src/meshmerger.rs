use std::io::{self, BufRead};

use bevy::prelude::Vec2;

/// Credits to https://bitbucket.org/dharabor/pathfinding/src/master/anyangle/polyanya/utils/meshmerger.cpp

#[derive(Default, Debug, Clone, PartialEq)]
pub struct UnionFind {
    parent: Vec<i32>,
}

impl UnionFind {
    fn new(polygon_count: i32) -> Self {
        Self {
            parent: (0..polygon_count).collect(),
        }
    }

    fn find(&self, x: i32) -> i32 {
        if x == -1 {
            return -1;
        }
        self.parent[x as usize]
    }
    fn merge(&mut self, to: i32, from: i32) {
        self.parent.iter_mut().for_each(|elem| {
            if *elem == from {
                *elem = to;
            }
        });
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Vertex {
    pub p: Vec2,
    pub num_polygons: u32,
    pub polygons: Vec<i32>,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Polygon {
    pub num_traversable: u32,
    pub area: f32,
    pub vertices: Vec<u32>,
    /// Stores the original polygons.
    /// To get the actual polygon, do polygon_unions.find on the polygon you get.
    pub polygons: Vec<i32>,
}

struct SearchNode {
    /// Index of poly.
    index: u32,
    /// Area of the best tentative merge.
    area: f32,
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PartialOrd for SearchNode {
    /// Comparison.
    /// Always take the "biggest" search node in a priority queue.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.area.partial_cmp(&other.area)
    }
}
/// Helper to compute the area of a polygon
/// https://www.wikihow.com/Calculate-the-Area-of-a-Polygon#Finding-the-Area-of-Irregular-Polygons
/// It could probably be computed by using `Mat2x2::determinant`
/// Also used for clockwise order detection
fn determinant(row1: &Vec2, row2: &Vec2) -> f32 {
    row1.x * row2.y - row1.y * row2.x
}

/// Circular get
fn getc<T: Copy>(vec: &Vec<T>, index: u32) -> T {
    vec[index as usize % vec.len()]
}

#[derive(Debug, PartialEq)]
pub struct MergeInfo {
    pub polygon_to: i32,
    /// index of the start vertex within the polygon we want to merge into
    pub to_index: u32,
    /// index of the polygon_from, from the root polygons
    pub polygon_from: i32,
    /// index of the same vertex but from the polygon we merge from.
    pub from_index: u32,
}

#[derive(Debug, PartialEq)]
pub enum ImpossibleMergeInfo {
    ToMergedIntoOther,
    NoNeighbour,
    FirstVertexClockwise,
    SecondVertexClockwise,
}

#[derive(Debug)]
pub struct MeshMerger {
    /// We'll keep all vertices,
    /// but we may throw them out in the end if num_polygons is 0.
    /// We'll figure it out once we're finished.
    pub mesh_vertices: Vec<Vertex>,
    pub mesh_polygons: Vec<Polygon>,
    pub polygon_unions: UnionFind,
}
impl MeshMerger {
    /// Actually returns double the area of the polygon...
    /// Assume that mesh_vertices is populated and is valid.
    pub fn get_area(mesh_vertices: &[Vertex], area: &Vec<u32>) -> f32 {
        let mut out = 0f32;
        for i in 1..=area.len() {
            out += determinant(
                &mesh_vertices[area[i - 1 as usize % area.len()] as usize].p,
                &mesh_vertices[area[i as usize % area.len()] as usize].p,
            );
        }
        out
    }
    pub fn from_bytes(bytes: &[u8]) -> MeshMerger {
        let mut list_data = Vec::<u32>::default();

        // number of total vertices, called V on reference implementation
        let mut v_nb_vertices = 0;
        // number of total vertices, called P on reference implementation
        let mut p_nb_polygons = 0;
        let mut reader = io::BufReader::new(bytes).lines();
        let line: String = reader.next().unwrap().unwrap();
        if line != "mesh" {
            panic!("First line should be 'mesh'");
        }
        let line: String = reader.next().unwrap().unwrap();
        if line != "2" {
            panic!("second line should be '2'");
        }
        let line: String = reader.next().unwrap().unwrap();
        // (V, P) from https://bitbucket.org/dharabor/pathfinding/src/ce5b02e9d051d5f17addb359429104c0293decaf/anyangle/polyanya/utils/meshmerger.cpp#lines-205
        (v_nb_vertices, p_nb_polygons) = line
            .split_once(' ')
            .map(|(a, b)| {
                (
                    a.parse().expect("failed reading nb_vertices"),
                    b.parse().expect("failed reading nb_polygons"),
                )
            })
            .unwrap();
        let mut mesh_vertices = vec![Vertex::default(); v_nb_vertices];
        let mut mesh_polygons = vec![Polygon::default(); p_nb_polygons];
        let polygon_unions = UnionFind::new(p_nb_polygons as i32);

        for mut vertex in mesh_vertices.iter_mut() {
            let line: String = reader.next().unwrap().unwrap();
            let mut values = line.split(' ');
            // Step: Read vertex coordinates
            let x = values.next().unwrap().parse().unwrap();
            let y = values.next().unwrap().parse().unwrap();
            // Step: Read vertex's neighbour polygons
            let neigbours = values.next().unwrap().parse().unwrap();
            if neigbours < 2 {
                panic!("vertex with less than 2 neigbours");
            }
            vertex.p = Vec2::new(x, y);
            vertex.num_polygons = neigbours;
            let neighbour_values: Vec<i32> = values.map(|v| v.parse().unwrap()).collect();
            if neighbour_values.len() != neigbours as usize {
                panic!("read more neighbours than defined.");
            }
            // Guaranteed to have 2 or more.
            for polygon_index in neighbour_values.into_iter() {
                if polygon_index >= p_nb_polygons as i32 {
                    panic!(
                        "Got a polygon index of {polygon_index} when nb_polygon is {p_nb_polygons}"
                    );
                }
                vertex.polygons.push(polygon_index);
            }
        }
        for mut polygon in mesh_polygons.iter_mut() {
            let line: String = reader.next().unwrap().unwrap();
            let mut values = line.split(' ');

            // Step: Read polygon's vertices (corresponding to neighbouring polygons too)
            let n = values.next().unwrap().parse().unwrap();
            if n < 3 {
                panic!("Invalid number of vertices in polygon (Got {n}).");
            }
            for _ in 0..n {
                let vertex_index: u32 = values.next().unwrap().parse().unwrap();
                if vertex_index >= v_nb_vertices as u32 {
                    panic!("Invalid vertex index when getting polygon");
                }
                polygon.vertices.push(vertex_index);
            }

            // Step: Read polygon's neighbour polygons
            polygon.num_traversable = 0;
            for _ in 0..n {
                let polygon_index: i32 = values.next().unwrap().parse().unwrap();
                if polygon_index >= p_nb_polygons as i32 {
                    panic!("Invalid polygon index when getting polygon");
                }
                if polygon_index != -1 {
                    polygon.num_traversable += 1;
                }
                polygon.polygons.push(polygon_index);
            }
            polygon.polygons.push(polygon.polygons[0]);
            polygon.polygons.remove(0);
            polygon.area = MeshMerger::get_area(&mesh_vertices, &polygon.vertices);
            assert!(polygon.area > 0f32, "Polygon has an area inferior to 0");
        }
        // TODO: check if the file is indeed finished.
        MeshMerger {
            mesh_vertices,
            mesh_polygons,
            polygon_unions,
        }
    }
    /// Checks if points are ordered clockwise
    fn cw(a: &Vec2, b: &Vec2, c: &Vec2) -> bool {
        determinant(&(*b - *a), &(*c - *b)) < -1e-8
    }

    /// Crashes if not correct
    fn is_correct(&self) -> bool {
        for (i, polygon) in self.mesh_polygons.clone().iter().enumerate() {
            for merge_index in 0..polygon.vertices.len() {
                self.can_merge(i as i32, merge_index as u32);
            }
        }
        true
    }
    /// Can polygon x merge with the polygon adjacent to the given edge index ?
    ///
    /// Try to merge `self.mesh_polygons[polygon_to_index]` with `self.mesh_polygons].polygon[vertex_to_index]`
    /// through edge `self.mesh_polygons[polygon_to_index].vertices[vertex_to_index..vertex_to_index + 1]`.
    ///
    /// Return None if `self.mesh_polygons[polygon_to_index]` is a merged polygon.
    /// Return None if resulting polygon would be concave.
    /// Return None if there's no neighbour polygon on that index.
    pub fn can_merge(
        &self,
        polygon_to_index: i32,
        vertex_to_index: u32,
    ) -> Result<MergeInfo, ImpossibleMergeInfo> {
        if self.polygon_unions.find(polygon_to_index) != polygon_to_index {
            return Err(ImpossibleMergeInfo::ToMergedIntoOther);
        }
        // The polygon we want to modify to be the merge result
        let polygon_to = &self.mesh_polygons[polygon_to_index as usize];
        let polygon_from_index = self
            .polygon_unions
            .find(polygon_to.polygons[vertex_to_index as usize]);
        if polygon_from_index == -1 {
            return Err(ImpossibleMergeInfo::NoNeighbour);
        }
        // The polygon we want to merge from
        let polygon_from = &self.mesh_polygons[polygon_from_index as usize];
        debug_assert!(
            polygon_from.vertices.len() != 0,
            "Wrong data: a polygon cannot have 0 vertices."
        );
        // edge 1 in the "to"
        let to_vertice_1 = (
            vertex_to_index,
            polygon_to.vertices[vertex_to_index as usize],
        );
        let to_vertice_2 = (
            vertex_to_index + 1,
            getc(&polygon_to.vertices, vertex_to_index + 1),
        );

        let from_vertice_1 = polygon_from
            .vertices
            .iter()
            .copied()
            .enumerate()
            .find(|v| v.1 == to_vertice_1.1);
        debug_assert!(
            from_vertice_1.is_some(),
            "Wrong data: polygon to merge should share corresponding vertices"
        );
        let from_vertice_1 = from_vertice_1.unwrap();
        // PERF: optimisation possible
        // FIXME: this calculation can occur after the first clockwise check.
        let from_vertice_2 = (
            (from_vertice_1.0 + polygon_from.vertices.len()) - 1,
            getc(
                &polygon_from.vertices,
                (from_vertice_1.0 + polygon_from.vertices.len() - 1) as u32,
            ),
        );
        debug_assert!(
            from_vertice_1.1 == to_vertice_1.1,
            "wrong first edge vertex correspondance"
        );
        debug_assert!(
            from_vertice_2.1 == to_vertice_2.1,
            "wrong second vertex correspondance: {}, {}",
            polygon_to_index.to_string(),
            vertex_to_index.to_string()
        );
        debug_assert!(
            self.polygon_unions
                .find(getc(&polygon_from.polygons, from_vertice_2.0 as u32))
                == polygon_to_index,
            "Neighbour from polygon {polygon_from_index};{vertex_to_index} does not match edge to. {}->{}->{} != {polygon_to_index}",
            from_vertice_2.0,
            getc(&polygon_from.polygons, from_vertice_2.0 as u32),
            self.polygon_unions
                .find(getc(&polygon_from.polygons, from_vertice_2.0 as u32))
        );

        // PERF: optimisation possible
        // FIXME: cw(a) can probable take value from `from_vertice_1.1`

        // The merge will insert vertices between 'from_vertice_1' and 'from_vertice_2'
        // between to_vertice_1 and to_vertice_2

        // check clockwiseness for (from_vertice_1 - 1, from_vertice_1, to_vertice_1 + 1)
        // If the new ones are clockwise, we must return false.
        if Self::cw(
            &self.mesh_vertices[getc(
                &polygon_from.vertices,
                from_vertice_1.0 as u32 + polygon_from.vertices.len() as u32 - 1,
            ) as usize]
                .p,
            &self.mesh_vertices[from_vertice_1.1 as usize].p,
            &self.mesh_vertices[getc(&polygon_to.vertices, to_vertice_1.0 + 1) as usize].p,
        ) {
            return Err(ImpossibleMergeInfo::FirstVertexClockwise);
        }
        // check clockwiseness for (to_vertice_2 - 1, to_vertice_2, from_vertice_1 + 1)
        // If the new ones are clockwise, we must return false.
        if Self::cw(
            &self.mesh_vertices[getc(
                &polygon_to.vertices,
                to_vertice_2.0 as u32 + polygon_to.vertices.len() as u32 - 1,
            ) as usize]
                .p,
            &self.mesh_vertices[from_vertice_2.1 as usize].p,
            &self.mesh_vertices[getc(&polygon_from.vertices, from_vertice_2.0 as u32 + 1) as usize]
                .p,
        ) {
            return Err(ImpossibleMergeInfo::SecondVertexClockwise);
        }
        Ok(MergeInfo {
            polygon_to: polygon_to_index,
            to_index: to_vertice_1.0,
            polygon_from: polygon_from_index,
            from_index: from_vertice_1.0 as u32,
        })
    }

    /// Assumes `can_merge` returns true
    fn merge(&mut self, merge_info: &MergeInfo) {
        let polygon_to_index = merge_info.polygon_to;
        let len_v_from = self.mesh_polygons[merge_info.polygon_from as usize]
            .vertices
            .len();
        let len_v_to = self.mesh_polygons[polygon_to_index as usize].vertices.len();

        self.mesh_polygons[polygon_to_index as usize].vertices = self.mesh_polygons
            [polygon_to_index as usize]
            .vertices
            .iter()
            .cycle()
            .skip((merge_info.to_index + 1) as usize)
            .take(len_v_to - 1)
            .chain(
                self.mesh_polygons[merge_info.polygon_from as usize]
                    .vertices
                    .iter()
                    .cycle()
                    .skip(merge_info.from_index as usize)
                    .take(len_v_from - 1),
            )
            .copied()
            .collect();
        self.mesh_polygons[polygon_to_index as usize].polygons = self.mesh_polygons
            [polygon_to_index as usize]
            .polygons
            .iter()
            .cycle()
            .skip((merge_info.to_index + 1) as usize)
            .take(len_v_to - 1)
            .chain(
                self.mesh_polygons[merge_info.polygon_from as usize]
                    .polygons
                    .iter()
                    .cycle()
                    .skip(merge_info.from_index as usize)
                    .take(len_v_from - 1),
            )
            .copied()
            .collect();

        self.mesh_polygons[polygon_to_index as usize].area +=
            self.mesh_polygons[merge_info.polygon_from as usize].area;

        self.mesh_polygons[polygon_to_index as usize].num_traversable +=
            self.mesh_polygons[merge_info.polygon_from as usize].num_traversable;
        self.mesh_polygons[polygon_to_index as usize].num_traversable -= 2;

        self.polygon_unions
            .merge(polygon_to_index, merge_info.polygon_from);
    }

    pub fn my_merge(&mut self) {
        // TODO: merge biggest areas first.
        // 1: Sort polygons by area

        let mut sorted_area_polygon_indexes: Vec<_> = self
            .mesh_polygons
            .iter()
            .enumerate()
            .map(|(index, polygon)| (index, polygon.area))
            .collect();
        let mut check_new_merge = true;
        let mut merge_count = 0;
        while check_new_merge {
            sorted_area_polygon_indexes
                .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            check_new_merge = false;
            'search_merge: for (polygon_to_index, _) in sorted_area_polygon_indexes.iter() {
                let polygon = &self.mesh_polygons[*polygon_to_index];
                for merge_index in 0..polygon.vertices.len() {
                    if let Ok(merge_info) =
                        self.can_merge(*polygon_to_index as i32, merge_index as u32)
                    {
                        dbg!("merging {merge_info}");
                        self.merge(&merge_info);
                        check_new_merge = true;
                        self.is_correct();
                        merge_count += 1;
                        break 'search_merge;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use crate::meshmerger::MergeInfo;

    use super::{ImpossibleMergeInfo, MeshMerger};

    // TODO: test read and assert result...

    // 0         1
    //  X-------X
    //  |      /|
    //  | 0   / |
    //  |    /  |
    //  |   /   |
    //  |  /    |
    //  | /   1 |
    //  |/      |
    //  X-------X
    // 3         2
    #[test]
    fn can_merge_4() {
        let mut file = std::fs::File::open("assets/meshes/quad.mesh").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let mut mesh_merger = MeshMerger::from_bytes(&buffer);
        assert_eq!(
            mesh_merger.can_merge(0, 0),
            Err(ImpossibleMergeInfo::NoNeighbour)
        );
        assert_eq!(
            mesh_merger.can_merge(0, 1),
            Ok(MergeInfo {
                polygon_to: 0,
                to_index: 1,
                polygon_from: 1,
                from_index: 0,
            })
        );
        assert_eq!(
            mesh_merger.can_merge(0, 0),
            Err(ImpossibleMergeInfo::NoNeighbour)
        );
        assert_eq!(
            mesh_merger.can_merge(1, 1),
            Err(ImpossibleMergeInfo::NoNeighbour)
        );
        assert_eq!(
            mesh_merger.can_merge(1, 2),
            Ok(MergeInfo {
                polygon_to: 1,
                to_index: 2,
                polygon_from: 0,
                from_index: 2,
            })
        );
    }
    #[test]
    fn manual_merge_4() {
        let mut file = std::fs::File::open("assets/meshes/quad.mesh").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let mut mesh_merger = MeshMerger::from_bytes(&buffer);
        mesh_merger.merge(&MergeInfo {
            polygon_to: 0,
            to_index: 1,
            polygon_from: 1,
            from_index: 0,
        });
        let mut mesh_merger = MeshMerger::from_bytes(&buffer);
        mesh_merger.merge(&MergeInfo {
            polygon_to: 1,
            to_index: 2,
            polygon_from: 0,
            from_index: 2,
        });
    }
    #[test]
    fn merge_quad() {
        let mut file = std::fs::File::open("assets/meshes/quad.mesh").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let mut mesh_merger = MeshMerger::from_bytes(&buffer);
        mesh_merger.my_merge();
        assert_eq!(
            mesh_merger.mesh_polygons[0],
            crate::meshmerger::Polygon {
                num_traversable: 0,
                area: 4.5,
                vertices: vec![0, 1, 2, 3,],
                polygons: vec![-1, -1, -1, -1,],
            }
        )
    }
    /// 0         1
    ///  X-------X
    ///  |      /|
    ///  | 0   / |
    ///  |    /  |
    ///  |   /   |
    ///  |  /    |
    ///  | /   1 |
    ///  |/      |
    ///  X-------X
    /// 3         2
    ///  X-------X
    ///  |      /
    ///  | 2   /
    ///  |    /
    ///  |   /
    ///  |  /
    ///  | /
    ///  |/
    ///  X
    /// 4
    #[test]
    fn merge_bigger_quad() {
        let mut file = std::fs::File::open("assets/meshes/quad_plus_one.mesh").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let mut mesh_merger = MeshMerger::from_bytes(&buffer);
        mesh_merger.my_merge();
        assert_eq!(
            mesh_merger.mesh_polygons[2],
            crate::meshmerger::Polygon {
                num_traversable: 0,
                area: 5.25,
                vertices: vec![1, 2, 4, 3, 0,],
                polygons: vec![-1, -1, -1, -1, -1],
            }
        )
    }
    #[test]
    fn manual_merge_5() {
        let mut file = std::fs::File::open("assets/meshes/quad_plus_one.mesh").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let mut mesh_merger = MeshMerger::from_bytes(&buffer);
        mesh_merger.merge(&MergeInfo {
            polygon_to: 0,
            to_index: 1,
            polygon_from: 1,
            from_index: 0,
        });
        for (polygon_to_index, _) in mesh_merger.mesh_polygons.iter().enumerate() {
            let polygon = &mesh_merger.mesh_polygons[polygon_to_index];
            for merge_index in 0..polygon.vertices.len() {
                if (polygon_to_index == 0 && merge_index == 3)
                    || (polygon_to_index == 2 && merge_index == 0)
                {
                    assert!(mesh_merger
                        .can_merge(polygon_to_index as i32, merge_index as u32)
                        .is_ok());
                } else {
                    assert!(mesh_merger
                        .can_merge(polygon_to_index as i32, merge_index as u32)
                        .is_err());
                }
            }
        }
        mesh_merger.merge(&MergeInfo {
            polygon_to: 2,
            to_index: 0,
            polygon_from: 0,
            from_index: 0,
        });
    }
    #[test]
    fn merge_arena() {
        let mut file = std::fs::File::open("assets/meshes/arena.mesh").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        let mut mesh_merger = MeshMerger::from_bytes(&buffer);
        assert!(
            mesh_merger.is_correct(),
            "source file is incorrect or loading code is not."
        );
        mesh_merger.my_merge();
    }
}
