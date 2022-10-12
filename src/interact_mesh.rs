use std::marker::PhantomData;

use bevy::{math::Vec3Swizzles, pbr::wireframe::Wireframe, prelude::*};
use bevy_rapier3d::prelude::RapierContext;
use bevy_transform_gizmo::TransformGizmoSystem;

use crate::{
    navmesh::NavMesh,
    screen_physics_ray_cast,
    tools::{self, bevymesh_from_trimesh, navmesh_from_trimesh, TriangleMesh},
    MainCamera,
};
use polyanya::Mesh as PAMesh;

#[derive(Default)]
pub struct InteractMeshPlugin<
    MeshData: 'static + Component + Sync + Send + IntoPAMesh + UpdateVertex + IntoBevyMesh,
> {
    _p: PhantomData<MeshData>,
}

impl<MeshData: 'static + Component + Sync + Send + IntoPAMesh + UpdateVertex + IntoBevyMesh> Plugin
    for InteractMeshPlugin<MeshData>
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(bevy_mod_picking::DefaultPickingPlugins)
            .add_plugin(bevy_transform_gizmo::TransformGizmoPlugin::default())
            .add_startup_system(init_assets)
            .add_stage_before(
                CoreStage::PreUpdate,
                "before_preupdate",
                SystemStage::parallel(),
            )
            .add_system_to_stage("before_preupdate", adapt_camera)
            .add_system(spawn_vertices_selectable::<MeshData>)
            .add_system(update_vertices_position::<MeshData>)
            .add_system(spawn_visual_mesh::<MeshData>)
            .add_system(update_visual_mesh::<MeshData>)
            .add_system(spawn_navmesh::<MeshData>)
            .add_system(update_navmesh::<MeshData>);
    }
}

pub trait IntoPAMesh {
    fn to_pa_mesh(&self) -> PAMesh;
}
pub trait UpdateVertex {
    fn update_vertex(&mut self, vertex_index: u32, position: Vec3);
    // FIXME: perf is horrible but I didn't succeed in getting a generic iterator over Vec2 or Vertex.
    fn iter_positions(&self) -> Vec<Vec2>;
}
pub trait IntoBevyMesh {
    fn to_bevy_mesh(&self) -> Mesh;
    fn update_mesh(&self, mesh: &mut Mesh);
}

/// Only useful if entity has a `TriangleMeshData`.
/// Will insert a `navmesh::NavMesh` as component,
/// and update its visual when its `TriangleMeshData` changes.
#[derive(Component)]
pub struct UpdateNavMesh;

/// Only useful if entity has a `TriangleMeshData`.
/// Will insert a bevy mesh,
/// and update its visual when its `TriangleMeshData` changes.
#[derive(Component, Default)]
pub struct ShowAndUpdateMesh(pub Option<Handle<Mesh>>);

/// Only useful if entity has a `TriangleMeshData`.
/// Will spawn children selectable handles via bevy_transform_gizmo.
/// When these gizmos are updated, they reach for their parent `EditableMesh`
/// and update its mesh.
#[derive(Component)]
pub struct EditableMesh;

#[derive(Component)]
pub struct EditableMeshVertex {
    pub vertex_id: u32,
}

pub struct InteractAssets {
    gizmo_mesh: Handle<Mesh>,
    gizmo_mesh_mat: Handle<StandardMaterial>,
    visual_mesh_mat: Handle<StandardMaterial>,
}

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(InteractAssets {
        gizmo_mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        gizmo_mesh_mat: materials.add(Color::rgb(0.99, 0.2, 0.3).into()),
        visual_mesh_mat: materials.add(Color::rgb(1f32, 1f32, 1f32).into()),
    });
}
fn adapt_camera(mut commands: Commands, q_cam: Query<Entity, Added<MainCamera>>) {
    for e in q_cam.iter() {
        commands
            .entity(e)
            .insert_bundle(bevy_mod_picking::PickingCameraBundle::default())
            .insert(bevy_transform_gizmo::GizmoPickSource::default());
    }
}

