/// No built-in ReflectEvent in bevy, unlike ReflectComponent.
///
/// This implementation yields a Command that can be applied to call world.trigger.
use bevy::{prelude::*, reflect::FromType};

/// A struct used to operate on reflected [`Event`] of a type.
///
/// A [`ReflectEvent`] for type `T` can be obtained via
/// [`bevy::reflect::TypeRegistration::data`].
#[derive(Clone)]
pub struct ReflectEvent(ReflectEventFns);

/// The raw function pointers needed to make up a [`ReflectEvent`].
///
/// This is used when creating custom implementations of [`ReflectEvent`] with
/// [`ReflectEvent::new()`].
#[derive(Clone)]
pub struct ReflectEventFns {
    /// Function pointer implementing [`ReflectEvent::trigger()`].
    pub trigger: fn(&dyn Reflect, Option<Entity>) -> TriggerEmitterCommand,
}

impl ReflectEventFns {
    /// Get the default set of [`ReflectEventFns`] for a specific component type using its
    /// [`FromType`] implementation.
    ///
    /// This is useful if you want to start with the default implementation before overriding some
    /// of the functions to create a custom implementation.
    pub fn new<T: Event + Reflect + Clone + std::fmt::Debug>() -> Self {
        <ReflectEvent as FromType<T>>::from_type().0
    }
}

impl ReflectEvent {
    /// Sends reflected [`Event`] to world using [`send()`](ReflectEvent::send).
    pub fn trigger(&self, event: &dyn Reflect, entity: Option<Entity>) -> TriggerEmitterCommand {
        (self.0.trigger)(event, entity)
    }

    /// Create a custom implementation of [`ReflectEvent`].
    pub fn new(fns: ReflectEventFns) -> Self {
        Self(fns)
    }

    /// The underlying function pointers implementing methods on `ReflectEvent`.
    pub fn fn_pointers(&self) -> &ReflectEventFns {
        &self.0
    }
}

impl<E: Event + Reflect + Clone + std::fmt::Debug> FromType<E> for ReflectEvent {
    fn from_type() -> Self {
        ReflectEvent(ReflectEventFns {
            trigger: |event, entity| -> TriggerEmitterCommand {
                let Some(ev) = event.downcast_ref::<E>() else {
                    panic!("Event is not of type {}", std::any::type_name::<E>());
                };
                let trig_event = ev.clone();
                TriggerEmitterCommand {
                    f: Box::new(move |world: &mut World| {
                        if let Some(entity) = entity {
                            info!("world.trigger_targets({trig_event:?}, {entity})");
                            world.trigger_targets(trig_event.clone(), entity);
                        } else {
                            info!("world.trigger({trig_event:?})");
                            world.trigger(trig_event.clone());
                        }
                    }),
                }
            },
        })
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
