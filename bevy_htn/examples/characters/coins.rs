/// Spawn a coin every few seconds
/// Check for collisions with characters, and trigger a coin collected event
/// Animate coins a bit.
use crate::*;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

const COIN_PATH: &str = "models/animated/coin.glb";
const COIN_SECS: u64 = 10;
const COIN_COLLECT_DIST: f32 = 5.0;

#[derive(Component)]
pub struct Coin;

pub fn coin_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            anim_coins,
            check_coin_collisions,
            update_coins.run_if(on_timer(Duration::from_secs(COIN_SECS))),
        ),
    )
    .add_observer(on_spawn_coin);
}

#[derive(Event)]
pub struct SpawnCoin(f32, f32);

fn update_coins(
    q_coins: Query<Entity, With<Coin>>,
    mut commands: Commands,
    level_config: Res<LevelConfig>,
) {
    // despawn all coins
    q_coins
        .iter()
        .for_each(|e| commands.entity(e).despawn_recursive());
    // maybe spawn one
    if rand::random::<bool>() {
        let (x, z) = level_config.random_position();
        commands.trigger(SpawnCoin(x, z));
    }
}

fn on_spawn_coin(t: Trigger<SpawnCoin>, mut commands: Commands, asset_server: Res<AssetServer>) {
    let x = t.event().0;
    let z = t.event().1;
    commands.spawn((
        Name::new("Coin"),
        Coin,
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(COIN_PATH))),
        Transform::from_translation(Vec3::new(x, 5.0, z)).with_scale(Vec3::ONE * 10.0),
    ));
}

#[derive(Component)]
struct CoinGhost;

#[derive(Event)]
pub struct CoinCollected;

fn check_coin_collisions(
    q_coins: Query<(Entity, &Transform), (With<Coin>, Without<CoinGhost>, Without<Dude>)>,
    q: Query<(Entity, &Transform), (With<Dude>, Without<CoinGhost>, Without<Coin>)>,
    mut q_coin_ghosts: Query<
        (Entity, &mut Transform),
        (With<CoinGhost>, Without<Coin>, Without<Dude>),
    >,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (coin_e, coin_transform) in &q_coins {
        for (player_e, player_transform) in &q {
            if coin_transform
                .translation
                .xz()
                .distance(player_transform.translation.xz())
                < COIN_COLLECT_DIST
            {
                // warn!("Coin collision!");
                commands
                    .entity(coin_e)
                    .remove::<Coin>()
                    .try_insert(CoinGhost);
                commands.trigger_targets(CoinCollected, player_e);
            }
        }
    }
    for (coin_e, mut coin_transform) in q_coin_ghosts.iter_mut() {
        coin_transform.translation.y = coin_transform.translation.y.lerp(300.0, time.delta_secs());
        if coin_transform.translation.y > 25.0 {
            commands.entity(coin_e).try_despawn_recursive();
        }
    }
}
fn anim_coins(mut q: Query<&mut Transform, With<Coin>>, time: Res<Time>) {
    for mut transform in &mut q {
        transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, time.elapsed_secs() * 2.0, 0.0);
        // Add gentle up/down floating motion
        transform.translation.y += (time.elapsed_secs() * 2.0).sin() * 0.02;
    }
}
