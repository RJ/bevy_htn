use crate::*;
use bevy::prelude::*;

pub fn cursor_plugin(app: &mut App) {
    app.init_resource::<CursorGroundCoords>();
    app.add_systems(Update, cursor_to_ground_plane);
    app.add_systems(Update, look_at_cursor);
}

fn look_at_cursor(
    gc: Res<CursorGroundCoords>,
    mut q: Query<(&mut Transform, &Cc), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut transform, cc)) = q.get_single_mut() else {
        return;
    };

    let look_target = if let Some(d) = cc.destination {
        d
    } else {
        gc.global.xz()
    };

    let target_position = Vec3::new(look_target.x, transform.translation.y, look_target.y);
    let direction = target_position - transform.translation;

    // Check if the direction vector is not zero
    if direction.length_squared() > 0.0 {
        let normalized_direction = direction.normalize();

        // Calculate the desired rotation
        let desired_rotation = Quat::from_rotation_arc(Vec3::Z, normalized_direction);

        // Maximum rotation speed in radians per second
        let max_rotation_speed = 5.0; // Adjust this value as needed
        let dt = time.delta_secs();

        // Interpolate the current rotation towards the desired rotation
        transform.rotation = transform
            .rotation
            .slerp(desired_rotation, max_rotation_speed * dt);
    }
}

/// Here we will store the position of the mouse cursor on the 3D ground plane.
#[derive(Resource, Default)]
pub struct CursorGroundCoords {
    // Global (world-space) coordinates
    pub global: Vec3,
    // Local (relative to the ground plane) coordinates
    pub local: Vec2,
}

/// Used to help identify our ground plane
#[derive(Component)]
pub struct GroundPlaneMarker;

fn cursor_to_ground_plane(
    mut mycoords: ResMut<CursorGroundCoords>,
    // query to get the window (so we can read the current cursor position)
    // (we will only work with the primary window)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    // query to get ground plane's transform
    q_plane: Query<&GlobalTransform, With<GroundPlaneMarker>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // Ditto for the ground plane's transform
    let ground_transform = q_plane.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    // Mathematically, we can represent the ground as an infinite flat plane.
    // To do that, we need a point (to position the plane) and a normal vector
    // (the "up" direction, perpendicular to the ground plane).

    // We can get the correct values from the ground entity's GlobalTransform
    let plane_origin = ground_transform.translation();
    let plane = InfinitePlane3d::new(ground_transform.up());

    // Ask Bevy to give us a ray pointing from the viewport (screen) into the world
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    // do a ray-plane intersection test, giving us the distance to the ground
    let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
        // If the ray does not intersect the ground
        // (the camera is not looking towards the ground), we can't do anything
        return;
    };

    // use the distance to compute the actual point on the ground in world-space
    let global_cursor = ray.get_point(distance);

    mycoords.global = global_cursor;
    // eprintln!(
    //     "Global cursor coords: {}/{}/{}",
    //     global_cursor.x, global_cursor.y, global_cursor.z
    // );

    // to compute the local coordinates, we need the inverse of the plane's transform
    let inverse_transform_matrix = ground_transform.compute_matrix().inverse();
    let local_cursor = inverse_transform_matrix.transform_point3(global_cursor);

    // we can discard the Y coordinate, because it should always be zero
    // (our point is supposed to be on the plane)
    mycoords.local = local_cursor.xz();
    // eprintln!("Local cursor coords: {}/{}", local_cursor.x, local_cursor.z);
}
