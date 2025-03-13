use crate::htn::*;
use bevy::prelude::*;

pub struct HtnPlanner<T: Reflect + Default + TypePath + Clone> {
    pub initial_state: T,
    pub htn: HTN<T>,
    pub tasks_to_process: Vec<String>,
}

struct DecompositionState {
    current_task: String,
    final_plan: Vec<String>,
}

impl<T: Reflect + Default + TypePath + Clone> HtnPlanner<T> {
    pub fn new(initial_state: T, htn: HTN<T>) -> Self {
        Self {
            initial_state,
            tasks_to_process: vec![htn.root_task().name().to_string()],
            htn,
        }
    }

    pub fn plan(&self) -> Vec<String> {
        let mut state = self.initial_state.clone();
        for current_task in self.tasks_to_process.iter() {
            let task = self.htn.get_task_by_name(current_task).unwrap();
            match task {
                Task::Compound(compound) => {
                    if let Some(method) = compound.find_method(&state) {
                        // record decomposition
                        let decomposition = DecompositionState {
                            current_task: current_task.clone(),
                            final_plan: Vec::new(),
                        };
                        for i in 0..method.subtasks.len() {
                            self.tasks_to_process.insert(0, method.subtasks[i].clone());
                        }
                    }
                }
                Task::Primitive(primitive) => {
                    primitive.execute(&mut state);
                }
            }
        }
        Vec::new()
    }
}
