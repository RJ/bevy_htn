use bevy_htn::prelude::*;

mod boxy_dude;
mod coins;
mod cursor;
mod label;
mod setup;
use bevy::{
    color::palettes::css,
    input::common_conditions::{input_just_pressed, input_toggle_active},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use boxy_dude::*;
use coins::*;
use cursor::*;
use label::*;
use setup::*;
use std::f32::consts::PI;

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Config {
    pub draw_gizmos: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .register_type::<Config>()
        // overhead floaty label plugin, to aid debugging:
        .add_plugins(label_plugin)
        // Add camera, ground plane, light, etc
        .add_plugins(setup_plugin)
        // Track cursor position on the ground plane, make player look towards cursor
        .add_plugins(cursor_plugin)
        // Periodic coin spawner and coin collision events
        .add_plugins(coin_plugin)
        // Character controller for all little boxy dudes, and input controller for human.
        .add_plugins(dude_plugin)
        // Show inspector with F12
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F12)),
        )
        .add_plugins(
            ResourceInspectorPlugin::<Config>::default()
                .run_if(input_toggle_active(true, KeyCode::F11)),
        )
        .add_systems(
            Update,
            draw_debug.run_if(|conf: Res<Config>| conf.draw_gizmos),
        )
        .add_systems(Startup, add_bot)
        .add_systems(Update, add_bot.run_if(input_just_pressed(KeyCode::KeyB)))
        .add_observer(on_add_bots)
        .add_plugins(htn_plugins)
        .init_resource::<Config>()
        .run();
}

fn add_bot(mut commands: Commands) {
    commands.trigger(AddBots(1));
}

#[derive(Event)]
struct AddBots(usize);

fn on_add_bots(t: Trigger<AddBots>, mut commands: Commands, level_config: Res<LevelConfig>) {
    let num = t.event().0;
    for _ in 0..num {
        let (x, z) = level_config.random_position();
        let pos = Vec3::new(x, 10.0, z);
        let bot_id = commands
            .spawn((
                Name::new("Bot"),
                Dude,
                Transform::from_scale(Vec3::ONE * 10.0).with_translation(pos),
                OverheadLabel::new("Bot"),
            ))
            .id();
        info!("Adding bot: {bot_id}");
    }
}

fn draw_debug(mut gizmos: Gizmos, level_config: Res<LevelConfig>) {
    let iso = Quat::from_rotation_arc(Vec3::Z, Vec3::Y);
    gizmos.rect(
        iso,
        Vec2::new(level_config.width, level_config.height),
        css::BLUE,
    );
}

// HTN stuff ---

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub enum Mood {
    #[default]
    Normal,
    Scared,
}

#[derive(Reflect, Component, Clone, Debug, Default)]
#[reflect(Default, Component)]
pub struct GameState {
    pub proximity_alert: bool,
    pub player_location: Vec2,
    pub next_destination: Option<Vec2>,
    pub coin_location: Option<Vec2>,
    pub coins_collected: i32,
    pub mood: Mood,
    pub scared_at_time: Option<f32>,
}

#[derive(Resource)]
pub struct Htns {
    pub dude: Handle<HtnAsset<GameState>>,
}

#[derive(Debug, Reflect, Default, Clone, Component, HtnOperator)]
#[reflect(Default, HtnOperator)]
#[spawn_named = "Move To"]
pub struct MoveToOperator(pub Option<Vec2>);

#[derive(Debug, Reflect, Default, Clone, Component, HtnOperator)]
#[reflect(Default, HtnOperator)]
#[spawn_named = "Spin"]
#[require(BehaveTimeout(||BehaveTimeout::from_secs(3.0, true)))]
pub struct SpinOperator;

#[derive(Debug, Reflect, Default, Clone, Component, HtnOperator)]
#[reflect(Default, HtnOperator)]
#[spawn_named = "Cower"]
#[require(BehaveTimeout(||BehaveTimeout::from_secs(5.0, true)))]
pub struct CowerOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct BecomeScaredOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct PickDestinationOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct PrepareToFleeOperator(pub Vec2);

// TODO runtime err when operator in htn says Foo(bar) and Foo is a bare struct with no param.

#[derive(Event)]
pub struct HtnLoaded;

fn htn_plugins(app: &mut App) {
    app.register_type::<PickDestinationOperator>();
    app.register_type::<PrepareToFleeOperator>();
    app.register_type::<WaitOperator>();
    app.register_type::<SpinOperator>();
    app.register_type::<CowerOperator>();
    app.register_type::<BecomeScaredOperator>();
    app.add_plugins(HtnAssetPlugin::<GameState>::default());
    app.add_plugins(HtnPlugin::<GameState>::default());
    app.add_systems(PreStartup, load_htn);
    app.register_type::<SpinOperator>();
    app.add_systems(
        Update,
        check_asset_loaded.run_if(on_event::<AssetEvent<HtnAsset<GameState>>>),
    );
    app.add_observer(on_htn_loaded);
    app.add_plugins(move_to_operator_plugin);
    app.add_observer(decorate_dudes);
    app.add_observer(on_pick_destination);
    app.add_observer(on_prepare_to_flee);
    app.add_observer(on_become_scared);
    app.add_systems(Update, enemy_sensors);
    app.add_systems(Update, spin_system);
    app.add_systems(Update, cower_system);
}

