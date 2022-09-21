pub mod interact_mesh;
pub mod navmesh;
pub mod tools;

use bevy::{input::keyboard::KeyboardInput, math::Vec3Swizzles, prelude::*};

use bevy_polyline::prelude::*;
use bevy_rapier3d::prelude::*;
use interact_mesh::InteractMeshPlugin;
use navmesh::NavMeshPlugin;

pub struct MeshquissePlugin;

impl Plugin for MeshquissePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb(
            0xF9 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            0xFF as f32 / 255.0,
        )))
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(PolylinePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InteractMeshPlugin)
        .add_plugin(NavMeshPlugin)
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(cast_ray_pathfinding)
        .add_startup_system(setup_path_display)
        .add_system(update_path_display);
    }
}

#[derive(Component)]
pub struct MainCamera;

fn setup_graphics(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-30.0, 30.0, 100.0)
                .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
            ..Default::default()
        })
        .insert(MainCamera);
    const HALF_SIZE: f32 = 100.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            3f32 * std::f32::consts::PI / 5.0,
            -std::f32::consts::PI / 4.,
        )),
        ..default()
    });
}

pub fn setup_physics(mut commands: Commands) {
    /*
     * Ground
     */
    let ground_size = 200.1;
    let ground_height = 0.1;

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(
            0.0,
            -ground_height,
            0.0,
        )))
        .insert(Collider::cuboid(ground_size, ground_height, ground_size));
}

fn cast_ray_pathfinding(
    mut commands: Commands,
    mut path_to_display: ResMut<PathToDisplay>,
    windows: Res<Windows>,
    navmesh: Query<&navmesh::NavMesh>,
    buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if keys.any_pressed([KeyCode::LControl, KeyCode::RControl]) {
        return;
    }
    if buttons.just_pressed(MouseButton::Right) {
        path_to_display.steps.clear();
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let navmesh = navmesh.iter().next();
    if navmesh.is_none() {
        return;
    }
    let navmesh = navmesh.unwrap();
    if let Some(position) = screen_physics_ray_cast(cameras, windows, rapier_context) {
        if let Some(last_pos) = path_to_display.steps.last() {
            let new_point = position.xz();
            if !navmesh.navmesh.point_in_mesh(new_point) {
                info!("point not in mesh");
                return;
            }
            let path = navmesh.navmesh.path(*last_pos, position.xz());
            for p in path.path {
                path_to_display.steps.push(p);
            }
        } else {
            path_to_display.steps.push(position.xz());
        }
    }
}

fn screen_physics_ray_cast(
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
) -> Option<Vec3> {
    if let Some((camera, camera_transform)) = cameras.iter().next() {
        // First, compute a ray from the mouse position.
        let (ray_pos, ray_dir) =
            ray_from_mouse_position(windows.get_primary().unwrap(), camera, camera_transform);

        // Then cast the ray.
        let hit = rapier_context.cast_ray(ray_pos, ray_dir, f32::MAX, true, QueryFilter::new());
        if let Some((_entity, toi)) = hit {
            return Some(ray_pos + ray_dir * toi);
        }
    }
    None
}

// Credit to @doomy on discord.
fn ray_from_mouse_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> (Vec3, Vec3) {
    let mouse_position = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));

    let x = 2.0 * (mouse_position.x / window.width() as f32) - 1.0;
    let y = 2.0 * (mouse_position.y / window.height() as f32) - 1.0;

    let camera_inverse_matrix =
        camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let near = camera_inverse_matrix * Vec3::new(x, y, -1.0).extend(1.0);
    let far = camera_inverse_matrix * Vec3::new(x, y, 1.0).extend(1.0);

    let near = near.truncate() / near.w;
    let far = far.truncate() / far.w;
    let dir: Vec3 = far - near;
    (near, dir)
}

#[derive(Default)]
struct PolylineAssets {
    polyline: Handle<Polyline>,
}
#[derive(Default)]
struct PathToDisplay {
    steps: Vec<Vec2>,
}

fn setup_path_display(
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    commands.insert_resource(PathToDisplay::default());

    let polyline = polylines.add(Polyline {
        vertices: vec![-Vec3::ONE, Vec3::ONE],
        ..Default::default()
    });
    commands.insert_resource(PolylineAssets {
        polyline: polyline.clone(),
    });
    commands.spawn_bundle(PolylineBundle {
        polyline: polyline,
        material: polyline_materials.add(PolylineMaterial {
            width: 3.0,
            color: Color::RED,
            perspective: true,
            ..Default::default()
        }),
        ..Default::default()
    });
}

fn update_path_display(
    path_to_display: Res<PathToDisplay>,
    polyline: Res<PolylineAssets>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    if !path_to_display.is_changed() {
        return;
    }
    if let Some(polyline_to_change) = polylines.get_mut(&polyline.polyline) {
        polyline_to_change.vertices = path_to_display
            .steps
            .iter()
            .map(|s| Vec3::new(s.x, 0f32, s.y))
            .collect();
    }
}
