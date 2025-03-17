/*
    Operators, which are referenced in the HTN domain file, are defined here.

    An operator's job is to implement HtnOperator which provides its behaviour tree.

    To execute an operator as part of a plan is to run its behaviour tree.


*/
use bevy::prelude::*;
use bevy_behave::prelude::*;
use bevy_htn::prelude::*;

/// TODO: attribute to provide the component which supports From<ThisOperator> ?
#[derive(Debug, Reflect, Default, Clone, Component, HtnOperator)]
#[reflect(Default, HtnOperator)]
#[spawn_named = "Trunk Slam"]
pub struct DoTrunkSlamOperator {
    // keep track of animation duration
    pub start: f32,
}

// if you remove the HtnOperator Derive (and spawn_named) you can manually provide a tree:
// impl HtnOperator for DoTrunkSlamOperator {
//     fn to_tree(&self) -> Option<Tree<Behave>> {
//         Some(behave! {
//             Behave::Sequence => {
//                 Behave::spawn_named("Trunk Slam", TrunkSlamOperator::default()),
//                 Behave::Wait(3.0),
//             }
//         })
//     }
// }

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct UprootTrunkOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct FindTrunkOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct NavigateToTrunkOperator(Vec2);

/// This creates a behave!{ Behave::spawn_named("Navigate To", self) }
/// from this operator, eg inserts as a component for the behaviour tree.
#[derive(Debug, Reflect, Default, Clone, HtnOperator, Component)]
#[reflect(Default, HtnOperator)]
#[spawn_named = "Navigate To"]
pub struct NavigateToOperator(pub Vec2);

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct RegainLOSOperator;

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct ChooseBridgeToCheckOperator;

#[derive(Debug, Reflect, Default, Clone, Component, HtnOperator)]
#[reflect(Default, HtnOperator)]
#[spawn_named = "Check Bridge"]
pub struct CheckBridgeOperator {
    pub bridge_position: Vec2,
    pub start: f32,
}
