use crate::*;

pub fn choose_bridge_plugin(app: &mut App) {
    app.register_type::<ChooseBridgeToCheckOperator>();
    app.add_observer(on_choose_bridge_to_check);
}

fn on_choose_bridge_to_check(
    t: Trigger<BehaveTrigger<ChooseBridgeToCheckOperator>>,
    mut q_sups: Query<(Entity, &mut GameState), With<HtnSupervisor<GameState>>>,
    mut commands: Commands,
) {
    let ctx = t.ctx();
    info!(
        "ChooseBridgeToCheckOperator target: {} tree: {}",
        ctx.target_entity(),
        ctx.behave_entity()
    );
    let Ok((_sup_entity, mut state)) = q_sups.get_mut(ctx.supervisor_entity().unwrap()) else {
        error!("ChooseBridgeToCheckOperator supervisor not found");
        commands.trigger(ctx.failure());
        return;
    };

    // let (mut state, mut _plan) = q
    //     .get_mut(ctx.behave_entity())
    //     .expect("Behave entity not found");
    state.next_bridge_to_check = 1 + (state.next_bridge_to_check + 1) % 3;
    // this needs to exec the next task somehow:
    // maybe trigger a report we get from the trigger to centralize reporting status,
    // so we can trigger the next task?
    //
    // or have this report update aplan internal "next job" thing we can pop off in the replan checker
    commands.trigger(ctx.success());
    // TaskComplete::new(t.event().task_id().clone(), true),
    // t.event().entity(),
    // );
}
