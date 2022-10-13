use std::{io::Read, time::SystemTime};

use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use meshquisse::{
    interact_mesh::{EditableMesh, InteractMeshPlugin, ShowAndUpdateMesh, UpdateNavMesh},
    mesh_data::{merge_triangles::ConvexPolygonsMeshData, only_triangles::TriangleMeshData},
    tools::create_grid_trimesh,
    trianglemerger::{MeshMerger, UnionFind},
    *,
};

fn main() {
    App::new().add_plugin(ToolPlugin).run();
}

struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MeshquissePlugin)
            //.add_plugin(NoCameraPlayerPlugin)
            //.add_plugin(WireframePlugin)
            .add_plugin(InteractMeshPlugin::<ConvexPolygonsMeshData>::default())
            .add_startup_system(setup)
            .add_system(update_camera)
            .add_system(save_mesh)
            .add_system(try_merge_1);
    }
}
fn update_camera(mut commands: Commands, cam: Query<Entity, Added<MainCamera>>) {
    for e in cam.iter() {
        //commands.entity(e).insert(FlyCam);
    }
}

fn setup(mut commands: Commands) {
    ///*
    let mut file = std::fs::File::open("assets/meshes/arena.mesh").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut mesh_merger = MeshMerger::from_bytes(&buffer);

    let convex_data = ConvexPolygonsMeshData::from(&mesh_merger);
    // */
    /*
    let triangles_data = create_grid_trimesh(3, 3, 10f32);

    let convex_data = ConvexPolygonsMeshData::from(&TriangleMeshData(triangles_data));
    // */
    let nb_polygons = convex_data.mesh_polygons.len();
    let mut mesh_merger = MeshMerger {
        mesh_vertices: convex_data.mesh_vertices,
        mesh_polygons: convex_data.mesh_polygons,
        polygon_unions: UnionFind::new(nb_polygons as i32),
    };
    let start = SystemTime::now();
    //dbg!(&mesh_merger);
    //mesh_merger.my_merge();
    let end = SystemTime::now();
    let elapsed = end.duration_since(start);

    println!(
        "Merging took around {}s",
        elapsed.unwrap_or_default().as_secs_f32()
    );
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
fn try_merge_1(
    keyboard_input: Res<Input<KeyCode>>,
    mut mesh_convex_data: Query<&mut ConvexPolygonsMeshData>,
) {
    if keyboard_input.just_pressed(KeyCode::M) {
        dbg!("just pressed M");
        for mut data in mesh_convex_data.iter_mut() {
            let nb_polygons = data.mesh_polygons.len();
            let union_find = UnionFind {
                parent: (0i32..(nb_polygons as i32))
                    .map(|polygon_index| polygon_index)
                    .collect(),
            };
            let mut mesh_merger = MeshMerger {
                mesh_vertices: data.mesh_vertices.clone(),
                mesh_polygons: data.mesh_polygons.clone(),
                polygon_unions: union_find,
            };
            mesh_merger.my_merge();
            mesh_merger.remove_unused();
            dbg!(&mesh_merger);

            *data = ConvexPolygonsMeshData::from(&mesh_merger);
        }
    }
}
