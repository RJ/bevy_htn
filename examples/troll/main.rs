use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_htn::prelude::*;

use bevy_inspector_egui::bevy_egui;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay, prelude::*, DefaultInspectorConfigPlugin,
};

mod setup_level;
use setup_level::*;
mod operators;
use operators::*;

#[derive(Reflect, Component, Clone, Debug, Default, InspectorOptions)]
#[reflect(Default, Component, InspectorOptions)]
pub struct GameState {
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
    pub dummy_field: bool,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultInspectorConfigPlugin);
    app.add_plugins(HtnAssetPlugin::<GameState>::default());
    // app.add_plugins(ResourceInspectorPlugin::<GameState>::default());
    app.register_type::<GameState>();
    app.add_plugins(setup_level);

    // app.register_type::<SellGold>();
    // app.add_observer(on_add_sellgold);

    app.add_systems(OnEnter(LoadingState::Ready), setup_troll_htn_supervisor);
    app.add_systems(OnEnter(LoadingState::SpawningEntities), print_htn);
    app.add_systems(Update, replan_checker.run_if(in_state(LoadingState::Ready)));

    app.run();
}

fn initial_gamestate() -> GameState {
    GameState {
        location: Vec2::new(1., 1.),
        trunk_health: 3,
        found_trunk: false,
        found_trunk_location: Vec2::new(2., 2.),
        can_navigate_to_enemy: true,
        attacked_recently: true,
        can_see_enemy: true,
        has_seen_enemy_recently: false,
        last_enemy_location: Vec2::new(666., 666.),
        dummy_field: false,
    }
}

/// This entity is the parent of the HTN operator entities.
/// It holds the HTN asset and the current plan, and is a direct child of the troll.
#[derive(Component)]
struct HtnSupervisor {
    htn_handle: Handle<HtnAsset<GameState>>,
    plan: Option<Plan>,
}

/// When this runs, all entities are spawned and the HTN asset is loaded.
fn setup_troll_htn_supervisor(
    mut commands: Commands,
    // mut assets: ResMut<Assets<HtnAsset<GameState>>>,
    rolodex: Res<Rolodex>,
) {
    // let troll_htn_asset = assets.get(&rolodex.troll_htn).unwrap();
    info!("rolodex: {:#?}", rolodex);
    let troll_htn_supervisor = commands
        .spawn((
            Name::new("Htn Supervisor"),
            HtnSupervisor {
                htn_handle: rolodex.troll_htn.clone(),
                plan: None,
            },
            initial_gamestate(),
        ))
        .id();
    commands
        .entity(rolodex.troll)
        .add_child(troll_htn_supervisor);
}

struct Plan {
    pub tasks: Vec<String>,
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

#[allow(clippy::type_complexity)]
fn replan_checker(
    assets: Res<Assets<HtnAsset<GameState>>>,
    // state: Res<GameState>,
    // rolodex: Res<Rolodex>,
    mut q: Query<
        (&mut HtnSupervisor, &Parent, &GameState),
        Or<(Added<GameState>, Changed<GameState>)>,
    >,
    // mut commands: Commands,
) {
    let Ok((mut htn_supervisor, _parent, state)) = q.get_single_mut() else {
        return;
    };
    let Some(htn_asset) = assets.get(&htn_supervisor.htn_handle) else {
        return;
    };
    let htn = &htn_asset.htn;

    info!("Planning - Initial State:\n{:#?}", state);
    let mut planner = HtnPlanner::new(htn);
    let tasks = planner.plan(state);
    info!("Plan:\n{:#?}\n", tasks);
    htn_supervisor.plan = Some(Plan {
        tasks: tasks.clone(),
    });
}
