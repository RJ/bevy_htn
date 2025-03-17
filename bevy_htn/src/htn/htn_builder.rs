use super::*;
use bevy::prelude::*;

/// This is the HTN domain - a list of all the compound and primitive tasks.
#[derive(Debug, Reflect, Clone)]
pub struct HTN<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> {
    pub tasks: Vec<Task<T>>,
    pub version: String,
}

impl<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> HTN<T> {
    pub fn builder() -> HTNBuilder<T> {
        HTNBuilder {
            tasks: Vec::new(),
            version: "".to_string(),
        }
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
    pub fn verify_operators(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        for task in self.tasks.iter() {
            match task {
                Task::Primitive(primitive) => primitive.verify_operator(state, atr)?,
                Task::Compound(_) => continue,
            }
        }
        Ok(())
    }
}

pub struct HTNBuilder<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> {
    tasks: Vec<Task<T>>,
    version: String,
}

impl<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> HTNBuilder<T> {
    pub fn primitive_task(mut self, task: PrimitiveTask<T>) -> Self {
        self.tasks.push(Task::Primitive(task));
        self
    }

    pub fn compound_task(mut self, task: CompoundTask<T>) -> Self {
        self.tasks.push(Task::Compound(task));
        self
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    /// Verifies that every operator has the correct type registry entries and that any fields used
    /// by operators are also present in the state.
    pub fn verify_operators(self, state: &T, atr: &AppTypeRegistry) -> Result<Self, String> {
        for task in self.tasks.iter() {
            match task {
                Task::Primitive(primitive) => primitive.verify_operator(state, atr)?,
                Task::Compound(_) => continue,
            }
        }
        Ok(self)
    }

    pub fn build(self) -> HTN<T> {
        HTN {
            tasks: self.tasks,
            version: self.version,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub enum Task<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> {
    Primitive(PrimitiveTask<T>),
    Compound(CompoundTask<T>),
}

impl<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> Task<T> {
    pub fn name(&self) -> &str {
        match self {
            Task::Primitive(primitive) => &primitive.name,
            Task::Compound(compound) => &compound.name,
        }
    }
}
