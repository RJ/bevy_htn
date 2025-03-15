/// No built-in ReflectHtnOperator in bevy, unlike ReflectComponent.
///
/// This implementation yields a Command that can be applied to call world.trigger.
use bevy::{prelude::*, reflect::FromType};

use bevy_behave::prelude::*;

use crate::planner::PlannedTaskId;

/// A trait derived for all HTN operator structs that get triggered when executing a task.
pub trait HtnOperator: Reflect + Default + Clone + std::fmt::Debug {
    /// If this returns a Some(tree), the operator will spawn a behaviour tree.
    /// If it returns None, the operator will emit a trigger: `HtnTaskExecute<Operator>`
    fn to_tree(&self) -> Option<Tree<Behave>>;
}

/// A struct used to operate on reflected [`HtnOperator`] of a type.
///
/// A [`ReflectHtnOperator`] for type `T` can be obtained via
/// [`bevy::reflect::TypeRegistration::data`].
#[derive(Clone)]
pub struct ReflectHtnOperator(ReflectHtnOperatorFns);

/// The raw function pointers needed to make up a [`ReflectHtnOperator`].
///
/// This is used when creating custom implementations of [`ReflectHtnOperator`] with
/// [`ReflectHtnOperator::new()`].
#[derive(Clone)]
pub struct ReflectHtnOperatorFns {
    /// Function pointer implementing [`ReflectHtnOperator::trigger()`].
    pub trigger: fn(&dyn Reflect, Entity, PlannedTaskId) -> TriggerEmitterCommand,
    pub to_tree: fn(&dyn Reflect) -> Option<Tree<Behave>>,
}

impl ReflectHtnOperatorFns {
    /// Get the default set of [`ReflectHtnOperatorFns`] for a specific component type using its
    /// [`FromType`] implementation.
    ///
    /// This is useful if you want to start with the default implementation before overriding some
    /// of the functions to create a custom implementation.
    pub fn new<T: HtnOperator + Reflect>() -> Self {
        <ReflectHtnOperator as FromType<T>>::from_type().0
    }
}

impl ReflectHtnOperator {
    /// Creates Command that will emit the trigger
    pub fn trigger(
        &self,
        event: &dyn Reflect,
        entity: Entity,
        task_id: PlannedTaskId,
    ) -> TriggerEmitterCommand {
        (self.0.trigger)(event, entity, task_id)
    }

    pub fn to_tree(&self, event: &dyn Reflect) -> Option<Tree<Behave>> {
        (self.0.to_tree)(event)
    }

    /// Create a custom implementation of [`ReflectHtnOperator`].
    pub fn new(fns: ReflectHtnOperatorFns) -> Self {
        Self(fns)
    }

    /// The underlying function pointers implementing methods on `ReflectHtnOperator`.
    pub fn fn_pointers(&self) -> &ReflectHtnOperatorFns {
        &self.0
    }
}

impl<E: HtnOperator + Reflect> FromType<E> for ReflectHtnOperator {
    fn from_type() -> Self {
        ReflectHtnOperator(ReflectHtnOperatorFns {
            trigger: |op, entity, task_id| -> TriggerEmitterCommand {
                let Some(ev) = op.downcast_ref::<E>() else {
                    panic!("Event is not of type {}", std::any::type_name::<E>());
                };
                let op_event = ev.clone();
                TriggerEmitterCommand {
                    f: Box::new(move |world: &mut World| {
                        let e = HtnTaskExecute {
                            // Fn closure, can't modify captured env, so clone again (they are small)
                            inner: op_event.clone(),
                            task_id,
                        };
                        info!("world.trigger_targets({e:?}, {entity})");
                        world.trigger_targets(e, entity);
                    }),
                }
            },
            to_tree: |op| -> Option<Tree<Behave>> {
                let Some(ev) = op.downcast_ref::<E>() else {
                    panic!("Event is not of type {}", std::any::type_name::<E>());
                };
                ev.to_tree()
            },
        })
    }
}

#[derive(Event, Debug, Clone)]
pub struct HtnTaskExecute<T: Clone + std::fmt::Debug> {
    inner: T,
    task_id: PlannedTaskId,
}

impl<T: Clone + std::fmt::Debug> HtnTaskExecute<T> {
    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn task_id(&self) -> PlannedTaskId {
        self.task_id
    }
}

pub struct TriggerEmitterCommand {
    f: Box<dyn Fn(&mut World) + Send>,
}

impl Command for TriggerEmitterCommand {
    fn apply(self, world: &mut World) {
        (self.f)(world);
    }
}

// pub struct TriggerOperatorCommand<T: Clone + Send + Sync> {
//     event: Box<T>,
//     target_entity: Entity,
// }

// #[derive(Event)]
// pub struct OperatorTrigger<T> {
//     operator: Box<T>,
// }

// pub trait CommandsHtnOperatorExt {
//     fn trigger_htn_operator(&mut self, operator: Box<dyn HtnOperator>, target_entity: Entity);
// }

// impl CommandsHtnOperatorExt for Commands {
//     fn trigger_htn_operator(&mut self, operator: Box<dyn HtnOperator>, target_entity: Entity) {
//         self.queue(move |world: &mut World| {
//             world.trigger_targets(operator.trigger_event(), target_entity);
//         });
//         // let world = self.world_mut();
//         // operator.trigger_command(target_entity, world);
//     }
// }

// look at dynamic trigger from behave for this?

// /// Operators impl this trait, so they can be emitted as triggers.
// #[reflect_trait]
// pub trait HtnOperator {
//     fn trigger_command(self: Box<Self>, target_entity: Entity, world: &mut World) {
//         // let mut ot = OperatorTrigger {
//         //     operator: self.clone(),
//         // };
//         // world.trigger_ref(&mut ot);
//         // commands.trigger;
//         // let c = self.downcast_ref::<Self>();
//     }
//     // {
//     //     OperatorTrigger {
//     //         operator: Box::new(self.clone()),
//     //     }
//     // }
//     // world.trigger_targets(
//     //     OperatorTrigger {
//     //         operator: Box::new(self),
//     //     },
//     //     target_entity,
//     // );
//     // let operator = self.clone();
//     // commands.queue(move |world: &mut World| {
//     // world.trigger_targets(OperatorTrigger { operator }, target_entity);
//     // });
//     // commands.add(TriggerOperatorCommand {
//     //     event: self.clone(),
//     //     target_entity,
//     // });
//     // }
//     // fn operator_trait_fn(&self) {
//     //     info!(
//     //         "Operator trait fn self = {:#?}, type name = {}",
//     //         self,
//     //         std::any::type_name::<Self>()
//     //     );
//     // }
// }
