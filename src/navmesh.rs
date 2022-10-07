use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::settings::{WgpuFeatures, WgpuSettings};
use polyanya::Mesh as PAMesh;

use crate::tools;

pub struct NavMeshPlugin;

impl Plugin for NavMeshPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .add_startup_system(setup_navmesh_graphics); //.add_system(update_mesh_visualization);
    }
}

#[derive(Component)]
pub struct NavMesh {
    pub navmesh: PAMesh,
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
        new_mesh.set_indices(Some(Indices::U32(tools::triangulate(&navmesh.navmesh))));
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
