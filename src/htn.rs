use bevy::{prelude::*, reflect::TypeRegistry};
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
        }
    }
}

// Make Task generic over any state type T that implements Reflect.
#[derive(Clone, Debug, Reflect)]
pub struct Task<T: Reflect> {
    pub name: String,
    pub operator_type: Option<String>,
    /// param names from state to copy into operator struct
    pub operator_params: Vec<String>,
    pub preconditions: Vec<HtnCondition>,
    pub effects: Vec<Effect>,
    pub subtasks: Vec<Task<T>>,
    _phantom: PhantomData<T>,
}

impl<T: Reflect> Task<T> {
    pub fn builder(name: impl Into<String>) -> TaskBuilder<T> {
        TaskBuilder {
            name: name.into(),
            operator_type: None,
            operator_params: Vec::new(),
            preconditions: Vec::new(),
            effects: Vec::new(),
            subtasks: Vec::new(),
        }
    }

    /// To execute a primitive task is to insert the operator component into an entity.
    /// The component can have fields with names matching fields from the state, and the
    /// value of those state fields are initialized into the operator component before spawning.
    pub fn insert_operator(
        &self,
        state: &T,
        type_registry: &TypeRegistry,
        entity: &mut EntityWorldMut,
    ) -> Option<bool> {
        let op_type = self.operator_type.as_ref()?;
        let registration = type_registry.get_with_short_type_path(op_type)?;
        let reflect_default = registration
            .data::<ReflectDefault>()
            .expect("ReflectDefault should be registered");
        let mut value: Box<dyn Reflect> = reflect_default.default();

        // not supporting tuple structs for now
        // (if just one param, and can't find by named field, assume tuple struct?)
        for param in self.operator_params.iter() {
            let Ok(Some(state_val)) = state.reflect_ref().as_struct().map(|s| s.field(param))
            else {
                continue;
            };
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

        let partial_reflect = value
            .reflect_mut()
            .as_struct()
            .unwrap()
            .as_partial_reflect();

        reflect_component.insert(entity, partial_reflect, type_registry);
        Some(true)
    }
}

pub struct TaskBuilder<T: Reflect> {
    name: String,
    operator_type: Option<String>,
    operator_params: Vec<String>,
    preconditions: Vec<HtnCondition>,
    effects: Vec<Effect>,
    subtasks: Vec<Task<T>>,
}

impl<T: Reflect> TaskBuilder<T> {
    pub fn operator(mut self, op_type: impl Into<String>, params: Vec<String>) -> Self {
        self.operator_type = Some(op_type.into());
        self.operator_params = params;
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
    pub fn subtask(mut self, task: Task<T>) -> Self {
        self.subtasks.push(task);
        self
    }
    pub fn build(self) -> Task<T> {
        Task {
            name: self.name,
            operator_type: self.operator_type,
            operator_params: self.operator_params,
            preconditions: self.preconditions,
            effects: self.effects,
            subtasks: self.subtasks,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct HTN<T: Reflect> {
    pub tasks: Vec<Task<T>>,
}

impl<T: Reflect> HTN<T> {
    pub fn builder() -> HTNBuilder<T> {
        HTNBuilder { tasks: Vec::new() }
    }
}

pub struct HTNBuilder<T: Reflect> {
    tasks: Vec<Task<T>>,
}

impl<T: Reflect> HTNBuilder<T> {
    pub fn task(mut self, task: Task<T>) -> Self {
        self.tasks.push(task);
        self
    }
    pub fn build(self) -> HTN<T> {
        HTN { tasks: self.tasks }
    }
}
