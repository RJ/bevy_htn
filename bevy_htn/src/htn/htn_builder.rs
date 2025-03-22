use crate::HtnStateTrait;

use super::*;
use bevy::prelude::*;

#[derive(Debug, Reflect, Clone, Default)]
pub struct HtnSchema {
    pub version: String,
}

/// This is the HTN domain - a list of all the compound and primitive tasks.
#[derive(Debug, Reflect, Clone)]
pub struct HTN<T: HtnStateTrait> {
    pub tasks: Vec<Task<T>>,
    pub schema: HtnSchema,
}

impl<T: HtnStateTrait> HTN<T> {
    pub fn builder() -> HTNBuilder<T> {
        HTNBuilder {
            tasks: Vec::new(),
            schema: HtnSchema::default(),
        }
    }

    /// Gets version declared in the htn block.
    pub fn version(&self) -> &str {
        &self.schema.version
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

    /// Verifies that every rust type used in the HTN is registered in the type registry, to
    /// avoid any runtime errors executing the HTN.
    ///
    /// Call this after parsing the HTN before trying to use it.
    pub fn verify_all(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        self.verify_conditions(state, atr)?;
        self.verify_effects(state, atr)?;
        self.verify_operators(state, atr)?;
        Ok(())
    }

    /// Verifies that every rust type used in the HTN in reference to the state type is registered.
    /// Doesn't check that operators are registered.
    /// Used in tests that check the planner output without actually running the HTNs.
    pub fn verify_without_operators(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        self.verify_conditions(state, atr)?;
        self.verify_effects(state, atr)?;
        Ok(())
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

    pub fn verify_effects(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        for task in self.tasks.iter() {
            info!("Verifying effects for task: {}", task.name());
            task.verify_effects(state, atr)?;
        }
        Ok(())
    }

    pub fn verify_conditions(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        for task in self.tasks.iter() {
            info!("Verifying conditions for task: {}", task.name());
            task.verify_conditions(state, atr)?;
        }
        Ok(())
    }
}

pub struct HTNBuilder<T: HtnStateTrait> {
    tasks: Vec<Task<T>>,
    schema: HtnSchema,
}

impl<T: HtnStateTrait> HTNBuilder<T> {
    pub fn primitive_task(mut self, task: PrimitiveTask<T>) -> Self {
        self.tasks.push(Task::Primitive(task));
        self
    }

    pub fn compound_task(mut self, task: CompoundTask<T>) -> Self {
        self.tasks.push(Task::Compound(task));
        self
    }

    pub fn schema(mut self, meta: HtnSchema) -> Self {
        self.schema = meta;
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
            schema: self.schema,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub enum Task<T: HtnStateTrait> {
    Primitive(PrimitiveTask<T>),
    Compound(CompoundTask<T>),
}

impl<T: HtnStateTrait> Task<T> {
    pub fn name(&self) -> &str {
        match self {
            Task::Primitive(primitive) => &primitive.name,
            Task::Compound(compound) => &compound.name,
        }
    }
    pub fn verify_effects(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        match self {
            Task::Primitive(primitive) => primitive.verify_effects(state, atr),
            // compound tasks don't have effects, only primitive tasks do.
            Task::Compound(_compound) => Ok(()),
        }
    }
    pub fn verify_conditions(&self, state: &T, atr: &AppTypeRegistry) -> Result<(), String> {
        match self {
            Task::Primitive(primitive) => primitive.verify_conditions(state, atr),
            Task::Compound(compound) => compound.verify_conditions(state, atr),
        }
    }
}
