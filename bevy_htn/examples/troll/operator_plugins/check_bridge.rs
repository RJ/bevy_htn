use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_behave::prelude::*;

pub fn check_bridge_plugin(app: &mut App) {
    app.register_type::<CheckBridgeOperator>();
    app.add_systems(Update, check_bridge_system);
}

use crate::*;

/// Never completes, just moves around waiting to see a player.
fn check_bridge_system(
    mut q_behave: Query<(&BehaveCtx, &mut CheckBridgeOperator)>,
    mut q_troll: Query<&mut Transform, With<Troll>>,
    time: Res<Time>,
) {
    for (ctx, mut check_bridge) in q_behave.iter_mut() {
        let mut troll_transform = q_troll
            .get_mut(ctx.target_entity())
            .expect("Troll not found");
        // troll has to be at the bridge before it can check it, so yoink the current position
        if check_bridge.bridge_position == Vec2::ZERO {
            check_bridge.bridge_position = troll_transform.translation.xy();
            check_bridge.start = time.elapsed_secs();
        }

        let radius = 25.0;
        let offset = (time.elapsed_secs() - check_bridge.start) % TAU;
        let x = check_bridge.bridge_position.x + radius * offset.sin();
        let y = check_bridge.bridge_position.y + radius * offset.cos();
        troll_transform.translation.x = x;
        troll_transform.translation.y = y;
    }
}
