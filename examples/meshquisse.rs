use bevy::prelude::*;
use meshquisse::navmesh::*;
use meshquisse::*;
use polyanya::Mesh as PAMesh;

fn main() {
    App::new()
        .add_plugin(MeshquissePlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let navmesh = PAMesh::from_file("assets/meshes/polyanya/arena-merged.mesh".into());
    commands.spawn().insert(NavMesh { navmesh });
}
