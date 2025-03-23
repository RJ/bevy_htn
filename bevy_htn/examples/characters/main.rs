use bevy_htn::prelude::*;
use bevy_panorbit_camera::*;

use bevy::{
    animation::{AnimationTargetId, RepeatAnimation},
    color::palettes::css::{self, WHITE},
    input::common_conditions::input_toggle_active,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::{thread_rng, Rng};
use std::{f32::consts::PI, time::Duration};

const FOX_PATH: &str = "models/animated/character.glb";
const G: f32 = 80.0;

#[derive(Default)]
enum Anims {
    Jump = 0,
    #[default]
    Idle = 1,
    Walk = 2,
}
// anims: idle, jump, walk

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
        })
        .add_plugins(DefaultPlugins)
        // .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(
            // Show inspector with F12
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F12)),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, setup_scene_once_loaded)
        .add_systems(Update, (keyboard_animation_control, character_controller))
        .add_plugins(cursor_plugin)
        .run();
}

#[derive(Component, Default)]
struct Cc {
    y_vel: f32,
    xz_vel: Vec2,
    speed: f32,
    jump_vel: f32,
    desired_anim: Anims,
    stopping: bool,
    destination: Option<Vec2>,
}

#[derive(Component)]
#[require(Cc(|| Cc{speed: 70.0, jump_vel: 30.0, ..default()}))]
struct Player;

fn character_controller(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut q: Query<(&mut Transform, &mut Cc), With<Player>>,
    ground_pos: Res<MyGroundCoords>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let Ok((mut transform, mut cc)) = q.get_single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    if mouse.pressed(MouseButton::Left) {
        cc.destination = Some(ground_pos.global.xz());
        cc.stopping = false;
    } else if !cc.stopping && cc.destination.is_some() {
        // set destination to a little way in front so we skid to a stop
        let dest = transform.translation + -transform.forward() * 10.0;
        cc.destination = Some(dest.xz());
        cc.stopping = true;
    }
    // draw destination gizmo
    if let Some(dest) = cc.destination {
        let start = Vec3::new(dest.x, 20.0, dest.y);
        let end = Vec3::new(dest.x, 0.0, dest.y);
        gizmos.arrow(start, end, css::RED);
    }

    // destination and distance to it, ignoring height (y)
    let distance_to_destination = cc
        .destination
        .map(|d| d.distance(transform.translation.xz()));
    let at_dest = distance_to_destination.map_or(true, |d| d < 3.0);
    let is_close_to_dest = distance_to_destination.is_some_and(|d| d < 25.0);

    if at_dest {
        cc.destination = None;
        cc.desired_anim = Anims::Idle;
        cc.xz_vel = Vec2::ZERO;
    }

    // Check if the character is on the ground
    let on_ground = transform.translation.y <= 0.0;

    // Walk forward when "Q" is pressed
    if !at_dest && cc.destination.is_some() {
        let forward = -transform.forward();
        let speed_factor = if is_close_to_dest {
            distance_to_destination.unwrap() / 20.0 // Scale speed based on distance
        } else {
            1.0
        };
        transform.translation += forward * cc.speed * speed_factor * dt;
        cc.desired_anim = Anims::Walk;
    }

    // Apply gravity if the character is not on the ground
    if !on_ground {
        cc.y_vel -= G * dt; // Gravity
        cc.desired_anim = Anims::Jump;
    } else {
        cc.desired_anim = Anims::Idle;
        cc.y_vel = 0.0; // Reset vertical velocity when on the ground
        transform.translation.y = 0.0; // Ensure character stays on the ground
    }

    // Jump when "W" is pressed and the character is on the ground
    if mouse.just_pressed(MouseButton::Right) && on_ground {
        cc.y_vel = cc.jump_vel; // Initial jump velocity
        cc.desired_anim = Anims::Jump;
    }

    // Update the character's position
    transform.translation.y += cc.y_vel * dt;
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(FOX_PATH)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(FOX_PATH)),
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(FOX_PATH)),
    ]);

    info!("{graph:#?}");

    // Insert a resource with the current scene information
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph: graph_handle,
    });

    // Camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        PanOrbitCamera::default(),
        Transform::from_xyz(50., 40., 100.).looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
    ));

    // Plane
    commands.spawn((
        Name::new("Plane"),
        MyGroundPlane,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Light
    commands.spawn((
        Name::new("Light"),
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
    ));

    // Fox
    commands.spawn((
        Name::new("Character"),
        Player,
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(FOX_PATH))),
    ));

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - digit 1 / 3 / 5: play the animation <digit> times");
    println!("  - L: loop the animation forever");
    println!("  - return: change animation");
}

