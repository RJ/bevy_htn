use bevy::prelude::*;
use bevy_htn::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub enum Location {
    #[default]
    House,
    Outside,
    Mushroom,
    Ore,
    Smelter,
    Merchant,
}

/// This is our marker components, so we can keep track of the various in-game entities
#[derive(Component)]
struct Miner;

#[derive(Component)]
struct House;

#[derive(Component)]
struct Smelter;

#[derive(Component)]
struct Mushroom;

#[derive(Component)]
struct Ore;

#[derive(Component)]
struct Merchant;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct EatOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct SleepOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct MineOreOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct SmeltOreOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct SellMetalOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct GoToOutsideOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct GoToHouseOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct GoToMushroomOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct GoToOreOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct GoToSmelterOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct GoToMerchantOperator;

#[derive(Reflect, Component, Clone, Debug, Default)]
#[reflect(Default, Component)]
pub struct GameState {
    pub hunger: i32,
    pub energy: i32,
    pub gold: i32,
    pub location: Location,
    pub has_ore: bool,
    pub has_metal: bool,
}

#[derive(Resource, Debug)]
pub struct Rolodex {
    pub htn: Handle<HtnAsset<GameState>>,
}
fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::log::LogPlugin::default());
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(HtnAssetPlugin::<GameState>::default());
    app.add_plugins(HtnPlugin::<GameState>::default());
    app.add_systems(Startup, load_htn);
    app.add_systems(Update, (check_asset_loaded));
    app.add_observer(on_htn_loaded);
    app.run();
}

fn load_htn(assets: Res<AssetServer>, mut commands: Commands) {
    let htn = assets.load("miner.htn");
    info!("Loading miner.htn via asset server.. {htn:?}");
    commands.insert_resource(Rolodex { htn });
}

#[derive(Event)]
struct HtnLoaded;

fn check_asset_loaded(
    mut ev_asset: EventReader<AssetEvent<HtnAsset<GameState>>>,
    assets: Res<Assets<HtnAsset<GameState>>>,
    mut commands: Commands,
) {
    for ev in ev_asset.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            info!("HTN asset loaded!");
            let htn = &assets.get(*id).unwrap().htn;
            commands.trigger(HtnLoaded);
            info!("{htn:#?}");
            break;
        }
    }
}

fn on_htn_loaded(
    _t: Trigger<HtnLoaded>,
    rolodex: Res<Rolodex>,
    assets: Res<Assets<HtnAsset<GameState>>>,
    atr: Res<AppTypeRegistry>,
    mut exit: EventWriter<AppExit>,
) {
    let htn = &assets.get(rolodex.htn.id()).unwrap().htn;
    let state = GameState {
        hunger: 0,
        energy: 100,
        gold: 0,
        location: Location::Outside,
        has_ore: false,
        has_metal: false,
    };
    let mut planner = HtnPlanner::new(htn, atr.as_ref());
    let plan = planner.plan(&state);
    info!("Plan found, contains {} tasks.", plan.tasks.len());
    info!(
        "Tasks: {:?}",
        plan.tasks
            .into_iter()
            .map(|pt| pt.name)
            .collect::<Vec<_>>()
            .join(", ")
    );
    exit.send(AppExit::Success);
}
