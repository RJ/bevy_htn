use std::collections::VecDeque;

use crate::htn::*;
use bevy::prelude::*;

pub struct HtnPlanner<'a, T: Reflect + Default + TypePath + Clone + core::fmt::Debug> {
    htn: &'a HTN<T>,
    task_stack: VecDeque<String>,
    decomp_stack: Vec<DecompositionState>,
    method_index: usize,
    final_plan: Vec<String>,
    elapsed_secs: f32,
}

#[derive(Debug)]
struct DecompositionState {
    current_task: String,
    final_plan: Vec<String>,
    next_method_index: usize,
}

impl<'a, T: Reflect + Default + TypePath + Clone + core::fmt::Debug> HtnPlanner<'a, T> {
    pub fn new(htn: &'a HTN<T>) -> Self {
        Self {
            task_stack: VecDeque::new(),
            htn,
            decomp_stack: Vec::new(),
            final_plan: Vec::new(),
            method_index: 0,
            elapsed_secs: 0.0,
        }
    }

    pub fn plan(&mut self, initial_state: &T) -> &Vec<String> {
        const SANITY_LIMIT: usize = 100;
        let mut sanity_count = 0;

        self.final_plan.clear();
        self.decomp_stack.clear();
        self.task_stack
            .push_back(self.htn.root_task().name().to_string());
        self.method_index = 0;
        let mut state = initial_state.clone();
        // Using vecdeque as a stack, top fo stack (next item) is the FRONT
        while let Some(current_task_name) = self.task_stack.pop_front() {
            sanity_count += 1;
            if sanity_count > SANITY_LIMIT {
                error!("Sanity limit reached, aborting");
                break;
            }
            let Some(task) = self.htn.get_task_by_name(&current_task_name) else {
                error!("Task {current_task_name} not found in HTN");
                self.final_plan.clear();
                break;
            };
            info!(
                "Processing: {current_task_name} Stack: {:?}",
                self.task_stack
            );
            match task {
                Task::Compound(compound) => {
                    // find the first method with passing preconditions
                    if let Some((method, method_index)) =
                        compound.find_method(&state, self.method_index)
                    {
                        info!("Compound task {current_task_name} has valid method {method_index}: {method:?}");
                        // record decomposition
                        let decomposition = DecompositionState {
                            current_task: current_task_name.clone(),
                            final_plan: self.final_plan.clone(),
                            next_method_index: method_index + 1,
                        };
                        self.decomp_stack.push(decomposition);
                        // add subtasks to the stack in reverse order since we're using push_back
                        for subtask in method.subtasks.iter().rev() {
                            self.task_stack.push_front(subtask.clone());
                        }
                        continue;
                    } else {
                        info!("Compound task {current_task_name} has no valid method");
                        // fall through to restore decomp
                    }
                }
                Task::Primitive(primitive) => {
                    if primitive.preconditions_met(&state) {
                        info!("Adding primitive task to plan: {current_task_name}");
                        // add task to final plan
                        self.final_plan.push(current_task_name);
                        // apply this task's effects to the working world state
                        for effect in primitive.effects.iter() {
                            effect.apply(&mut state);
                        }
                        info!("Working state is now: {state:?}");
                        continue;
                    } else {
                        info!("Primitive task preconditions not met: {current_task_name}");
                        // fall through to restore decomp
                    }
                }
            }
            let decomp = self.decomp_stack.pop().unwrap();
            warn!("Restoring decomp {decomp:?}");
            self.final_plan = decomp.final_plan;
            self.method_index = decomp.next_method_index;
            self.task_stack.push_front(decomp.current_task);
        }
        info!("Planning final state: {state:#?}");
        &self.final_plan
    }
}
