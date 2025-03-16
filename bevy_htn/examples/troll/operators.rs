use bevy::prelude::*;
use bevy_behave::prelude::*;
use bevy_htn::prelude::*;

use crate::GameState;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct DoTrunkSlamOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct UprootTrunkOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct FindTrunkOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct NavigateToTrunkOperator(Vec2);

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct NavigateToOperator(Vec2);

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct RegainLOSOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct ChooseBridgeToCheckOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct CheckBridgeOperator;

pub fn setup_operators_plugin(app: &mut App) {
    app.register_type::<DoTrunkSlamOperator>();
    app.register_type::<UprootTrunkOperator>();
    app.register_type::<FindTrunkOperator>();
    app.register_type::<NavigateToTrunkOperator>();
    app.register_type::<NavigateToOperator>();
    app.register_type::<RegainLOSOperator>();
    app.register_type::<ChooseBridgeToCheckOperator>();
    app.register_type::<CheckBridgeOperator>();

    app.add_observer(on_choose_bridge_to_check);
}

fn on_choose_bridge_to_check(
    t: Trigger<HtnTaskExecute<ChooseBridgeToCheckOperator>>,
    mut q: Query<(&mut GameState, &mut Plan)>,
    mut commands: Commands,
) {
    info!("ChooseBridgeToCheckOperator {}", t.entity());
    let (mut state, mut _plan) = q.get_mut(t.entity()).unwrap();
    state.next_bridge_to_check = 1 + (state.next_bridge_to_check + 1) % 3;
    // this needs to exec the next task somehow:
    // maybe trigger a report we get from the trigger to centralize reporting status,
    // so we can trigger the next task?
    //
    // or have this report update aplan internal "next job" thing we can pop off in the replan checker
    commands.trigger_targets(
        TaskComplete::new(t.event().task_id().clone(), true),
        t.event().entity(),
    );
}
