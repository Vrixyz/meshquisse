use std::io::Read;

use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use meshquisse::{
    interact_mesh::{EditableMesh, InteractMeshPlugin, ShowAndUpdateMesh, UpdateNavMesh},
    meshmerger::{MeshMerger, UnionFind},
    polygon_mesh_data::{ConvexPolygonsMeshData, TriangleMeshData},
    tools::create_grid_trimesh,
    *,
};

fn main() {
    App::new().add_plugin(ToolPlugin).run();
}

struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MeshquissePlugin)
            //.add_plugin(WireframePlugin)
            .add_plugin(InteractMeshPlugin::<ConvexPolygonsMeshData>::default())
            .add_startup_system(setup)
            .add_system(save_mesh);
    }
}

fn setup(mut commands: Commands) {
    /*
        let mut file = std::fs::File::open("assets/meshes/arena_merged.mesh").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let mut mesh_merger = MeshMerger::from_bytes(&buffer);

        let convex_data = ConvexPolygonsMeshData::from(&mesh_merger);
    */
    let triangles_data = create_grid_trimesh(2, 2, 10f32);

    let convex_data = ConvexPolygonsMeshData::from(&TriangleMeshData(triangles_data));

    let nb_polygons = convex_data.mesh_polygons.len();
    let mut mesh_merger = MeshMerger {
        mesh_vertices: convex_data.mesh_vertices,
        mesh_polygons: convex_data.mesh_polygons,
        polygon_unions: UnionFind::new(nb_polygons as i32),
    };
    mesh_merger.my_merge();

    let convex_data = ConvexPolygonsMeshData::from(&mesh_merger);

    commands
        .spawn()
        //.insert(NavMesh { navmesh })
        .insert(convex_data)
        .insert(ShowAndUpdateMesh::default())
        .insert(UpdateNavMesh)
        .insert(EditableMesh);
}

fn save_mesh(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::S) {
        // TODO: save mesh into mesh 2 format, to load into it and test different merge stuff.
    }
}
