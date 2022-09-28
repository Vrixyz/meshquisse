use std::io::{self, BufRead};

use bevy::prelude::Vec2;

/// Credits to https://bitbucket.org/dharabor/pathfinding/src/master/anyangle/polyanya/utils/meshmerger.cpp

#[derive(Default, Debug, Clone)]
pub struct UnionFind {
    parent: Vec<i32>,
}

impl UnionFind {
    fn new(polygon_count: i32) -> Self {
        Self {
            parent: (0..polygon_count).collect(),
        }
    }

    fn find(&mut self, x: i32) -> i32 {
        if x == -1 {
            return -1;
        }
        if self.parent[x as usize] != x {
            self.parent[x as usize] = self.find(self.parent[x as usize]);
        }
        return self.parent[x as usize];
    }
    fn merge(&mut self, x: i32, y: i32) {
        let x = self.find(x);
        let y = self.find(y);
        self.parent[y as usize] = x;
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub p: Vec2,
    pub num_polygons: u32,
    pub polygons: Vec<i32>,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Polygon {
    num_traversable: u32,
    area: f32,
    vertices: Vec<u32>,
    /// Stores the original polygons.
    /// To get the actual polygon, do polygon_unions.find on the polygon you get.
    polygons: Vec<i32>,
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

fn index<T>(vec: &Vec<T>, index: u32) -> usize {
    index as usize % vec.len()
}
fn getc<'a, T: Copy>(vec: &'a Vec<T>, index: u32) -> T {
    vec[index as usize % vec.len()]
}

fn getc_mut<'a, T>(vec: &'a mut Vec<T>, index: u32) -> &'a mut T {
    let len = vec.len();
    &mut vec[index as usize % len]
}

pub struct MergeInfo {
    /// index of the polygon_from, from the root polygons
    pub polygon_from: i32,
    /// index of the start vertex within the polygon we want to merge into
    pub to_index: u32,
    /// index of the same vertex but from the polygon we merge from.
    pub from_index: u32,
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

    /// Can polygon x merge with the polygon adjacent to the given edge index ?
    ///
    /// Try to merge `self.mesh_polygons[polygon_to_index]` with `self.mesh_polygons].polygon[edge_index]`
    /// through edge `self.mesh_polygons[polygon_to_index].vertices[edge_index..edge_index + 1]`.
    ///
    /// Return None if `self.mesh_polygons[polygon_to_index]` is a merged polygon.
    /// Return None if resulting polygon would be concave.
    /// Return None if there's no neighbour polygon on that index.
    fn can_merge(&mut self, polygon_to_index: i32, edge_index: u32) -> Option<MergeInfo> {
        let p = |index: u32| &self.mesh_polygons[index as usize];
        if self.polygon_unions.find(polygon_to_index) != polygon_to_index {
            return None;
        }
        // The polygon we want to modify to be the merge result
        let polygon_to = &self.mesh_polygons[polygon_to_index as usize];

        let merge_index = self
            .polygon_unions
            .find(getc(&polygon_to.polygons, edge_index) as i32);
        if merge_index == -1 {
            return None;
        }
        // The polygon we want to merge from
        let polygon_from = p(merge_index as u32);
        if polygon_from.vertices.len() == 0 {
            return None;
        }
        // edge 1 in the "from"
        let to_vertice_1 = (edge_index, polygon_to.vertices[edge_index as usize]);
        let to_vertice_2 = (edge_index + 1, getc(&polygon_to.vertices, edge_index + 1));

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
            from_vertice_1.0 + &polygon_from.vertices.len() - 1,
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
            "wrong second vertex correspondance"
        );
        debug_assert!(
            getc(&polygon_from.polygons, from_vertice_2.0 as u32) == polygon_to_index,
            "Neighbour from polygon does not match edge to."
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
            return None;
        }
        // check clockwiseness for (to_vertice_2 - 1, to_vertice_2, from_vertice_1 + 1)
        // If the new ones are clockwise, we must return false.
        if Self::cw(
            &self.mesh_vertices[getc(
                &polygon_to.vertices,
                from_vertice_2.0 as u32 + polygon_to.vertices.len() as u32 - 1,
            ) as usize]
                .p,
            &self.mesh_vertices[from_vertice_2.1 as usize].p,
            &self.mesh_vertices[getc(&polygon_from.vertices, to_vertice_2.0 + 1) as usize].p,
        ) {
            return None;
        }
        Some(MergeInfo {
            to_index: to_vertice_1.0,
            from_index: to_vertice_1.0,
            polygon_from: merge_index,
        })
    }

    /// Assumes `can_merge` returns true
    fn merge(&mut self, polygon_index: i32, merge_info: MergeInfo) {
        let len_v_from = self.mesh_polygons[merge_info.polygon_from as usize]
            .vertices
            .len();
        let len_v_to = self.mesh_polygons[polygon_index as usize].vertices.len();

        self.mesh_polygons[polygon_index as usize].vertices = self.mesh_polygons
            [polygon_index as usize]
            .vertices
            .iter()
            .cycle()
            .skip((merge_info.to_index + 2) as usize)
            .take(len_v_to - 2)
            .chain(
                self.mesh_polygons[merge_info.polygon_from as usize]
                    .vertices
                    .iter()
                    .cycle()
                    .skip((merge_info.from_index + len_v_from as u32 - 1) as usize)
                    .take(len_v_from),
            )
            .copied()
            .collect();
        self.mesh_polygons[polygon_index as usize].polygons = self.mesh_polygons
            [polygon_index as usize]
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
                    .skip((merge_info.from_index + len_v_from as u32 - 1) as usize)
                    .take(len_v_from - 1),
            )
            .copied()
            .collect();

        self.mesh_polygons[polygon_index as usize].area +=
            self.mesh_polygons[merge_info.polygon_from as usize].area;

        self.mesh_polygons[polygon_index as usize].num_traversable +=
            self.mesh_polygons[merge_info.polygon_from as usize].num_traversable;
        self.mesh_polygons[polygon_index as usize].num_traversable -= 2;

        self.polygon_unions
            .merge(polygon_index, merge_info.polygon_from);
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
        sorted_area_polygon_indexes
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut check_new_merge = true;
        while check_new_merge {
            check_new_merge = false;
            for (i, _) in sorted_area_polygon_indexes.iter() {
                let polygon = &self.mesh_polygons[*i];
                for merge_index in 0..polygon.vertices.len() {
                    if let Some(merge_info) = self.can_merge(*i as i32, merge_index as u32) {
                        self.merge(*i as i32, merge_info);
                        check_new_merge = true;
                    }
                }
            }
        }
        dbg!(self);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::MeshMerger;

    #[test]
    fn read_file() {
        let mut file = std::fs::File::open("assets/meshes/quad.mesh").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let mut mesh_merger = MeshMerger::from_bytes(&buffer);
        dbg!(&mesh_merger);
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
}