fn check_asset_loaded(
    mut ev_asset: EventReader<AssetEvent<HtnAsset<GameState>>>,
    mut commands: Commands,
) {
    for ev in ev_asset.read() {
        info!("HTN asset event: {:?}", ev);
        if let AssetEvent::LoadedWithDependencies { .. } = ev {
            info!("HTN asset loaded");
            commands.trigger(HtnLoaded);
            break;
        }
    }
}

#[derive(Debug, Reflect, Clone, Component)]
#[reflect(Default, HtnOperator)]
pub struct WaitOperator(pub i32);
impl Default for WaitOperator {
    fn default() -> Self {
        Self(3)
    }
}

// if you remove the HtnOperator Derive (and spawn_named) you can manually provide a tree:
impl HtnOperator for WaitOperator {
    fn to_tree(&self) -> Tree<Behave> {
        behave! {
            Behave::Wait(self.0 as f32)
        }
    }
}

// Pick destination operator does everything in the trigger then reports success.
// just pick a random location within the level bounds, and update the game state.
fn on_pick_destination(
    t: Trigger<BehaveTrigger<PickDestinationOperator>>,
    mut commands: Commands,
    mut q: Query<&mut GameState>,
    level_config: Res<LevelConfig>,
) {
    let ctx = t.ctx();
    let mut state = q
        .get_mut(
            ctx.supervisor_entity()
                .expect("Supervisor entity not found"),
        )
        .expect("GameState not found");
    let (x, z) = level_config.random_position();
    state.next_destination = Some(Vec2::new(x, z));
    commands.trigger(ctx.success());
}

fn on_prepare_to_flee(
    t: Trigger<BehaveTrigger<PrepareToFleeOperator>>,
    mut commands: Commands,
    mut q: Query<&mut GameState>,
    q_own_location: Query<&Transform>,
    level_config: Res<LevelConfig>,
) {
    let ctx = t.ctx();
    let mut state = q
        .get_mut(
            ctx.supervisor_entity()
                .expect("Supervisor entity not found"),
        )
        .expect("GameState not found");
    let own_location = q_own_location
        .get(ctx.target_entity())
        .expect("Own location not found");
    let flee_from = state.player_location;
    // Get direction away from threat
    let away_dir = (own_location.translation.xz() - flee_from).normalize();

    // Add random rotation between -1 and 1 radian
    let random_angle = rand::random::<f32>() * 2.0 - 1.0;
    let rotated_dir = Vec2::from_angle(random_angle).rotate(away_dir);

    let flee_distance = rand::random::<f32>() * 100.0 + 50.0;
    let new_location = own_location.translation.xz() + rotated_dir * flee_distance;

    // clamp to level bounds
    let new_location = Vec2::new(
        new_location
            .x
            .clamp(-level_config.width, level_config.width),
        new_location
            .y
            .clamp(-level_config.height, level_config.height),
    );

    state.next_destination = Some(new_location);

    // let (x, z) = level_config.random_position();
    // state.next_destination = Some(Vec2::new(x, z));
    commands.trigger(ctx.success());
}

fn on_become_scared(
    t: Trigger<BehaveTrigger<BecomeScaredOperator>>,
    mut commands: Commands,
    mut q: Query<&mut GameState>,
    mut q_cc: Query<&mut Cc>,
    time: Res<Time>,
) {
    let ctx = t.ctx();
    let mut state = q
        .get_mut(
            ctx.supervisor_entity()
                .expect("Supervisor entity not found"),
        )
        .expect("GameState not found");
    state.scared_at_time = Some(time.elapsed_secs());
    state.mood = Mood::Scared;
    let mut cc = q_cc.get_mut(ctx.target_entity()).expect("Dude not found");
    cc.jump();
    commands.trigger(ctx.success());
}

// Add the HTN supervisor child to all dude entities (not the player though)
fn decorate_dudes(
    t: Trigger<OnAdd, Dude>,
    mut commands: Commands,
    htns: Res<Htns>,
    q: Query<Entity, (With<Dude>, Without<Player>)>,
) {
    if !q.contains(t.entity()) {
        return;
    }
    let htn_sup = commands
        .entity(t.entity())
        .spawn_htn_supervisor(htns.dude.clone(), &GameState::default());
    commands.trigger_targets(ReplanRequest, htn_sup);
}

