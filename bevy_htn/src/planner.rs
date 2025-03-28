use crate::{htn::*, HtnStateTrait};
use bevy::prelude::*;
use rand::Rng;
use std::collections::VecDeque;

#[derive(Reflect, Debug, Component)]
pub struct Plan {
    plan_id: u32,
    next_task_index: usize,
    pub tasks: Vec<PlannedTask>,
    mtr: Vec<usize>,
    status: Option<bool>,
}

impl Plan {
    pub fn new(tasks: Vec<String>, mtr: Vec<usize>) -> Self {
        let plan_id = rand::rng().random::<u32>();
        let tasks = tasks
            .iter()
            .enumerate()
            .map(|(idx, name)| PlannedTask {
                name: name.clone(),
                status: TaskStatus::NotStarted,
                id: PlannedTaskId::new(plan_id, idx, name.clone()),
            })
            .collect();
        Self {
            plan_id,
            next_task_index: 0,
            tasks,
            mtr,
            status: None,
        }
    }
    // pub fn preconditions_met<T: HtnStateTrait>(&self, state: &T, atr: &AppTypeRegistry) -> bool {
    //     // let mut state = state.clone();
    //     // for planned_task in self.tasks.iter() {
    //     //     if !task.preconditions_met(&state, atr) {
    //     //         info!("Preconditions not met for task: {task:?}");
    //     //         return false;
    //     //     }
    //     // }
    //     // TODO
    //     true
    // }
    pub fn id(&self) -> u32 {
        self.plan_id
    }
    pub fn mtr(&self) -> &[usize] {
        &self.mtr
    }
    /// None = pending, Some(true) = success, Some(false) = failure
    pub fn status(&self) -> Option<bool> {
        self.status
    }

    pub fn abort(&mut self) {
        self.status = Some(false);
    }

    pub fn task_names(&self) -> Vec<String> {
        self.tasks.iter().map(|t| t.name.clone()).collect()
    }

    /// Iterates over primitive tasks in plan, checking working_state preconditions met, then
    /// applying effects and checking next task, etc.
    pub fn check_validity<T: HtnStateTrait>(
        &self,
        htn: &HTN<T>,
        mut working_state: T,
        atr: &AppTypeRegistry,
    ) -> bool {
        for task_name in self.tasks.iter() {
            let task = htn.get_task_by_name(task_name.name.as_str()).unwrap();
            if let Task::Primitive(task) = task {
                if !task.preconditions_met(&working_state, atr) {
                    info!(
                        "Plan invalidated, preconditions not met: {} `{}`",
                        task_name.name,
                        task.find_first_failing_precondition(&working_state, atr)
                            .map(|c| c.syntax())
                            .unwrap_or("???".to_string())
                    );
                    return false;
                }
                task.apply_effects(&mut working_state, atr);
                task.apply_expected_effects(&mut working_state, atr);
            } else {
                panic!("Non primitive task in plan, should not happen");
            }
        }
        true
    }

    /// Marks next task as running and returns the planned task id
    pub fn next_task_to_execute(&mut self) -> Option<PlannedTaskId> {
        if self.status.is_some() {
            warn!("Plan already has a status, cannot execute next task.");
            return None;
        }
        if self.next_task_index >= self.tasks.len() {
            info!("Plan complete, no next task.");
            return None;
        }
        let task = &mut self.tasks[self.next_task_index];
        task.status = TaskStatus::Running;
        self.next_task_index += 1;
        Some(task.id.clone())
    }

    pub fn report_task_completion(&mut self, task_id: &PlannedTaskId, success: bool) {
        if self.status.is_some() {
            warn!("Plan already has a status, cannot report task completion.");
            return;
        }
        if let Some((idx, task)) = self
            .tasks
            .iter_mut()
            .enumerate()
            .find(|(_idx, t)| t.id == *task_id)
        {
            info!(
                "Report task completion: {task_id:?} {} = {success}",
                task.name
            );
            if success {
                task.status = TaskStatus::Success;
            } else {
                task.status = TaskStatus::Failure;
                warn!("Task {task:?} failed, plan failed.");
                self.status = Some(false);
                return;
            }
            self.next_task_index = idx + 1;
        } else {
            error!("Task {task_id:?} not found in plan?");
        }
        if self.next_task_index >= self.tasks.len() {
            info!("Plan completed!");
            self.status = Some(true);
        }
    }
}