fn spawn_visual_mesh<MeshData: IntoBevyMesh + Component>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<InteractAssets>,
    mut q_new_shown_meshes: Query<
        (Entity, &MeshData, &mut ShowAndUpdateMesh),
        Added<ShowAndUpdateMesh>,
    >,
) {
    for (e, mesh_data, mut show_update_mesh) in q_new_shown_meshes.iter_mut() {
        let mesh_handle = meshes.add(mesh_data.to_bevy_mesh());
        (*show_update_mesh).0 = Some(mesh_handle.clone());
        commands
            .entity(e)
            .insert_bundle(PbrBundle {
                mesh: mesh_handle,
                material: assets.visual_mesh_mat.clone(),
                ..default()
            })
            .insert(Wireframe);
    }
}
/*
/// Optimised update to modify only vertex positions
fn update_visual_mesh<MeshData: IntoBevyMesh + Component>(
    mut meshes: ResMut<Assets<Mesh>>,
    q_updated_meshes: Query<(&ShowAndUpdateMesh, &MeshData), Changed<MeshData>>,
) {
    for (update, mesh_data) in q_updated_meshes.iter() {
        dbg!("changed");
        if let Some(mesh_handle) = update.0.as_ref() {
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                mesh_data.update_mesh(mesh)
            }
        }
    }
}*/

/// full update
fn update_visual_mesh<MeshData: IntoBevyMesh + Component>(
    mut meshes: ResMut<Assets<Mesh>>,
    q_updated_meshes: Query<(&ShowAndUpdateMesh, &MeshData), Changed<MeshData>>,
) {
    for (update, mesh_data) in q_updated_meshes.iter() {
        if let Some(mesh_handle) = update.0.as_ref() {
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                *mesh = mesh_data.to_bevy_mesh();
            }
        }
    }
}

fn spawn_vertices_selectable<MeshData: UpdateVertex + Component>(
    mut commands: Commands,
    assets: Res<InteractAssets>,
    q_new_editable_meshes: Query<(Entity, &MeshData), Added<EditableMesh>>,
) {
    for (e, mesh_data) in q_new_editable_meshes.iter() {
        commands.entity(e).add_children(|parent| {
            for (vertex_id, position) in mesh_data.iter_positions().iter().enumerate() {
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: assets.gizmo_mesh.clone(),
                        material: assets.gizmo_mesh_mat.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            position.x, 0f32, position.y,
                        )),
                        ..Default::default()
                    })
                    .insert(EditableMeshVertex {
                        vertex_id: vertex_id as u32,
                    })
                    .insert_bundle(bevy_mod_picking::PickableBundle::default())
                    .insert(bevy_transform_gizmo::GizmoTransformable);
            }
        });
    }
}

fn update_vertices_position<MeshData: UpdateVertex + Component>(
    q_changed_vertices: Query<(&Parent, &EditableMeshVertex, &Transform), Changed<Transform>>,
    mut q_parent_mesh_data: Query<&mut MeshData>,
) {
    for (parent, vertex, transform) in q_changed_vertices.iter() {
        if let Ok(mut mesh_data_to_edit) = q_parent_mesh_data.get_mut(parent.get()) {
            mesh_data_to_edit.update_vertex(vertex.vertex_id, transform.translation);
        }
    }
}

fn spawn_navmesh<MeshData: IntoPAMesh + Component>(
    mut commands: Commands,
    mut q_new_shown_meshes: Query<(Entity, &MeshData), Added<UpdateNavMesh>>,
) {
    for (e, mesh_data) in q_new_shown_meshes.iter_mut() {
        let navmesh = mesh_data.to_pa_mesh();
        commands.entity(e).insert(NavMesh { navmesh });
    }
}

fn update_navmesh<MeshData: IntoPAMesh + Component>(
    mut q_updated_meshes: Query<
        (&mut NavMesh, &MeshData),
        (Changed<MeshData>, With<UpdateNavMesh>),
    >,
) {
    for (mut update, mesh_data) in q_updated_meshes.iter_mut() {
        update.navmesh = mesh_data.to_pa_mesh();
    }
}
