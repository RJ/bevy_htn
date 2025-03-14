use bevy::prelude::*;
use bevy_htn::prelude::*;

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
}
