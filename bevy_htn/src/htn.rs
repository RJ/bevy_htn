use crate::prelude::{ReflectHtnOperator, TriggerEmitterCommand};
use bevy::{prelude::*, reflect::TypeRegistry};
use bevy_behave::prelude::*;
use std::marker::PhantomData;

// use crate::prelude::{HtnOperator, ReflectHtnOperator};

#[derive(Clone, Debug, Reflect)]
pub enum HtnCondition {
    EqualsBool { field: String, value: bool },
    GreaterThanInt { field: String, threshold: i32 },
}

impl HtnCondition {
    pub fn evaluate<T: Reflect>(&self, state: &T) -> bool {
        let reflected = state
            .reflect_ref()
            .as_struct()
            .expect("State is not a struct");
        match self {
            HtnCondition::EqualsBool { field, value } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(b) = val.try_downcast_ref::<bool>() {
                        *b == *value
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HtnCondition::GreaterThanInt { field, threshold } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(i) = val.try_downcast_ref::<i32>() {
                        *i > *threshold
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub enum Effect {
    SetBool { field: String, value: bool },
    SetInt { field: String, value: i32 },
    SetIdentifier { field: String, value: String },
    IncrementInt { field: String, by: i32 },
}

impl Effect {
    pub fn apply<T: Reflect>(&self, state: &mut T) {
        let reflected = state
            .reflect_mut()
            .as_struct()
            .expect("State is not a struct");
        match self {
            Effect::SetBool { field, value } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(b) = val.try_downcast_mut::<bool>() {
                        *b = *value;
                    }
                }
            }
            Effect::SetInt { field, value } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(i) = val.try_downcast_mut::<i32>() {
                        *i = *value;
                    }
                }
            }
            Effect::IncrementInt { field, by } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(i) = val.try_downcast_mut::<i32>() {
                        *i += *by;
                    }
                }
            }
            Effect::SetIdentifier { field, value } => {
                let newval = reflected.field(value).unwrap().clone_value();
                let val = reflected.field_mut(field).unwrap();
                val.apply(newval.as_ref());
            }
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

#[derive(Clone, Debug, Reflect)]
pub struct Method<T: Reflect> {
    pub preconditions: Vec<HtnCondition>,
    pub subtasks: Vec<String>, // Just the task names now
    _phantom: PhantomData<T>,
}

#[derive(Clone, Debug, Reflect)]
pub struct CompoundTask<T: Reflect> {
    pub name: String,
    pub methods: Vec<Method<T>>,
    _phantom: PhantomData<T>,
}

impl<T: Reflect> CompoundTask<T> {
    /// Finds the first method with passing preconditions, skipping the first `skip` methods.
    pub fn find_method(&self, state: &T, skip: usize) -> Option<(&Method<T>, usize)> {
        self.methods
            .iter()
            .enumerate()
            .skip(skip)
            .find(|(_, method)| method.preconditions.iter().all(|cond| cond.evaluate(state)))
            .map(|(i, method)| (method, i))
    }
}

impl<T: Reflect> PrimitiveTask<T> {
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
        entity: Option<Entity>,
    ) -> Option<(TriggerEmitterCommand, Tree<Behave>)> {
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

        let tree = reflect_op.to_tree(boxed_reflect.as_reflect());

        let command = reflect_op.trigger(boxed_reflect.as_reflect(), entity);
        Some((command, tree))
    }

    /// Checks that every operator has the correct type registry entries and that any fields used
    /// by operators are also present in the state.
    pub fn verify_operator(&self, state: &T, type_registry: &TypeRegistry) -> Result<(), String> {
        let op_type = self.operator.name();
        let Some(registration) = type_registry.get_with_short_type_path(op_type) else {
            return Err(format!("No type registry entry for operator '{op_type}'"));
        };
        if registration.data::<ReflectDefault>().is_none() {
            return Err(format!(
                "ReflectDefault should be registered, did you forget to add #[reflect(Default)] to {op_type}?"
            ));
        }
        if registration.data::<ReflectHtnOperator>().is_none() {
            return Err(format!(
                "ReflectHtnOperator should be registered, did you forget to add #[reflect(HtnOperator)] to {op_type}?"
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
    pub fn preconditions_met(&self, state: &T) -> bool {
        self.preconditions.iter().all(|cond| cond.evaluate(state))
    }
}

#[derive(Clone, Debug, Reflect)]
pub enum Operator {
    Spawn { name: String, params: Vec<String> },
    Trigger { name: String, params: Vec<String> },
}

impl Operator {
    pub fn name(&self) -> &str {
        match self {
            Operator::Spawn { name, .. } => name,
            Operator::Trigger { name, .. } => name,
        }
    }
    pub fn params(&self) -> &[String] {
        match self {
            Operator::Spawn { params, .. } => params,
            Operator::Trigger { params, .. } => params,
        }
    }
}

/// This is the HTN domain - a list of all the compound and primitive tasks.
#[derive(Debug, Reflect, Clone)]
pub struct HTN<T: Reflect> {
    pub tasks: Vec<Task<T>>,
}

impl<T: Reflect> HTN<T> {
    pub fn builder() -> HTNBuilder<T> {
        HTNBuilder { tasks: Vec::new() }
    }

    /// Returns the task with the given name.
    pub fn get_task_by_name(&self, name: &str) -> Option<&Task<T>> {
        self.tasks.iter().find(|task| match task {
            Task::Primitive(primitive) => primitive.name == name,
            Task::Compound(compound) => compound.name == name,
        })
    }

    /// Returns the first (compound) task in the HTN.
    pub fn root_task(&self) -> &Task<T> {
        self.tasks.first().expect("No root task found")
    }

    /// Verifies that every operator has the correct type registry entries and that any fields used
    /// by operators are also present in the state.
    pub fn verify_operators(&self, state: &T, type_registry: &TypeRegistry) -> Result<(), String> {
        for task in self.tasks.iter() {
            match task {
                Task::Primitive(primitive) => primitive.verify_operator(state, type_registry)?,
                Task::Compound(_) => continue,
            }
        }
        Ok(())
    }
}

pub struct HTNBuilder<T: Reflect> {
    tasks: Vec<Task<T>>,
}

impl<T: Reflect> HTNBuilder<T> {
    pub fn primitive_task(mut self, task: PrimitiveTask<T>) -> Self {
        self.tasks.push(Task::Primitive(task));
        self
    }

    pub fn compound_task(mut self, task: CompoundTask<T>) -> Self {
        self.tasks.push(Task::Compound(task));
        self
    }

    /// Verifies that every operator has the correct type registry entries and that any fields used
    /// by operators are also present in the state.
    pub fn verify_operators(self, state: &T, type_registry: &TypeRegistry) -> Result<Self, String> {
        for task in self.tasks.iter() {
            match task {
                Task::Primitive(primitive) => primitive.verify_operator(state, type_registry)?,
                Task::Compound(_) => continue,
            }
        }
        Ok(self)
    }

    pub fn build(self) -> HTN<T> {
        HTN { tasks: self.tasks }
    }
}

// Add this for building methods
pub struct MethodBuilder<T: Reflect> {
    preconditions: Vec<HtnCondition>,
    subtasks: Vec<String>, // Just task names, not the actual tasks
    _phantom: PhantomData<T>,
}

impl<T: Reflect> MethodBuilder<T> {
    pub fn new() -> Self {
        MethodBuilder {
            preconditions: Vec::new(),
            subtasks: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn precondition(mut self, cond: HtnCondition) -> Self {
        self.preconditions.push(cond);
        self
    }

    pub fn subtask(mut self, task_name: impl Into<String>) -> Self {
        self.subtasks.push(task_name.into());
        self
    }

    pub fn build(self) -> Method<T> {
        Method {
            preconditions: self.preconditions,
            subtasks: self.subtasks,
            _phantom: PhantomData,
        }
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

pub struct CompoundTaskBuilder<T: Reflect> {
    name: String,
    methods: Vec<Method<T>>,
    _phantom: PhantomData<T>,
}

impl<T: Reflect> CompoundTaskBuilder<T> {
    pub fn new(name: impl Into<String>) -> Self {
        CompoundTaskBuilder {
            name: name.into(),
            methods: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn method(mut self, method: Method<T>) -> Self {
        self.methods.push(method);
        self
    }

    pub fn build(self) -> CompoundTask<T> {
        CompoundTask {
            name: self.name,
            methods: self.methods,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub enum Task<T: Reflect> {
    Primitive(PrimitiveTask<T>),
    Compound(CompoundTask<T>),
}

impl<T: Reflect> Task<T> {
    pub fn name(&self) -> &str {
        match self {
            Task::Primitive(primitive) => &primitive.name,
            Task::Compound(compound) => &compound.name,
        }
    }
}