impl PartialOrd for Plan {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Plan {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare MTRs element by element
        for (a, b) in self.mtr.iter().zip(other.mtr.iter()) {
            match a.cmp(b) {
                std::cmp::Ordering::Equal => continue,
                ordering => return ordering.reverse(), // Reverse since lower values take priority
            }
        }
        // If one MTR is shorter but matches the other so far, shorter one has priority
        self.mtr.len().cmp(&other.mtr.len()).reverse()
    }
}

impl std::fmt::Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Plan MTR: [{}] Tasks: [{}]",
            self.mtr
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.tasks
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl PartialEq for Plan {
    fn eq(&self, other: &Self) -> bool {
        if self.tasks.len() != other.tasks.len() {
            return false;
        }
        self.tasks
            .iter()
            .zip(other.tasks.iter())
            .all(|(a, b)| a.name == b.name)
    }
}

impl Eq for Plan {}

/// A unique id of a task in a plan, comprised of the plan id and the index of the task in the plan.
#[derive(Reflect, Clone, Debug, Component, PartialEq)]
pub struct PlannedTaskId {
    plan_id: u32,
    index: usize,
    name: String,
}

impl PlannedTaskId {
    pub fn new(plan_id: u32, index: usize, name: String) -> Self {
        Self {
            plan_id,
            index,
            name,
        }
    }
    pub fn plan_id(&self) -> u32 {
        self.plan_id
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Reflect, Clone, Debug)]
pub struct PlannedTask {
    pub id: PlannedTaskId,
    pub name: String,
    pub status: TaskStatus,
}

#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum TaskStatus {
    NotStarted,
    Running,
    Success,
    Failure,
}

#[derive(Debug)]
struct DecompositionState {
    current_task: String,
    final_plan: Vec<String>,
    skip_methods: usize,
    mtr: Vec<usize>,
}

pub struct HtnPlanner<'a, T: HtnStateTrait> {
    htn: &'a HTN<T>,
    task_stack: VecDeque<String>,
    decomp_stack: Vec<DecompositionState>,
    skip_methods: usize,
    atr: &'a AppTypeRegistry,
    mtr: Vec<usize>,
}

