use std::io::Read;

use bevy::prelude::*;
use meshquisse::{
    interact_mesh::{EditableMesh, InteractMeshPlugin, ShowAndUpdateMesh, UpdateNavMesh},
    meshmerger::MeshMerger,
    polygon_mesh_data::{ConvexPolygonsMeshData, TriangleMeshData},
    *,
};

fn main() {
    App::new().add_plugin(ToolPlugin).run();
}

struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MeshquissePlugin)
            .add_plugin(InteractMeshPlugin::<ConvexPolygonsMeshData>::default())
            .add_startup_system(setup);
    }
}

fn setup(mut commands: Commands) {
    let mut file = std::fs::File::open("assets/meshes/aurora_merged.mesh").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut mesh_merger = MeshMerger::from_bytes(&buffer);
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
