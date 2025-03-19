use super::*;
use crate::reflect_operator::*;
use crate::PlannedTaskId;
use bevy::{prelude::*, reflect::TypeRegistry};
use bevy_behave::prelude::*;
use std::marker::PhantomData;

pub enum TaskExecutionStrategy {
    BehaviourTree {
        tree: Tree<Behave>,
        task_id: PlannedTaskId,
    },
}

#[derive(Clone, Debug, Reflect)]
pub enum Operator {
    Trigger { name: String, params: Vec<String> },
}

impl Operator {
    pub fn name(&self) -> &str {
        match self {
            Operator::Trigger { name, .. } => name,
        }
    }
    pub fn params(&self) -> &[String] {
        match self {
            Operator::Trigger { params, .. } => params,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct PrimitiveTask<T: Reflect> {
    pub name: String,
    pub operator: Operator,
    pub preconditions: Vec<HtnCondition>,
    pub effects: Vec<Effect>,
    pub expected_effects: Vec<Effect>,
    _phantom: PhantomData<T>,
}

impl<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> PrimitiveTask<T> {
    /// To execute a primitive task is to either:
    /// - insert the operator component into an entity
    /// - trigger an event using the operator struct
    ///
    /// The operator struct can have fields with names matching fields from the state, and the
    /// value of those state fields are initialized into the operator component before spawning.
    ///
    /// This returns a struct that "impl Command" and applying it will emit a trigger event.
    pub fn execution_command(
        &self,
        state: &T,
        type_registry: &TypeRegistry,
        task_id: &PlannedTaskId,
    ) -> TaskExecutionStrategy {
        let op_type = self.operator.name();
        let Some(registration) = type_registry.get_with_short_type_path(op_type) else {
            error!("No type registry entry for operator '{op_type}', be sure you've called app.register_type::<{op_type}>()");
            panic!("Missing type registry entry for operator");
        };
        let Some(reflect_default) = registration.data::<ReflectDefault>() else {
            error!("ReflectDefault should be registered");
            panic!("Missing ReflectDefault for operator");
        };
        let mut boxed_reflect: Box<dyn Reflect> = reflect_default.default();

        for param in self.operator.params().iter() {
            let Ok(Some(state_val_for_param)) =
                state.reflect_ref().as_struct().map(|s| s.field(param))
            else {
                continue;
            };
            // operator components are either structs or tuple structs
            if let Ok(dyn_struct) = boxed_reflect.reflect_mut().as_struct() {
                if let Some(pr_field) = dyn_struct.field_mut(param) {
                    pr_field.apply(state_val_for_param);
                } else {
                    error!("No field found for param: {param}, operator: {op_type}");
                }
            } else if let Ok(dyn_tuple_struct) = boxed_reflect.reflect_mut().as_tuple_struct() {
                if let Some(pr_field) = dyn_tuple_struct.field_mut(0) {
                    pr_field.apply(state_val_for_param);
                } else {
                    error!("No field found for param: {param}, operator: {op_type}");
                }
            } else {
                panic!(
                    "Unsupported operator type: {:#?} - should be tuple_struct or struct",
                    boxed_reflect
                );
            }
        }

        let reflect_op = registration
            .data::<ReflectHtnOperator>()
            .expect("`ReflectHtnOperator` should be registered");

        let tree = reflect_op
            .to_tree(boxed_reflect.as_reflect())
            .expect("Must return a tree?");

        TaskExecutionStrategy::BehaviourTree {
            tree,
            task_id: task_id.clone(),
        }
    }

    pub fn apply_effects(&self, state: &mut T, atr: &AppTypeRegistry) {
        for effect in self.effects.iter() {
            info!("APPLY: {effect:?}");
            effect.apply(state, atr);
        }
    }

    pub fn apply_expected_effects(&self, state: &mut T, atr: &AppTypeRegistry) {
        for effect in self.expected_effects.iter() {
            info!("APPLY(expected): {effect:?}");
            effect.apply(state, atr);
        }
    }

    /// Checks any field names used in effects, expected_effects, are present in the state.
    pub fn verify_effects(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        for effect in self.effects.iter() {
            effect.verify_types(state, atr, false)?;
        }
        for effect in self.expected_effects.iter() {
            effect.verify_types(state, atr, true)?;
        }
        Ok(())
    }

    pub fn verify_conditions(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        for cond in self.preconditions.iter() {
            cond.verify_types(state, atr)?;
        }
        Ok(())
    }

    /// Checks that every operator has the correct type registry entries and that any fields used
    /// by operators are also present in the state.
    pub fn verify_operator(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        let op_type = self.operator.name();
        let Some(registration) = atr.get_type_by_name(op_type) else {
            return Err(format!("No type registry entry for operator '{op_type}'"));
        };
        if registration.data::<ReflectDefault>().is_none() {
            return Err(format!(
                "ReflectDefault should be registered, did you forget to add #[reflect(Default)] to {op_type}?"
            ));
        }
        if registration.data::<ReflectHtnOperator>().is_none() {
            return Err(format!("Operator '{op_type}' is missing Reflection data. Did you forget to derive/implement, AND add #[reflect(HtnOperator)] to {op_type}?"
            ));
        }
        let s = state
            .reflect_ref()
            .as_struct()
            .expect("State should be a reflectable struct");
        let state_type = std::any::type_name::<T>();
        for param in self.operator.params().iter() {
            if s.field(param).is_none() {
                return Err(format!(
                    "State type `{state_type}` does not have field `{param}`, which is used in the `{op_type}` operator"
                ));
            }
        }
        Ok(())
    }

    /// Returns true if all preconditions are met.
    pub fn preconditions_met(&self, state: &T, atr: &AppTypeRegistry) -> bool {
        self.preconditions
            .iter()
            .all(|cond| cond.evaluate(state, atr))
    }
}

// Create specific builders for each task type
pub struct PrimitiveTaskBuilder<T: Reflect> {
    name: String,
    operator: Option<Operator>,
    preconditions: Vec<HtnCondition>,
    effects: Vec<Effect>,
    expected_effects: Vec<Effect>,
    _phantom: PhantomData<T>,
}

impl<T: Reflect> PrimitiveTaskBuilder<T> {
    pub fn new(name: impl Into<String>) -> Self {
        PrimitiveTaskBuilder {
            name: name.into(),
            operator: None,
            preconditions: Vec::new(),
            effects: Vec::new(),
            expected_effects: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn operator(mut self, op: Operator) -> Self {
        self.operator = Some(op);
        self
    }

    pub fn precondition(mut self, cond: HtnCondition) -> Self {
        self.preconditions.push(cond);
        self
    }

    pub fn effect(mut self, eff: Effect) -> Self {
        self.effects.push(eff);
        self
    }

    pub fn expected_effect(mut self, eff: Effect) -> Self {
        self.expected_effects.push(eff);
        self
    }

    pub fn build(self) -> PrimitiveTask<T> {
        PrimitiveTask {
            name: self.name,
            operator: self
                .operator
                .expect("Operator is required for primitive tasks"),
            preconditions: self.preconditions,
            effects: self.effects,
            expected_effects: self.expected_effects,
            _phantom: PhantomData,
        }
    }
}
