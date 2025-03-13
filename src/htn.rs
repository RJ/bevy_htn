use bevy::{
    prelude::*,
    reflect::{ReflectMut, TypeRegistry},
};
use std::marker::PhantomData;

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
    /// To execute a primitive task is to insert the operator component into an entity.
    /// The component can have fields with names matching fields from the state, and the
    /// value of those state fields are initialized into the operator component before spawning.
    pub fn insert_operator(
        &self,
        state: &T,
        type_registry: &TypeRegistry,
        entity: &mut EntityWorldMut,
    ) -> Option<bool> {
        let op_type = self.operator.name();
        let registration = type_registry.get_with_short_type_path(op_type)?;
        let reflect_default = registration
            .data::<ReflectDefault>()
            .expect("ReflectDefault should be registered");
        let mut value: Box<dyn Reflect> = reflect_default.default();

        for param in self.operator.params().iter() {
            let Ok(Some(state_val)) = state.reflect_ref().as_struct().map(|s| s.field(param))
            else {
                continue;
            };
            // operator components are either structs or tuple structs
            if let Ok(dyn_struct) = value.reflect_mut().as_struct() {
                dyn_struct.field_mut(param).unwrap().apply(state_val);
            } else if let Ok(dyn_tuple_struct) = value.reflect_mut().as_tuple_struct() {
                dyn_tuple_struct.field_mut(0).unwrap().apply(state_val);
            } else {
                panic!(
                    "Unsupported operator type: {:#?} - should be tuple_struct or struct",
                    value
                );
            }
        }
        info!("Inserting operator: {value:?}");

        let reflect_component = registration
            .data::<ReflectComponent>()
            .expect("ReflectComponent should be registered");

        // components are either structs or tuple structs
        let partial_reflect = if let Ok(s) = value.reflect_mut().as_struct() {
            s.as_partial_reflect()
        } else if let Ok(ts) = value.reflect_mut().as_tuple_struct() {
            ts.as_partial_reflect()
        } else {
            panic!("Value must be either a struct or tuple struct")
        };

        reflect_component.insert(entity, partial_reflect, type_registry);
        Some(true)
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
#[derive(Debug, Reflect)]
pub struct HTN<T: Reflect> {
    pub tasks: Vec<Task<T>>,
}

impl<T: Reflect> HTN<T> {
    pub fn builder() -> HTNBuilder<T> {
        HTNBuilder { tasks: Vec::new() }
    }

    pub fn get_task_by_name(&self, name: &str) -> Option<&Task<T>> {
        self.tasks.iter().find(|task| match task {
            Task::Primitive(primitive) => primitive.name == name,
            Task::Compound(compound) => compound.name == name,
        })
    }

    pub fn root_task(&self) -> &Task<T> {
        self.tasks.first().expect("No root task found")
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
