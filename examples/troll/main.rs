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

#[derive(Reflect, Resource, Clone, Debug, Default, InspectorOptions)]
#[reflect(Default, Resource, InspectorOptions)]
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
    app.add_plugins(ResourceInspectorPlugin::<GameState>::default());
    app.register_type::<GameState>();
    let state = GameState {
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
    };
    app.insert_resource(state);
    app.add_plugins(setup_level);

    // app.register_type::<SellGold>();
    // app.add_observer(on_add_sellgold);

    app.add_systems(Update, plan.run_if(resource_changed::<GameState>));

    app.run();
}

struct Plan {
    pub tasks: Vec<String>,
}

// need a child of the troll to act as the HtnOperator parent, that holds the plan and has children that
// contain the operator components.
// then when an operator completes, it sets HtnOperator.set_result(true), and an OnChange system
// can despawn that entity and spawn the next one in the plan.

// TODO sensors that update the world state too..

fn plan(
    mut htns: ResMut<Htns>,
    assets: Res<Assets<HtnAsset<GameState>>>,
    state: Res<GameState>,
    rolodex: Res<Rolodex>,
    mut commands: Commands,
) {
    let Some(htn_asset) = assets.get(&htns.troll_htn) else {
        return;
    };
    if !htns.printed {
        info!("HTN: {:#?}", htn_asset.htn);
        htns.printed = true;
    }
    info!("Planning - Initial State:\n{:#?}", state);
    let mut planner = HtnPlanner::new(&htn_asset.htn);
    let plan = planner.plan(state.as_ref());
    info!("Plan:\n{:#?}\n", plan);
}
