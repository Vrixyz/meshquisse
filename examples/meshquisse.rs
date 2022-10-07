use bevy::prelude::*;
use meshquisse::{
    interact_mesh::{EditableMesh, InteractMeshPlugin, ShowAndUpdateMesh, UpdateNavMesh},
    polygon_mesh_data::TriangleMeshData,
    *,
};

fn main() {
    App::new().add_plugin(ToolPlugin).run();
}

struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MeshquissePlugin)
            .add_plugin(InteractMeshPlugin::<TriangleMeshData>::default())
            .add_startup_system(setup);
    }
}

fn setup(mut commands: Commands) {
    // NavMesh
    let trimesh = tools::create_grid_trimesh(3, 3, 10f32);

    commands
        .spawn()
        //.insert(NavMesh { navmesh })
        .insert(TriangleMeshData(trimesh))
        .insert(ShowAndUpdateMesh::default())
        .insert(UpdateNavMesh)
        .insert(EditableMesh);
}
