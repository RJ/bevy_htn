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
    app.add_plugins(TrollUiPlugin);
    // app.add_plugins(ResourceInspectorPlugin::<GameState>::default());
    app.add_plugins(HtnPlugin::<GameState>::default());
    app.add_plugins(setup_level);
    app.add_plugins(setup_operators_plugin);
    // app.register_type::<SellGold>();
    // app.add_observer(on_add_sellgold);

    app.add_systems(OnEnter(LoadingState::Ready), setup_troll_htn_supervisor);
    app.add_systems(OnEnter(LoadingState::SpawningEntities), print_htn);
    app.add_systems(Update, replan_checker.run_if(in_state(LoadingState::Ready)));
    app.add_systems(Update, troll_enemy_vision_sensor);

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

fn troll_enemy_vision_sensor(
    mut q: Query<&mut GameState>,
    q_troll: Query<&Transform, With<Troll>>,
    q_player: Query<&Transform, With<Player>>,
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
        state.last_enemy_location = player_transform.translation.xy();
    }
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
            },
            initial_gamestate(),
        ))
        .id();
    commands
        .entity(rolodex.troll)
        .add_child(troll_htn_supervisor);
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
    _rolodex: Res<Rolodex>,
    mut q: Query<
        (
            Entity,
            &mut HtnSupervisor<GameState>,
            &Parent,
            &GameState,
            Option<&Plan>,
        ),
        Or<(Added<GameState>, Changed<GameState>)>,
    >,
    mut commands: Commands,
    // type_registry: Res<AppTypeRegistry>,
) {
    let Ok((sup_entity, mut htn_supervisor, _parent, state, opt_plan)) = q.get_single_mut() else {
        return;
    };
    let Some(htn_asset) = assets.get(&htn_supervisor.htn_handle) else {
        return;
    };
    let htn = &htn_asset.htn;

    // let type_registry = type_registry.read();

    info!("Planning - Initial State:\n{:#?}", state);
    let mut planner = HtnPlanner::new(htn);
    let plan = planner.plan(state);

    if let Some(existing_plan) = opt_plan {
        if *existing_plan == plan {
            info!("Plan is the same as existing, skipping");
            return;
        }
    }

    info!("Inserting Plan:\n{:#?}\n", plan);
    commands.entity(sup_entity).insert(plan);

    // htn_supervisor.plan = Some(Plan {
    //     tasks: plan.clone(),
    //     current_task: 0,
    // });

    // let Task::Primitive(task) = htn.get_task_by_name(&plan[0]).unwrap() else {
    //     panic!("Task is not a primitive");
    // };
    // let cmd = task
    //     .execution_command(state, &type_registry, Some(rolodex.troll))
    //     .unwrap();
    // commands.queue(cmd);
}
