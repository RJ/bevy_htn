use bevy::prelude::*;
use bevy_htn::prelude::*;
#[derive(Event, Debug, Reflect, Default, Clone)]
#[reflect(Default, Event)]
pub struct DoTrunkSlamOperator;

#[derive(Event, Debug, Reflect, Default, Clone)]
#[reflect(Default, Event)]
pub struct UprootTrunkOperator;

#[derive(Event, Debug, Reflect, Default, Clone)]
#[reflect(Default, Event)]
pub struct FindTrunkOperator;

#[derive(Event, Debug, Reflect, Default, Clone)]
#[reflect(Default, Event)]
pub struct NavigateToTrunkOperator(Vec2);

#[derive(Event, Debug, Reflect, Default, Clone)]
#[reflect(Default, Event)]
pub struct NavigateToOperator(Vec2);

#[derive(Event, Debug, Reflect, Default, Clone)]
#[reflect(Default, Event)]
pub struct RegainLOSOperator;

#[derive(Event, Debug, Reflect, Default, Clone)]
#[reflect(Default, Event)]
pub struct ChooseBridgeToCheckOperator;

#[derive(Event, Debug, Reflect, Default, Clone)]
#[reflect(Default, Event)]
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
