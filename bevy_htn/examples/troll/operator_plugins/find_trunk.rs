use crate::*;

pub fn find_trunk_plugin(app: &mut App) {
    app.register_type::<FindTrunkOperator>();
    app.add_observer(on_find_trunk);
}

fn on_find_trunk(
    t: Trigger<BehaveTrigger<FindTrunkOperator>>,
    q: Query<&Transform, With<Trunk>>,
    mut q_state: Query<&mut GameState>,
    mut commands: Commands,
) {
    let ctx = t.ctx();
    let Some(trunk_transform) = q.iter().next() else {
        error!("FindTrunkOperator can't find a trunk");
        commands.trigger(ctx.failure());
        return;
    };
    let Ok(mut state) = q_state.get_mut(ctx.supervisor_entity().unwrap()) else {
        error!("FindTrunkOperator supervisor not found");
        commands.trigger(ctx.failure());
        return;
    };
    state.found_trunk = true; // set as an effect so not necessary here.
    state.found_trunk_location = trunk_transform.translation.xy(); // needed!
    commands.trigger(ctx.success());
}
