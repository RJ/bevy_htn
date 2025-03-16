use super::*;
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Clone, Debug, Reflect)]
pub struct Method<T: Reflect> {
    pub preconditions: Vec<HtnCondition>,
    pub subtasks: Vec<String>, // Just the task names now
    _phantom: PhantomData<T>,
}

#[derive(Clone, Debug, Reflect)]
pub struct CompoundTask<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> {
    pub name: String,
    pub methods: Vec<Method<T>>,
    _phantom: PhantomData<T>,
}

impl<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> CompoundTask<T> {
    /// Finds the first method with passing preconditions, skipping the first `skip` methods.
    pub fn find_method(
        &self,
        state: &T,
        skip: usize,
        mirror: &Mirror,
    ) -> Option<(&Method<T>, usize)> {
        self.methods
            .iter()
            .enumerate()
            .skip(skip)
            .find(|(_, method)| {
                method
                    .preconditions
                    .iter()
                    .all(|cond| cond.evaluate(state, mirror))
            })
            .map(|(i, method)| (method, i))
    }
}

pub struct CompoundTaskBuilder<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> {
    name: String,
    methods: Vec<Method<T>>,
    _phantom: PhantomData<T>,
}

impl<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> CompoundTaskBuilder<T> {
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

// Add this for building methods
pub struct MethodBuilder<T: Reflect> {
    preconditions: Vec<HtnCondition>,
    subtasks: Vec<String>, // Just task names, not the actual tasks
    _phantom: PhantomData<T>,
}

impl<T: Reflect> MethodBuilder<T> {
    #[allow(clippy::new_without_default)]
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