impl<'a, T: HtnStateTrait> HtnPlanner<'a, T> {
    pub fn new(htn: &'a HTN<T>, atr: &'a AppTypeRegistry) -> Self {
        Self {
            task_stack: VecDeque::new(),
            htn,
            decomp_stack: Vec::new(),
            skip_methods: 0,
            atr,
            mtr: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.decomp_stack.clear();
        self.task_stack.clear();
        self.skip_methods = 0;
        self.mtr.clear();
    }

    pub fn plan(&mut self, initial_state: &T) -> Plan {
        const SANITY_LIMIT: usize = 100;
        let mut sanity_count = 0;
        self.reset();
        let mut final_plan = Vec::new();
        self.task_stack
            .push_back(self.htn.root_task().name().to_string());
        let mut state = initial_state.clone();
        // debug!("PLAN initial state: {state:?}");
        // Using vecdeque as a stack, top of stack (next item) is the FRONT
        while let Some(current_task_name) = self.task_stack.pop_front() {
            sanity_count += 1;
            if sanity_count > SANITY_LIMIT {
                // in case of logic errors during dev..
                error!("Sanity limit reached, aborting");
                break;
            }
            let Some(task) = self.htn.get_task_by_name(&current_task_name) else {
                error!("Task {current_task_name} not found in HTN");
                final_plan.clear();
                break;
            };

            // if let Some(top_task) = self.decomp_stack.last().map(|d| d.current_task.clone()) {
            //     if top_task == current_task_name {
            //         info!("Resetting skip_methods to 0 for {current_task_name}");
            //         self.skip_methods = 0;
            //     }
            // }

            // info!(
            //     "Processing: {current_task_name} Stack: {:?}",
            //     self.task_stack
            // );
            debug!(
                "EVALUATING {current_task_name} with self.skip_methods = {}",
                self.skip_methods
            );
            debug!(" planner state: {state:?}");
            debug!(" decomp stack len: {:?}", self.decomp_stack.len());
            match task {
                Task::Compound(compound) => {
                    // find the first method with passing preconditions

                    // for (method_index, method) in
                    //     compound.methods.iter().enumerate().skip(self.skip_methods)
                    // {
                    //     let valid = method
                    //         .preconditions
                    //         .iter()
                    //         .all(|cond| cond.evaluate(&state, self.atr));
                    //     if !valid {
                    //         info!(
                    //             "🟥 {current_task_name} method: {method_index} - {} (skipped {}) PRECONDITIONS NOT MET",
                    //             method
                    //                 .name
                    //                 .clone()
                    //                 .unwrap_or_else(|| format!("#{method_index}")),
                    //             self.skip_methods,
                    //         );
                    //         continue;
                    //     }
                    // }

                    if let Some((method, method_index)) =
                        compound.find_method(&state, self.skip_methods, self.atr)
                    {
                        debug!(
                            "🟨 {current_task_name} -> {} (using index: {method_index}, skipped {})",
                            method
                                .name
                                .clone()
                                .unwrap_or_else(|| format!("#{method_index}")),
                            self.skip_methods,
                        );
                        self.mtr.push(method_index);
                        // record decomposition
                        let decomposition = DecompositionState {
                            current_task: current_task_name.clone(),
                            final_plan: final_plan.clone(),
                            // method index 0 based, skip is number to skip:
                            skip_methods: method_index + 1,
                            mtr: self.mtr.clone(),
                        };
                        debug!("📚 Adding {decomposition:?}");
                        self.decomp_stack.push(decomposition);
                        // add subtasks to the stack, preserving order
                        for subtask in method.subtasks.iter().rev() {
                            self.task_stack.push_front(subtask.clone());
                        }
                        debug!("🟡 Adding decomposed tasks to plan: {:?}", method.subtasks);
                        // do we need to reset the skip_methods when recursively calling ourself?

                        // info!("💫 Resetting skip_methods to 0 for {current_task_name}");
                        self.skip_methods = 0;
                        // how? TODO
                        continue;
                    } else {
                        debug!(
                            "🟥 Compound task {current_task_name} has no valid method, skip: {} \nstate was {state:?}",
                            self.skip_methods
                        );
                        // info!("Current state: {state:?}");
                        // fall through to restore decomp
                    }
                }
                Task::Primitive(primitive) => {
                    if primitive.preconditions_met(&state, self.atr) {
                        debug!(
                            "🟢 Adding primitive task to plan: {current_task_name} -> [{}]",
                            final_plan.join(", ")
                        );
                        // add task to final plan
                        final_plan.push(current_task_name);
                        // apply this task's effects to the planner state
                        for effect in primitive.effects.iter() {
                            effect.apply(&mut state, self.atr);
                        }
                        for effect in primitive.expected_effects.iter() {
                            effect.apply(&mut state, self.atr);
                        }
                        // debug!("Working state is now: {state:?}");
                        continue;
                    } else {
                        debug!("🔴 Primitive task preconditions not met: {current_task_name}\nstate was: {state:?}");
                        // info!("Current state: {state:?}");
                        // fall through to restore decomp
                    }
                }
            }
            if let Some(decomp) = self.decomp_stack.pop() {
                debug!("Restoring decomp {decomp:?}");
                final_plan = decomp.final_plan;
                self.skip_methods = decomp.skip_methods;
                self.task_stack.push_front(decomp.current_task);
                self.mtr = decomp.mtr;
            } else {
                debug!("No decomp, plan failed?");
            }
        }
        debug!("Planning final state: {state:#?}");
        info!("final plan: {final_plan:?} mtr: {:?}", self.mtr);
        Plan::new(final_plan, self.mtr.clone())
    }
}
