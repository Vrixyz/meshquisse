mod navmesh;

use bevy::prelude::*;

use bevy_rapier3d::prelude::*;
use navmesh::NavMeshPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(
            0xF9 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            0xFF as f32 / 255.0,
        )))
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(NavMeshPlugin)
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(cast_ray)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-30.0, 30.0, 100.0)
            .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
        ..Default::default()
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

fn cast_ray(
    mut commands: Commands,
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    // We will color in read the colliders hovered by the mouse.
    for (camera, camera_transform) in cameras.iter() {
        // First, compute a ray from the mouse position.
        let (ray_pos, ray_dir) =
            ray_from_mouse_position(windows.get_primary().unwrap(), camera, camera_transform);

        // Then cast the ray.
        let hit = rapier_context.cast_ray(ray_pos, ray_dir, f32::MAX, true, QueryFilter::new());

        if let Some((_entity, toi)) = hit {
            let cube_size = 0.5f32;
            let mut position = ray_pos + ray_dir * toi;
            position.y += cube_size;
            commands
                .spawn_bundle(TransformBundle::from(Transform::from_translation(position)))
                .insert(RigidBody::Dynamic)
                .insert(Collider::cuboid(cube_size, cube_size, cube_size));
        }
    }
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
