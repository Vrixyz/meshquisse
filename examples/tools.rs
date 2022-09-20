use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use meshquisse::{navmesh::NavMesh, *};
use polyanya::Mesh as PAMesh;

fn main() {
    App::new().add_plugin(ToolPlugin).run();
}

struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MeshquissePlugin).add_startup_system(setup);
    }
}
struct MyMaterials {
    bevy_mesh: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Materials
    let mat = materials.add(Color::rgb(0.2, 1.0, 0.2).into());
    commands.insert_resource(MyMaterials {
        bevy_mesh: mat.clone(),
    });

    // NavMesh
    let trimesh = tools::create_grid_trimesh(30, 30, 1f32);
    //dbg!(&trimesh);

    // Spawn the navmesh for meshquisse plugin

    let mut navmesh = tools::mesh_from_trimesh(trimesh);
    navmesh.bake();
    //dbg!(&navmesh);

    commands.spawn().insert(NavMesh { navmesh });
}
