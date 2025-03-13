use bevy::prelude::*;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Default)]
pub struct DoTrunkSlamOperator;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Default)]
pub struct UprootTrunkOperator;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Default)]
pub struct FindTrunkOperator;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Default)]
pub struct NavigateToTrunkOperator(Vec2);

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Default)]
pub struct NavigateToOperator(Vec2);

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Default)]
pub struct RegainLOSOperator;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Default)]
pub struct ChooseBridgeToCheckOperator;

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component, Default)]
pub struct CheckBridgeOperator;
