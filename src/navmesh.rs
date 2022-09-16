use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::settings::{WgpuFeatures, WgpuSettings};
use bevy_polyline::PolylinePlugin;
use polyanya::Mesh as PAMesh;

pub struct NavMeshPlugin;

impl Plugin for NavMeshPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .add_plugin(WireframePlugin)
        .add_startup_system(setup_navmesh)
        .add_startup_system(setup_navmesh_graphics)
        .add_system(update_mesh_visualization);
    }
}

#[derive(Component)]
pub struct NavMesh {
    pub navmesh: PAMesh,
}

fn setup_navmesh(mut commands: Commands) {
    let navmesh = PAMesh::from_file("assets/meshes/polyanya/arena-merged.mesh".into());
    commands.spawn().insert(NavMesh { navmesh });
}

struct NavMeshMaterials {
    mesh: Handle<StandardMaterial>,
}

fn setup_navmesh_graphics(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(NavMeshMaterials {
        mesh: materials.add(Color::rgb(0.8, 0.4, 0.3).into()),
    });
}

fn update_mesh_visualization(
    mut commands: Commands,
    materials: Res<NavMeshMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    q: Query<(Entity, &NavMesh), Changed<NavMesh>>,
) {
    for (e, navmesh) in q.iter() {
        // TODO: only modify mesh resource and not recreate a new one
        // TODO: so modifying the entity is not even mandatory.
        let mut new_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        new_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            navmesh
                .navmesh
                .vertices
                .iter()
                .map(|v| [v.coords.x, 0.0, v.coords.y])
                .collect::<Vec<[f32; 3]>>(),
        );
        // FIXME: polyanya has polygons, but bevy mesh is waiting for triangles, I should probably triangulate each one ?

        new_mesh.set_indices(Some(Indices::U32(
            navmesh
                .navmesh
                .polygons
                .iter()
                .flat_map(|p| {
                    // Convex triangulation.
                    // thanks to https://www.gamedev.net/articles/programming/graphics/polygon-triangulation-r3334/
                    // for pseudocode
                    let mut triangles = vec![];
                    let mut index = 0;
                    let p0 = p.vertices[index];
                    index += 1;
                    let mut p_helper = p.vertices[index];
                    index += 1;

                    while index < p.vertices.len() {
                        let p_temp = p.vertices[index];
                        index += 1;
                        triangles.push([p0, p_temp, p_helper]);
                        p_helper = p_temp;
                    }

                    triangles.into_iter().flatten().map(|v| v as u32)
                })
                .collect(),
        )));
        new_mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            (0..navmesh.navmesh.vertices.len())
                .into_iter()
                .map(|_| [0.0, 1.0, 0.0])
                .collect::<Vec<[f32; 3]>>(),
        );

        new_mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            navmesh
                .navmesh
                .vertices
                .iter()
                .map(|v| [v.coords.x, v.coords.y])
                .collect::<Vec<[f32; 2]>>(),
        );
        let nav_mesh_handle = meshes.add(new_mesh);
        commands.entity(e).insert_bundle(PbrBundle {
            mesh: nav_mesh_handle,
            material: materials.mesh.clone(),
            ..default()
        });
    }
}
