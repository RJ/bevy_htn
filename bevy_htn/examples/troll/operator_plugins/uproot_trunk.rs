use crate::*;

pub fn uproot_trunk_plugin(app: &mut App) {
    app.register_type::<UprootTrunkOperator>();
    app.add_observer(on_uproot_trunk);
}

fn on_uproot_trunk(
    t: Trigger<BehaveTrigger<UprootTrunkOperator>>,
    q_trunk: Query<(Entity, &Transform), With<Trunk>>,
    q_troll: Query<&Transform, With<crate::setup_level::Troll>>,
    // time: Res<Time>,
    mut commands: Commands,
) {
    let ctx = t.ctx();
    let Ok(troll_transform) = q_troll.get(ctx.target_entity()) else {
        error!("Troll not found");
        return;
    };
    // ensure we're next to a trunk, then despawn it and succeed so we pick it up.
    for (entity, transform) in q_trunk.iter() {
        if transform
            .translation
            .xy()
            .distance(troll_transform.translation.xy())
            < 50.0
        {
            info!("Found a trunk to uproot");
            commands.entity(entity).despawn_recursive();
            commands.trigger(ctx.success());
            return;
        }
    }
    error!("No trunk found to uproot");
    commands.trigger(ctx.failure());
}