fn on_htn_loaded(
    _t: Trigger<HtnLoaded>,
    htns: Res<Htns>,
    mut commands: Commands,
    q: Query<Entity, With<HtnSupervisor<GameState>>>,
    atr: Res<AppTypeRegistry>,
    assets: Res<Assets<HtnAsset<GameState>>>,
) {
    let htn = &assets.get(htns.dude.id()).unwrap().htn;
    match htn.verify_all(&GameState::default(), &atr) {
        Ok(_) => info!("HTN verified"),
        Err(e) => panic!("HTN verification failed: {:#?}", e),
    }
    q.iter()
        .for_each(|e| commands.trigger_targets(ReplanRequest, e));
}

fn load_htn(mut commands: Commands, assets: Res<AssetServer>) {
    let dude_htn = assets.load("dude.htn");
    commands.insert_resource(Htns { dude: dude_htn });
}

fn move_to_operator_plugin(app: &mut App) {
    app.register_type::<MoveToOperator>();
    app.add_systems(Update, move_to_system);
    app.add_observer(on_add_move_to);
}

// character controller already handles move to, so we update it when MoveToOperator is added
fn on_add_move_to(
    t: Trigger<OnInsert, MoveToOperator>,
    q_ctx: Query<(&MoveToOperator, &BehaveCtx)>,
    mut q: Query<&mut Cc, With<Dude>>,
) {
    let (move_to, ctx) = q_ctx.get(t.entity()).expect("Context not found");
    let mut cc = q.get_mut(ctx.target_entity()).expect("Dude not found");
    cc.goto(move_to.0.unwrap());
}

// all we need to do here is trigger success when the dude is at the destination
fn move_to_system(
    q_behave: Query<(&BehaveCtx, &MoveToOperator)>,
    mut q_dude: Query<&Transform, With<Dude>>,
    mut commands: Commands,
) {
    for (ctx, move_to) in q_behave.iter() {
        let dude_transform = q_dude.get_mut(ctx.target_entity()).expect("Dude not found");
        let dist = dude_transform.translation.xz().distance(move_to.0.unwrap());
        if dist < 3.0 {
            commands.trigger(ctx.success());
        }
    }
}

// spin around to celebrate
// we rely on BehaveTimeout, a required component of SpinOperator, to trigger success of this behaviour.
fn spin_system(
    q_behave: Query<(&BehaveCtx, &SpinOperator)>,
    mut q_dude: Query<&mut Transform, With<Dude>>,
    time: Res<Time>,
) {
    for (ctx, _spin) in q_behave.iter() {
        let mut dude_transform = q_dude.get_mut(ctx.target_entity()).expect("Dude not found");
        dude_transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 10.0);
    }
}
// while cowering, we turn our back to the player
// we rely on BehaveTimeout, a required component of CowerOperator, to trigger success of this behaviour.
fn cower_system(
    q_behave: Query<(&BehaveCtx, &CowerOperator)>,
    mut q_dude: Query<&mut Transform, (Without<Player>, With<Dude>)>,
    q_player: Query<&Transform, With<Player>>,
) {
    let player_trans = q_player.single();
    let look_target = player_trans.translation.xz();
    for (ctx, _cower) in q_behave.iter() {
        let mut transform = q_dude.get_mut(ctx.target_entity()).expect("Dude not found");
        let target_position = Vec3::new(look_target.x, transform.translation.y, look_target.y);
        let direction = target_position - transform.translation;
        // face away from the player
        let desired_rotation = Quat::from_rotation_arc(Vec3::Z, -direction.normalize());
        transform.rotation = desired_rotation;
    }
}

fn enemy_sensors(
    q_npc: Query<(&Transform, &Children), (With<Dude>, Without<Player>)>,
    mut q_sups: Query<(Entity, &mut GameState)>,
    q_player: Query<&Transform, With<Player>>,
    q_coins: Query<&Transform, With<Coin>>,
    time: Res<Time>,
) {
    let player_trans = q_player.single();

    for (npc_trans, children) in q_npc.iter() {
        let Some((_sup_ent, mut state)) = q_sups.iter_mut().find(|(e, _)| children.contains(&e))
        else {
            continue;
        };
        let dist = npc_trans
            .translation
            .xz()
            .distance(player_trans.translation.xz());
        let proximate = dist < 10.0;
        if state.proximity_alert != proximate {
            state.proximity_alert = proximate;
        }
        state.bypass_change_detection().player_location = player_trans.translation.xz();
        if let Some(scared_at_time) = state.scared_at_time {
            if time.elapsed_secs() - scared_at_time > 7.0 {
                state.scared_at_time = None;
                state.mood = Mood::Normal;
            }
        }
        // detect coin locations
        if let Some(coin_trans) = q_coins.iter().next() {
            if state.coin_location != Some(coin_trans.translation.xz()) {
                state.coin_location = Some(coin_trans.translation.xz());
            }
        } else if state.coin_location.is_some() {
            state.coin_location = None;
        }
    }
}
