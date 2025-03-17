use crate::*;
// use bevy::prelude::*;
// use bevy_behave::prelude::*;

pub fn navigate_to_plugin(app: &mut App) {
    app.register_type::<NavigateToOperator>();
    app.add_systems(Update, navigate_to_system);
}

fn navigate_to_system(
    q_behave: Query<(&BehaveCtx, &NavigateToOperator)>,
    mut q_troll: Query<&mut Transform, With<Troll>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (ctx, navigate_to) in q_behave.iter() {
        let mut troll_transform = q_troll
            .get_mut(ctx.target_entity())
            .expect("Troll not found");
        let dist_threshold = 40.0;
        let dist = troll_transform.translation.xy().distance(navigate_to.0);
        if dist < dist_threshold {
            commands.trigger(ctx.success());
            continue;
        }
        let direction = (navigate_to.0 - troll_transform.translation.xy()).normalize();
        let movement = direction * TROLL_SPEED * time.delta_secs();
        troll_transform.translation.x += movement.x;
        troll_transform.translation.y += movement.y;
    }
}
