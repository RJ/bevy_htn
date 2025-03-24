use bevy::{color::palettes::css, prelude::*};
use std::time::Duration;

use crate::{coins::CoinCollected, CursorGroundCoords, GameState};
const CHARACTER_PATH: &str = "models/animated/character.glb";
const G: f32 = 80.0;

pub fn dude_plugin(app: &mut App) {
    app.register_type::<Anims>();
    app.register_type::<Cc>();
    app.add_systems(Startup, (setup_animations, spawn_player));
    app.add_systems(Update, setup_scene_once_loaded);
    app.add_systems(
        Update,
        (
            look_at_destination,
            sample_inputs,
            character_controller,
            update_anim,
        )
            .chain(),
    );
    app.add_observer(on_spawn_dude);
    app.add_observer(on_spawn_player);
}

/// Marker for all characters, human or AI
#[derive(Component)]
#[require(Name(|| Name::new("Boxy Dude")))]
#[require(Cc(|| Cc {
    speed: 50.0,
    jump_vel: 20.0,
    ..default()
}))]
pub struct Dude;

/// Controlled by human marker
#[derive(Component)]
pub struct Player;

#[derive(Default, Debug, Copy, Clone, PartialEq, Reflect)]
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
        .observe(on_coin_collected)
        .observe(on_control_animation);
}

// add a marker above the head of our own player
fn on_spawn_player(
    t: Trigger<OnAdd, Player>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.entity(t.entity()).insert((
        Name::new("Player Character"),
        Cc {
            speed: 70.0,
            jump_vel: 30.0,
            ..default()
        },
    ));
    commands
        .spawn((
            Name::new("Player Marker Mesh"),
            Mesh3d(meshes.add(Cone::new(0.2, 0.42))),
            MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
            Transform::from_translation(Vec3::new(0.0, 2.0, 0.0))
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
        ))
        .set_parent(t.entity());
}

fn spawn_player(mut commands: Commands) {
    // Character at 0,0,0
    commands.spawn((Player, Dude, Transform::from_scale(Vec3::ONE * 10.0)));
}

#[derive(Component, Default, Reflect)]
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

impl Cc {
    pub fn jump(&mut self) {
        self.do_jump = true;
    }
    pub fn goto(&mut self, dest: Vec2) {
        self.new_dest = Some(dest);
    }
}

#[derive(Component)]
pub struct AnimParent;

#[derive(Event)]
pub struct ControlAnimation {
    pub anim: Anims,
}

fn on_coin_collected(t: Trigger<CoinCollected>, mut q: Query<(&mut GameState, &Parent)>) {
    for (mut state, parent) in q.iter_mut() {
        if parent.get() == t.entity() {
            state.coins_collected += 1;
            return;
        }
    }
}

fn on_control_animation(
    t: Trigger<ControlAnimation>,
    animations: Res<Animations>,
    _q_char: Query<(&Cc, &Children)>,
    q_anim_parents: Query<(&Children, &Parent), With<AnimParent>>,
    mut q_anim: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let ControlAnimation { anim: desired_anim } = t.event();

    let Some(anim_children) = q_anim_parents
        .iter()
        .find(|(_children, parent)| parent.get() == t.entity())
        .map(|(children, _)| children)
    else {
        warn!("No anim_children found");
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
    mut q: Query<(&mut Cc, &Transform), (With<Player>, With<Dude>)>,
    ground_pos: Res<CursorGroundCoords>,
) {
    let Ok((mut cc, transform)) = q.get_single_mut() else {
        return;
    };
    cc.do_jump = mouse.pressed(MouseButton::Right);
    if mouse.pressed(MouseButton::Left) {
        cc.new_dest = Some(ground_pos.global.xz());
    } else if mouse.just_released(MouseButton::Left) {
        // change destination to a little way in front so we skid to a stop
        let dest = transform.translation + -transform.forward() * 7.0;
        cc.destination = Some(dest.xz());
    }
}

fn look_at_destination(
    mut q: Query<(&mut Transform, &Cc), (With<Dude>, Without<Player>)>,
    time: Res<Time>,
) {
    for (mut transform, cc) in q.iter_mut() {
        let Some(look_target) = cc.destination else {
            continue;
        };
        let target_position = Vec3::new(look_target.x, transform.translation.y, look_target.y);
        let direction = target_position - transform.translation;

        // Check if the direction vector is not zero
        if direction.length_squared() > 0.0 {
            let normalized_direction = direction.normalize();

            // Calculate the desired rotation
            let desired_rotation = Quat::from_rotation_arc(Vec3::Z, normalized_direction);

            // Maximum rotation speed in radians per second
            let max_rotation_speed = 5.0;
            let dt = time.delta_secs();

            // Interpolate the current rotation towards the desired rotation
            transform.rotation = transform
                .rotation
                .slerp(desired_rotation, max_rotation_speed * dt);
        }
    }
}

fn character_controller(
    mut q: Query<(&mut Transform, &mut Cc, Has<Player>), With<Dude>>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (mut transform, mut cc, is_player) in q.iter_mut() {
        let dt = time.delta_secs();

        // figure out the correct animation, defaulting to idle:
        cc.desired_anim = Anims::Idle;

        // consume new_dest if set
        if let Some(new_dest) = cc.new_dest.take() {
            cc.destination = Some(new_dest);
            cc.desired_anim = Anims::Walk;
            cc.stopping = false;
        }

        if let Some(dest) = cc.destination {
            // draw destination gizmo:
            let start = Vec3::new(dest.x, 20.0, dest.y);
            let end = Vec3::new(dest.x, 0.0, dest.y);
            gizmos.arrow(start, end, css::RED);
            // -----
            let dist = dest.distance(transform.translation.xz());
            let at_dest = dist < 3.0;
            if at_dest {
                // info!("Reached destination, clearing cc.destination");
                cc.destination = None;
                cc.desired_anim = Anims::Idle;
                cc.xz_vel = Vec2::ZERO;
            } else {
                let is_close_to_dest = dist < 20.0;
                let forward = -transform.forward();
                let speed_factor = if is_close_to_dest {
                    dist / 20.0 // Scale speed based on distance
                } else {
                    1.0
                };
                // Calculate acceleration
                let target_velocity = forward * cc.speed * speed_factor;
                let acceleration = 10.0;
                cc.xz_vel = cc.xz_vel.lerp(target_velocity.xz(), acceleration * dt);
                let vel = Vec3::new(cc.xz_vel.x, 0.0, cc.xz_vel.y);
                transform.translation += vel * dt;
                cc.desired_anim = Anims::Walk;
            }
        }

        // Check if the character is on the ground
        let on_ground = transform.translation.y <= 0.0;

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
    // q_parents: Query<&Parent>,
    mut players: Query<
        (Entity, &mut AnimationPlayer, &mut Transform, &Parent),
        Added<AnimationPlayer>,
    >,
) {
    for (entity, mut player, mut _transform, parent) in &mut players {
        // let character_entity = q_parents.root_ancestor(entity);
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
