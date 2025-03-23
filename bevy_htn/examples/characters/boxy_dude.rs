use bevy::{color::palettes::css, prelude::*};
use std::time::Duration;

use crate::CursorGroundCoords;
const CHARACTER_PATH: &str = "models/animated/character.glb";
const G: f32 = 80.0;

pub fn dude_plugin(app: &mut App) {
    app.add_systems(Startup, (setup_animations, spawn_player));
    app.add_systems(Update, setup_scene_once_loaded);
    app.add_systems(
        Update,
        (sample_inputs, character_controller, update_anim).chain(),
    );
    app.add_observer(on_spawn_dude);
}

/// Marker for all characters, human or AI
#[derive(Component)]
#[require(Name(|| Name::new("Dude")))]
#[require(Cc(|| Cc{speed: 70.0, jump_vel: 30.0, ..default()}))]
pub struct Dude;

/// Controlled by human marker
#[derive(Component)]
pub struct Player;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum Anims {
    Jump = 0,
    #[default]
    Idle = 1,
    Walk = 2,
}

fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(CHARACTER_PATH)),
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(CHARACTER_PATH)),
    ]);

    // info!("{graph:#?}");

    // Insert a resource with the current scene information
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph: graph_handle,
    });
}

fn on_spawn_dude(t: Trigger<OnAdd, Dude>, mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .entity(t.entity())
        .insert(SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(CHARACTER_PATH)),
        ))
        .observe(on_control_animation);
}

fn spawn_player(mut commands: Commands) {
    // Character
    commands.spawn((
        Name::new("Character"),
        Player,
        Dude,
        Transform::from_scale(Vec3::ONE * 10.0),
    ));
}

#[derive(Component, Default)]
pub struct Cc {
    y_vel: f32,
    xz_vel: Vec2,
    speed: f32,
    do_jump: bool,
    new_dest: Option<Vec2>,
    jump_vel: f32,
    desired_anim: Anims,
    stopping: bool,
    pub destination: Option<Vec2>,
}

#[derive(Component)]
pub struct AnimParent;

#[derive(Event)]
pub struct ControlAnimation {
    pub anim: Anims,
}

fn on_control_animation(
    t: Trigger<ControlAnimation>,
    animations: Res<Animations>,
    q_char: Query<(&Cc, &Children)>,
    q_anim_parents: Query<&Children, With<AnimParent>>,
    mut q_anim: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let ControlAnimation { anim: desired_anim } = t.event();
    let Ok((_cc, children)) = q_char.get(t.entity()) else {
        warn!("No character found");
        return;
    };
    let anim_parent = children[0];
    let Ok(anim_children) = q_anim_parents.get(anim_parent) else {
        warn!("No anim parent found: {anim_parent}");
        return;
    };
    let e = anim_children[0];
    let Ok((mut player, mut transitions)) = q_anim.get_mut(e) else {
        warn!("No anim player found: {e}");
        return;
    };
    // let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
    //     warn!("No playing animation found");
    //     return;
    // };
    // info!("Playing animation index: {playing_animation_index}");
    let transition_duration = if *desired_anim == Anims::Idle {
        Duration::from_millis(150)
    } else {
        Duration::ZERO
    };
    transitions
        .play(
            &mut player,
            animations.animations[*desired_anim as usize],
            transition_duration,
        )
        .repeat();
}

// map mouse inputs to player's CC.
fn sample_inputs(
    mouse: Res<ButtonInput<MouseButton>>,
    mut q: Query<&mut Cc, (With<Player>, With<Dude>)>,
    ground_pos: Res<CursorGroundCoords>,
) {
    let Ok(mut cc) = q.get_single_mut() else {
        return;
    };
    cc.do_jump = mouse.pressed(MouseButton::Right);
    if mouse.pressed(MouseButton::Left) {
        cc.new_dest = Some(ground_pos.global.xz());
    }
}

fn character_controller(
    mut q: Query<(&mut Transform, &mut Cc), With<Dude>>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let Ok((mut transform, mut cc)) = q.get_single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    // figure out the correct animation, defaulting to idle:
    cc.desired_anim = Anims::Idle;

    // consume new_dest if set
    if let Some(new_dest) = cc.new_dest.take() {
        // info!("New destination: {new_dest}");
        cc.destination = Some(new_dest);
        cc.desired_anim = Anims::Walk;
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

    if !at_dest && cc.destination.is_some() {
        let forward = -transform.forward();
        let speed_factor = if is_close_to_dest {
            distance_to_destination.unwrap() / 20.0 // Scale speed based on distance
        } else {
            1.0
        };

        // Calculate acceleration
        let target_velocity = forward * cc.speed * speed_factor;
        let acceleration = 10.0; // Adjust this value for faster/slower acceleration
        cc.xz_vel = cc.xz_vel.lerp(target_velocity.xz(), acceleration * dt);
        let vel = Vec3::new(cc.xz_vel.x, 0.0, cc.xz_vel.y);
        transform.translation += vel * dt;
        cc.desired_anim = Anims::Walk;
    }

    // Apply gravity if the character is not on the ground
    if !on_ground {
        cc.y_vel -= G * dt; // Gravity
        cc.desired_anim = Anims::Jump;
    } else {
        cc.y_vel = 0.0; // Reset vertical velocity when on the ground
        transform.translation.y = 0.0; // Ensure character stays on the ground
    }

    // consume do_jump if set
    if cc.do_jump {
        cc.do_jump = false;
        if on_ground {
            cc.y_vel = cc.jump_vel; // Initial jump velocity
            cc.desired_anim = Anims::Jump;
        }
    }

    // Update the character's position
    transform.translation.y += cc.y_vel * dt;
}

fn update_anim(
    mut q: Query<(Entity, &Cc)>,
    mut current_animation: Local<bevy::utils::HashMap<Entity, usize>>,
    mut commands: Commands,
) {
    for (entity, cc) in &mut q {
        let desired = cc.desired_anim as usize;
        if current_animation.get(&entity) != Some(&desired) {
            // if *current_animation != cc.desired_anim as usize {
            // info!("{entity} Updating anim to {:?}", cc.desired_anim);
            *current_animation
                .entry(entity)
                .or_insert(cc.desired_anim as usize) = cc.desired_anim as usize;
            commands.entity(entity).trigger(ControlAnimation {
                anim: cc.desired_anim,
            });
        }
    }
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

// An `AnimationPlayer` is automatically added to the scene when it's ready.
// When the player is added, start the animation.
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    q_parents: Query<&Parent>,
    mut players: Query<
        (Entity, &mut AnimationPlayer, &mut Transform, &Parent),
        Added<AnimationPlayer>,
    >,
) {
    for (entity, mut player, mut _transform, parent) in &mut players {
        let character_entity = q_parents.root_ancestor(entity);
        // info!("Character entity: {character_entity} GOT animation bits");
        let mut transitions = AnimationTransitions::new();

        commands
            .entity(parent.get())
            .insert((AnimParent, Name::new("Anim Parent")));
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
            .insert(transitions)
            .insert(Name::new("Animation Transitions Here"));
    }
}
