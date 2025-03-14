use std::time::Duration;

use crate::GameState;
use bevy::{
    color::palettes::css, input::common_conditions::input_toggle_active, prelude::*,
    render::mesh::AnnulusMeshBuilder, time::common_conditions::on_timer,
};
use bevy_htn::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum LoadingState {
    #[default]
    AwaitingAssetLoading,
    SpawningEntities,
    Ready,
}

#[derive(Resource, Debug)]
pub struct Rolodex {
    pub troll_htn: Handle<HtnAsset<GameState>>,
    pub bridge_positions: Vec<Vec2>,
    pub troll: Entity,
    pub player: Entity,
}

fn create_rolodex(assets: Res<AssetServer>, mut commands: Commands) {
    let test = assets.load("test.htn");
    info!("Loading test.htn via asset server.. {test:?}");
    commands.insert_resource(Rolodex {
        troll_htn: test,
        bridge_positions: vec![],
        troll: Entity::PLACEHOLDER,
        player: Entity::PLACEHOLDER,
    });
}

fn check_asset_loaded(
    mut ev_asset: EventReader<AssetEvent<HtnAsset<GameState>>>,
    mut next_state: ResMut<NextState<LoadingState>>,
) {
    for ev in ev_asset.read() {
        if let AssetEvent::LoadedWithDependencies { .. } = ev {
            info!("HTN asset loaded: ");
            next_state.set(LoadingState::SpawningEntities);
            break;
        }
    }
}

pub fn setup_level(app: &mut App) {
    app.insert_state(LoadingState::AwaitingAssetLoading);
    app.add_plugins(
        // Show inspector with F12
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F12)),
    );
    // this triggers the asset loading
    app.add_systems(Startup, create_rolodex);
    // this checks if the htn asset is loaded and advances the state
    app.add_systems(
        Update,
        check_asset_loaded
            .run_if(on_event::<AssetEvent<HtnAsset<GameState>>>)
            .run_if(in_state(LoadingState::AwaitingAssetLoading)),
    );
    app.add_systems(
        OnEnter(LoadingState::SpawningEntities),
        (
            // (
            create_rolodex,
            setup_bridges,
            setup_troll,
            setup_player,
            // ),
            |mut next_state: ResMut<NextState<LoadingState>>| {
                info!("Assets loaded, entities spawned: Ready.");
                next_state.set(LoadingState::Ready);
            },
        )
            .chain(),
    );
    app.add_systems(
        Update,
        spawn_trunk
            .run_if(in_state(LoadingState::Ready))
            .run_if(on_timer(Duration::from_secs(1))),
    );
    app.add_systems(
        Update,
        player_movement.run_if(in_state(LoadingState::Ready)),
    );
}

const PLAYER_SPEED: f32 = 200.0;

// move the player using arrow keys
fn player_movement(
    mut players: Query<&mut Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    players.single_mut().translation += Vec3::new(
        (keys.pressed(KeyCode::ArrowRight) as i32 - keys.pressed(KeyCode::ArrowLeft) as i32) as f32,
        (keys.pressed(KeyCode::ArrowUp) as i32 - keys.pressed(KeyCode::ArrowDown) as i32) as f32,
        0.,
    )
    .normalize_or_zero()
        * PLAYER_SPEED
        * time.delta_secs();
}

pub const BRIDGE_WIDTH: f32 = 200.0;
pub const BRIDGE_HEIGHT: f32 = 40.0;
pub const BRIDGE_SPACING: f32 = 70.0;
pub const BRIDGE_DIST: f32 = 120.0;
pub const TROLL_VISION_RADIUS: f32 = 300.0;

#[derive(Component, Debug)]
pub struct Bridge;

#[derive(Component, Debug)]
pub struct Troll;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Trunk;

