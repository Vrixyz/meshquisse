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

// TODO: we should probably use newtype pattern to differentiate index for polygons and index for vertices
type ListNodePtr = u32;

#[derive(Default, Debug, Clone)]
pub struct ListData {
    pub nodes: Vec<ListNode>,
}

#[derive(Default, Debug, Clone)]
pub struct ListNode {
    /// id in `ListData`
    pub next: ListNodePtr,
    pub val: i32,
}

trait GetVal {
    fn get<'b, 'a: 'b>(self, data: &'a ListData) -> &'b ListNode;
    fn get_mut<'b, 'a: 'b>(self, data: &'a mut ListData) -> &'b mut ListNode;
}

impl GetVal for ListNodePtr {
    fn get<'b, 'a: 'b>(self, data: &'a ListData) -> &'b ListNode {
        &data.nodes[self as usize]
    }
    fn get_mut<'b, 'a: 'b>(self, data: &'a mut ListData) -> &'b mut ListNode {
        &mut data.nodes[self as usize]
    }
}

impl ListData {
    pub fn go(&self, from: ListNodePtr, n: u32) -> ListNodePtr {
        let mut out = from;
        for i in 0..n {
            out = out.get(self).next;
        }
        out
    }
    pub fn go_get(&self, from: ListNodePtr, n: u32) -> &ListNode {
        let mut out = from;
        for i in 0..n {
            out = out.get(self).next;
        }
        out.get(self)
    }
    pub fn go_get_mut(&mut self, from: ListNodePtr, n: u32) -> &mut ListNode {
        let mut out = from;
        for i in 0..n {
            out = out.get(self).next;
        }
        let real_out = out.get_mut(self);
        real_out
    }
    /// FIXME: Not sure why this needs next (it should always be 0 to maintain the loop)
    pub fn push_new_node(&mut self, next: ListNodePtr, val: i32) -> ListNodePtr {
        let new_id = self.nodes.len() as ListNodePtr;
        self.nodes.push(ListNode { next, val });
        new_id
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub p: Vec2,
    pub num_polygons: u32,
    pub polygons: ListData,
}

#[derive(Default, Debug, Clone)]
pub struct Polygon {
    num_vertices: u32,
    num_traversable: u32,
    area: f32,
    vertices: ListData,
    /// Stores the original polygons.
    /// To get the actual polygon, do polygon_unions.find on the polygon you get.
    polygons: ListData,
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
    pub fn get_area(mesh_vertices: &[Vertex], area: &ListData) -> f32 {
        let mut vertex_id = 0;
        // first point x second point + second point x third point + ...
        let mut out = 0f32;

        let start_vertex = vertex_id;
        let mut is_first = true;

        while is_first || start_vertex != vertex_id {
            is_first = false;
            let current = vertex_id.get(area);
            let next = current.next.get(area);
            out += determinant(
                &mesh_vertices[current.val as usize].p,
                &mesh_vertices[next.val as usize].p,
            );
            vertex_id = current.next;
        }
        out
    }
    pub fn from_bytes(bytes: &[u8]) -> MeshMerger {
        let mut list_data = ListData::default();

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

        for i in 0..v_nb_vertices {
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
            let mut vertex = &mut mesh_vertices[i];
            vertex.p = Vec2::new(x, y);
            vertex.num_polygons = neigbours;
            let neighbour_values: Vec<i32> = values.map(|v| v.parse().unwrap()).collect();
            if neighbour_values.len() != neigbours as usize {
                panic!("read more neighbours than defined.");
            }
            // Guaranteed to have 2 or more.
            // TODO: those lines should probably be abstracted away by creating a circular linked list with a given array of values.
            let mut cur_node = 0;
            for (j, polygon_index) in neighbour_values.into_iter().enumerate() {
                if polygon_index >= p_nb_polygons as i32 {
                    panic!(
                        "Got a polygon index of {polygon_index} when nb_polygon is {p_nb_polygons}"
                    );
                }
                let new_node = vertex.polygons.push_new_node(0, polygon_index as i32);
                if j == 0 {
                    cur_node = new_node;
                } else {
                    cur_node.get_mut(&mut vertex.polygons).next = new_node;
                    cur_node = new_node;
                }
            }
        }
        for i in 0..p_nb_polygons {
            let mut polygon = &mut mesh_polygons[i];
            let line: String = reader.next().unwrap().unwrap();
            let mut values = line.split(' ');

            // Step: Read polygon's vertices (corresponding to neighbouring polygons too)
            let n = values.next().unwrap().parse().unwrap();
            if n < 3 {
                panic!("Invalid number of vertices in polygon (Got {n}).");
            }
            polygon.num_vertices = n;
            let mut cur_node = 0;
            for j in 0..n {
                let vertex_index: i32 = values.next().unwrap().parse().unwrap();
                if vertex_index >= v_nb_vertices as i32 {
                    panic!("Invalid vertex index when getting polygon");
                }
                let new_node = polygon.vertices.push_new_node(0, vertex_index as i32);
                if j == 0 {
                    cur_node = new_node;
                } else {
                    cur_node.get_mut(&mut polygon.vertices).next = new_node;
                    cur_node = new_node;
                }
            }

            // Step: Read polygon's neighbour polygons
            polygon.num_traversable = 0;
            for j in 0..n {
                let polygon_index: i32 = values.next().unwrap().parse().unwrap();
                if polygon_index >= p_nb_polygons as i32 {
                    panic!("Invalid polygon index when getting polygon");
                }
                if polygon_index != -1 {
                    polygon.num_traversable += 1;
                }
                let new_node = polygon.polygons.push_new_node(0, polygon_index as i32);
                if j == 0 {
                    cur_node = new_node;
                } else {
                    cur_node.get_mut(&mut polygon.polygons).next = new_node;
                    cur_node = new_node;
                }
            }
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

    // Can polygon x merge with the polygon adjacent to the edge
    // (v->next, v->next->next)?
    // (The reason for this is because we don't have back pointers, and we need
    // to have the vertex before the edge starts).
    // Assume that v and p are "aligned", that is, they have been offset by the
    // same amount.
    // This also means that the actual polygon used will be p->next->next.
    // Also assume that x is a valid non-merged polygon.
    // NOTE: v and p are vertices and polygons data from x.
    fn can_merge(&mut self, x_polygon_index: i32, v: ListNodePtr, p: ListNodePtr) -> bool {
        if self.polygon_unions.find(x_polygon_index) != x_polygon_index {
            return false;
        }
        let polygon = &self.mesh_polygons[x_polygon_index as usize];
        // D for data
        let pD = &polygon.polygons;
        // D for data
        let vD = &polygon.vertices;
        let merge_index = self.polygon_unions.find(pD.go_get(p, 2).val);

        if merge_index == -1 {
            return false;
        }
        let to_merge = &self.mesh_polygons[merge_index as usize];
        if to_merge.num_vertices == 0 {
            return false;
        }
        // Define (v->next, v->next->next).
        let A = vD.go_get(p, 1).val;
        let B = vD.go_get(p, 2).val;

        // We want to find (B, A) inside to_merge's vertices.
        // In fact, we want to find the one BEFORE B. We'll call this merge_end.
        // Assert that we have good data - that is, if B appears, A must be next.
        // Also, we can't iterate for more than to_merge.num_vertices.
        let mut merge_end_v: ListNodePtr = 0;
        let mut merge_end_p: ListNodePtr = 0;
        let mut counter = 0;
        while merge_end_v.get(vD).next.get(vD).val != B {
            merge_end_v = merge_end_v.get(vD).next;
            merge_end_p = merge_end_p.get(pD).next;
            counter += 1;
            assert!(counter <= to_merge.num_vertices);
        }
        // Ensure that A comes after B.
        assert!(vD.go(merge_end_v, 2).get(vD).val == A);

        // Ensure that the neighbouring polygon is x.
        assert!(self.polygon_unions.find(pD.go_get(merge_end_p, 2).val) == x_polygon_index);
        // The merge will change
        // (v, A, B) to (v, A, [3 after merge_end_v]) and
        // (A, B, [3 after v]) to (merge_end_v, B, [3 after v]).
        // If the new ones are clockwise, we must return false.
        if Self::cw(
            &self.mesh_vertices[v.get(vD).val as usize].p,
            &self.mesh_vertices[vD.go_get(v, 1).val as usize].p,
            &self.mesh_vertices[vD.go_get(merge_end_v, 3).val as usize].p,
        ) {
            return false;
        }
        true
    }
    fn merge(&mut self, x: i32, v: ListNodePtr, p: ListNodePtr) {
        assert!(self.can_merge(x, v, p));
        // TODO: continue
        // https://bitbucket.org/dharabor/pathfinding/src/ce5b02e9d051d5f17addb359429104c0293decaf/anyangle/polyanya/utils/meshmerger.cpp#lines-447

        let polygon = &self.mesh_polygons[x as usize];
        // D for data
        let pD = &polygon.polygons;
        // D for data
        let vD = &polygon.vertices;
        // Note that because of the way we're merging,
        // the resulting polygon will NOT always have a valid ListNodePtr, so
        // we need to set it ourself.

        let merge_index = self.polygon_unions.find(pD.go_get(p, 2).val);

        // FIXME: ? original implementation does a find again ? Seems unnecessary ?
        let to_merge = &self.mesh_polygons[merge_index as usize];

        let A = vD.go_get(p, 1).val;
        let B = vD.go_get(p, 2).val;

        let mut merge_end_v: ListNodePtr = 0;
        let mut merge_end_p: ListNodePtr = 0;
        let mut counter = 0;
        while merge_end_v.get(vD).next.get(vD).val != B {
            merge_end_v = merge_end_v.get(vD).next;
            merge_end_p = merge_end_p.get(pD).next;
            counter += 1;
            assert!(counter <= to_merge.num_vertices);
        }
        // https://bitbucket.org/dharabor/pathfinding/src/ce5b02e9d051d5f17addb359429104c0293decaf/anyangle/polyanya/utils/meshmerger.cpp#lines-467
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
        let mesh_merger = MeshMerger::from_bytes(&buffer);
        dbg!(mesh_merger);
    }
}