// An `AnimationPlayer` is automatically added to the scene when it's ready.
// When the player is added, start the animation.
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    graphs: Res<Assets<AnimationGraph>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    mut players: Query<(Entity, &mut AnimationPlayer, &mut Transform), Added<AnimationPlayer>>,
) {
    fn get_clip<'a>(
        node: AnimationNodeIndex,
        graph: &AnimationGraph,
        clips: &'a mut Assets<AnimationClip>,
    ) -> &'a mut AnimationClip {
        let node = graph.get(node).unwrap();
        let clip = match &node.node_type {
            AnimationNodeType::Clip(handle) => clips.get_mut(handle),
            _ => unreachable!(),
        };
        clip.unwrap()
    }

    for (entity, mut player, mut transform) in &mut players {
        let graph = graphs.get(&animations.graph).unwrap();
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(
                &mut player,
                animations.animations[Anims::Idle as usize],
                Duration::ZERO,
            )
            .repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph.clone()))
            .insert(transitions);

        transform.scale = Vec3::ONE * 10.0;
    }
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for (mut player, mut transitions) in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };

        if keyboard_input.just_pressed(KeyCode::Space) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            if playing_animation.is_paused() {
                playing_animation.resume();
            } else {
                playing_animation.pause();
            }
        }

        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 1.2);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 0.8);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time();
            playing_animation.seek_to(elapsed - 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time();
            playing_animation.seek_to(elapsed + 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Enter) {
            *current_animation = (*current_animation + 1) % animations.animations.len();

            transitions
                .play(
                    &mut player,
                    animations.animations[*current_animation],
                    Duration::from_millis(250),
                )
                .repeat();
        }

        if keyboard_input.just_pressed(KeyCode::Digit8) {
            transitions
                .play(
                    &mut player,
                    animations.animations[Anims::Jump as usize],
                    Duration::from_millis(250),
                )
                .repeat();
        }
        if keyboard_input.just_pressed(KeyCode::Digit9) {
            transitions
                .play(
                    &mut player,
                    animations.animations[Anims::Walk as usize],
                    Duration::from_millis(250),
                )
                .repeat();
        }

        if keyboard_input.just_pressed(KeyCode::Digit0) {
            transitions
                .play(
                    &mut player,
                    animations.animations[Anims::Idle as usize],
                    Duration::from_millis(250),
                )
                .repeat();
        }

        if keyboard_input.just_pressed(KeyCode::Digit1) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(1))
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::Digit3) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(3))
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::Digit5) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(5))
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::KeyL) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation.set_repeat(RepeatAnimation::Forever);
        }
    }
}

fn look_at_cursor(gc: Res<MyGroundCoords>, mut q: Query<(&mut Transform, &Cc), With<Player>>) {
    let Ok((mut transform, cc)) = q.get_single_mut() else {
        return;
    };
    let look_target = if let Some(d) = cc.destination {
        d
    } else {
        gc.global.xz()
    };
    let target = 2.0 * transform.translation.xz() - look_target;
    // dont look directly at the ground when jumping
    let target = Vec3::new(target.x, transform.translation.y, target.y);
    transform.look_at(target, Vec3::Y);
}

fn cursor_plugin(app: &mut App) {
    app.init_resource::<MyGroundCoords>();
    app.add_systems(Update, cursor_to_ground_plane);
    app.add_systems(Update, look_at_cursor);
}

/// Here we will store the position of the mouse cursor on the 3D ground plane.
#[derive(Resource, Default)]
struct MyGroundCoords {
    // Global (world-space) coordinates
    global: Vec3,
    // Local (relative to the ground plane) coordinates
    local: Vec2,
}

/// Used to help identify our ground plane
#[derive(Component)]
struct MyGroundPlane;

fn cursor_to_ground_plane(
    mut mycoords: ResMut<MyGroundCoords>,
    // query to get the window (so we can read the current cursor position)
    // (we will only work with the primary window)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    // query to get ground plane's transform
    q_plane: Query<&GlobalTransform, With<MyGroundPlane>>,
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
