// use bevy::color::palettes::css;
use bevy::prelude::*;
// use bevy::reflect::TypeRegistry;
use bevy_htn::prelude::*;

// use bevy_inspector_egui::bevy_egui;
// use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay, prelude::*, DefaultInspectorConfigPlugin,
};
mod ui;
use ui::*;
mod setup_level;
use setup_level::*;
mod operators;
use operators::*;
mod operator_plugins;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub enum Location {
    #[default]
    Unknown,
    Player,
    Trunk,
    Bridge1,
    Bridge2,
    Bridge3,
}

#[derive(Reflect, Component, Clone, Debug, Default, InspectorOptions)]
#[reflect(Default, Component, InspectorOptions)]
pub struct GameState {
    pub location_enum: Location,
    pub location: Vec2,
    #[inspector(min = 0, max = 3, display = NumberDisplay::Slider)]
    pub trunk_health: i32,
    // true if found_trunk_location is set
    pub found_trunk: bool,
    pub found_trunk_location: Vec2,
    pub can_navigate_to_enemy: bool,
    pub attacked_recently: bool,
    pub can_see_enemy: bool,
    pub has_seen_enemy_recently: bool,
    pub last_enemy_location: Vec2,
    pub next_bridge_to_check: usize,
    pub within_melee_range: bool,
    pub within_trunk_pickup_range: bool,
    pub dummy_field: bool,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultInspectorConfigPlugin);
    app.add_plugins(HtnAssetPlugin::<GameState>::default());
    app.add_plugins(operator_plugins::OperatorPlugins);
    app.add_plugins(TrollUiPlugin);
    // app.add_plugins(ResourceInspectorPlugin::<GameState>::default());
    app.add_plugins(HtnPlugin::<GameState>::default());
    app.add_plugins(setup_level);
    // app.register_type::<SellGold>();
    // app.add_observer(on_add_sellgold);

    app.add_systems(OnEnter(LoadingState::Ready), setup_troll_htn_supervisor);
    app.add_systems(OnEnter(LoadingState::SpawningEntities), print_htn);
    app.add_systems(OnEnter(LoadingState::Ready), trigger_first_plan);
    app.add_systems(Update, troll_enemy_vision_sensor);

    app.run();
}

// ask eveery supervisor to replan.
fn trigger_first_plan(q: Query<Entity, With<HtnSupervisor<GameState>>>, mut commands: Commands) {
    info!("Triggering first plan 🥇");
    q.iter()
        .for_each(|e| commands.trigger_targets(ReplanRequest, e));
}

fn initial_gamestate() -> GameState {
    GameState {
        location_enum: Location::Unknown,
        location: Vec2::new(1., 1.),
        trunk_health: 3,
        found_trunk: false,
        found_trunk_location: Vec2::new(2., 2.),
        can_navigate_to_enemy: true,
        attacked_recently: false,
        can_see_enemy: false,
        has_seen_enemy_recently: false,
        last_enemy_location: Vec2::new(666., 666.),
        next_bridge_to_check: 1,
        within_melee_range: false,
        within_trunk_pickup_range: false,
        dummy_field: false,
    }
}

// doing check and set here to avoid triggering change detection by setting a field
// to it's existing value.
fn troll_enemy_vision_sensor(
    mut q: Query<&mut GameState>,
    q_troll: Query<&Transform, With<Troll>>,
    q_player: Query<&Transform, With<Player>>,
    q_trunks: Query<&Transform, With<Trunk>>,
    mut last_seen: Local<f32>,
    time: Res<Time>,
) {
    let Ok(mut state) = q.get_single_mut() else {
        return;
    };
    let troll_transform = q_troll.single();
    let player_transform = q_player.single();

    let distance = troll_transform
        .translation
        .xy()
        .distance(player_transform.translation.xy());
    // plus half player radius
    let can_see_enemy = distance < TROLL_VISION_RADIUS + 15.0;
    if state.can_see_enemy != can_see_enemy {
        state.can_see_enemy = can_see_enemy;
        if can_see_enemy {
            state.has_seen_enemy_recently = true;
            *last_seen = time.elapsed_secs();
        }
    }
    if !state.can_see_enemy
        && state.has_seen_enemy_recently
        && time.elapsed_secs() - *last_seen > 5.0
    {
        state.has_seen_enemy_recently = false;
    }
    if can_see_enemy && state.last_enemy_location != player_transform.translation.xy() {
        state.bypass_change_detection().last_enemy_location = player_transform.translation.xy();
    }

    let within_melee_range = distance < TROLL_MELEE_RANGE;
    if state.within_melee_range != within_melee_range {
        state.within_melee_range = within_melee_range;
    }

    let mut within_trunk_pickup_range = false;
    for trunk_transform in q_trunks.iter() {
        let distance = troll_transform
            .translation
            .xy()
            .distance(trunk_transform.translation.xy());
        if distance < TRUNK_PICKUP_RANGE {
            within_trunk_pickup_range = true;
            break;
        }
    }
    if state.within_trunk_pickup_range != within_trunk_pickup_range {
        state.within_trunk_pickup_range = within_trunk_pickup_range;
    }
}

/// When this runs, all entities are spawned and the HTN asset is loaded.
fn setup_troll_htn_supervisor(mut commands: Commands, rolodex: Res<Rolodex>) {
    commands
        .spawn((
            Name::new("Htn Supervisor"),
            HtnSupervisor {
                htn_handle: rolodex.troll_htn.clone(),
            },
            initial_gamestate(),
        ))
        .set_parent(rolodex.troll)
        .trigger(ReplanRequest);
}

// need a child of the troll to act as the HtnOperator parent, that holds the plan and has children that
// contain the operator components.
// then when an operator completes, it sets HtnOperator.set_result(true), and an OnChange system
// can despawn that entity and spawn the next one in the plan.

// TODO sensors that update the world state too..

fn print_htn(assets: Res<Assets<HtnAsset<GameState>>>, rolodex: Res<Rolodex>) {
    let Some(htn_asset) = assets.get(&rolodex.troll_htn) else {
        return;
    };
    info!("HTN: {:#?}", htn_asset.htn);
}