fn setup_bridges(
    mut commands: Commands,
    mut rolodex: ResMut<Rolodex>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bridge_mesh = Mesh2d(meshes.add(Rectangle::new(BRIDGE_WIDTH, BRIDGE_HEIGHT)));
    let bridge_col: Color = css::GRAY.into();
    let bridge_material = MeshMaterial2d(materials.add(bridge_col));

    let bridge1 = Vec2::new(0.0, BRIDGE_SPACING + BRIDGE_HEIGHT);
    let bridge2 = Vec2::new(0.0, 0.0);
    let bridge3 = Vec2::new(0.0, -BRIDGE_SPACING - BRIDGE_HEIGHT);

    let z = -3.0;
    commands.spawn((
        Transform::from_xyz(bridge1.x, bridge1.y, z),
        bridge_mesh.clone(),
        bridge_material.clone(),
        Bridge,
        Name::new("Bridge1"),
        Text2d::new("Bridge 1"),
    ));
    commands.spawn((
        Transform::from_xyz(bridge2.x, bridge2.y, z),
        bridge_mesh.clone(),
        bridge_material.clone(),
        Bridge,
        Name::new("Bridge2"),
        Text2d::new("Bridge 2"),
    ));
    commands.spawn((
        Transform::from_xyz(bridge3.x, bridge3.y, z),
        bridge_mesh.clone(),
        bridge_material.clone(),
        Bridge,
        Name::new("Bridge3"),
        Text2d::new("Bridge 3"),
    ));
    rolodex.bridge_positions = vec![bridge1, bridge2, bridge3];
}

fn setup_troll(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rolodex: ResMut<Rolodex>,
) {
    let troll_mesh = Mesh2d(meshes.add(RegularPolygon::new(40.0, 6)));
    let troll_col: Color = css::RED.into();
    let troll_material = MeshMaterial2d(materials.add(troll_col));

    rolodex.troll = commands
        .spawn((
            Transform::from_xyz(BRIDGE_WIDTH + BRIDGE_DIST, 0.0, -3.0),
            Text2d::new("Troll"),
            Troll,
            Name::new("Troll"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0.0, 0.0, -0.1),
                troll_mesh.clone(),
                troll_material.clone(),
            ));
            parent.spawn((
                Transform::from_xyz(0.0, 0.0, -0.1),
                Mesh2d(
                    meshes.add(
                        AnnulusMeshBuilder::new(
                            TROLL_VISION_RADIUS - 5.0,
                            TROLL_VISION_RADIUS,
                            128,
                        )
                        .build(),
                    ),
                ),
                MeshMaterial2d(materials.add(Color::WHITE.with_alpha(0.1))),
            ));
        })
        .id();
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rolodex: ResMut<Rolodex>,
) {
    let player_mesh = Mesh2d(meshes.add(Circle::new(30.0)));
    let player_col: Color = css::GREEN.into();
    let player_material = MeshMaterial2d(materials.add(player_col));

    rolodex.player = commands
        .spawn((
            Transform::from_xyz(-BRIDGE_WIDTH / 2.0 - BRIDGE_DIST, 0.0, -2.0),
            Text2d::new("Player"),
            Player,
            Name::new("Player"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0.0, 0.0, -0.1),
                player_mesh.clone(),
                player_material.clone(),
            ));
        })
        .id();
}

fn spawn_trunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q: Query<Entity, With<Trunk>>,
    time: Res<Time>,
) {
    if !q.is_empty() {
        return;
    }

    let trunk_mesh = Mesh2d(meshes.add(Capsule2d::new(15.0, 50.0)));
    let trunk_col: Color = css::BROWN.into();
    let trunk_material = MeshMaterial2d(materials.add(trunk_col));

    // always spawn on right side of screen

    let possible_spawn_locations = (0..6)
        .map(|i| BRIDGE_SPACING * i as f32 - BRIDGE_SPACING * 3.0)
        .map(|y| Vec2::new(BRIDGE_WIDTH + BRIDGE_DIST * 2.5, y))
        .collect::<Vec<_>>();

    let spawn_location =
        possible_spawn_locations[time.elapsed_secs() as usize % possible_spawn_locations.len()];

    commands
        .spawn((
            Transform::from_translation(spawn_location.extend(-4.0)),
            Trunk,
            Name::new("Trunk"),
            Text2d::new("Trunk"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
                trunk_mesh.clone(),
                trunk_material.clone(),
            ));
        });
}
