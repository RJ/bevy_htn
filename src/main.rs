use bevy::prelude::*;

mod dsl;
mod htn;
mod htn_assets;
use dsl::*;
use htn_assets::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct GameState {
    pub location: Vec2,
    pub trunk_health: i32,
    // true if found_trunk_location is set
    pub found_trunk: bool,
    pub found_trunk_location: Vec2,
    pub can_navigate_to_enemy: bool,
    pub attacked_recently: bool,
    pub can_see_enemy: bool,
    pub has_seen_enemy_recently: bool,
    pub last_enemy_location: Vec2,
}

// ---------- Example Usage ----------

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::log::LogPlugin::default());
    app.register_type::<SellGold>();
    app.add_observer(on_add_sellgold);
    app.add_systems(Startup, startup);
    app.add_plugins(htn_assets::HtnAssetPlugin::<GameState>::default());
    app.init_resource::<Htns>();
    app.add_systems(Update, print_htn);
    app.run();
}

#[derive(Resource, Debug, Default)]
struct Htns {
    test: Handle<HtnAsset<GameState>>,
    printed: bool,
}

fn startup(assets: Res<AssetServer>, mut htns: ResMut<Htns>) {
    let test = assets.load("test.htn");
    info!("Loading test.htn via asset server.. {test:?}");
    htns.test = test;
}

fn print_htn(mut htns: ResMut<Htns>, assets: Res<Assets<HtnAsset<GameState>>>) {
    if htns.printed {
        return;
    }

    let Some(htn_asset) = assets.get(&htns.test) else {
        return;
    };

    htns.printed = true;
    info!("HTN: {:#?}", htn_asset.htn);
}

// fn startup(world: &mut World) {
//     // Here we specify that our HTN is for GameState.
//     let htn = parse_htn::<GameState>(dsl);
//     println!("Parsed HTN: {:#?}", htn);

//     // Example execution of top-level tasks (subtask execution omitted for brevity):
//     // let mut state = GameState {
//     //     gold: false,
//     //     energy: 1,
//     // };
//     // println!("Initial state: {:#?}", state);
//     // for task in htn.tasks.iter() {
//     //     if task.preconditions.iter().all(|c| c.evaluate(&state)) {
//     //         info!("Executing task: {}", task.name);
//     //         println!("State: {:#?}", state);
//     //         let mut entity = world.spawn(());
//     //         task.insert_operator(&state, &app_type_registry.read(), &mut entity);
//     //         let eid = entity.id();
//     //         world.commands().entity(eid).log_components();

//     //         for eff in task.effects.iter() {
//     //             eff.apply(&mut state);
//     //         }
//     //     } else {
//     //         println!("Skipping task: {}", task.name);
//     //         println!("State: {:#?}", state);
//     //     }
//     // }
//     // println!("Final state: {:#?}", state);
// }

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Default)]
struct SellGold {
    energy: i32,
}

fn on_add_sellgold(t: Trigger<OnAdd, SellGold>, q: Query<&SellGold>) {
    let Ok(sellgold) = q.get(t.entity()) else {
        return;
    };
    info!("SellGold added: {:#?}", sellgold);
}
