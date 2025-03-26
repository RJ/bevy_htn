use crate::{prelude::*, HtnStateTrait};
use bevy::prelude::*;
use bevy_behave::prelude::*;
use std::marker::PhantomData;

pub struct HtnExecutorPlugin<T: HtnStateTrait> {
    phantom: PhantomData<T>,
}

impl<T: HtnStateTrait> Default for HtnExecutorPlugin<T> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T: HtnStateTrait> Plugin for HtnExecutorPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                task_finished,
                when_to_replan_system::<T>,
                check_plans_still_valid::<T>,
            ),
        );
        app.add_observer(on_exec_next_task::<T>);
        // TODO each T instance of pluin will add this as a global obs, not what we want:
        app.add_observer(on_plan_added);
        app.add_observer(on_task_complete::<T>);
        app.add_observer(on_replan_request::<T>);
    }
}

pub trait HtnSupervisorExt {
    fn spawn_htn_supervisor<T: HtnStateTrait>(
        &mut self,
        htn_handle: Handle<HtnAsset<T>>,
        initial_state: &T,
    ) -> Entity;
}
impl HtnSupervisorExt for EntityCommands<'_> {
    fn spawn_htn_supervisor<T: HtnStateTrait>(
        &mut self,
        htn_handle: Handle<HtnAsset<T>>,
        initial_state: &T,
    ) -> Entity {
        let initial_state = initial_state.clone();
        let id = self.id();
        self.commands_mut()
            .spawn((
                Name::new("Htn Supervisor"),
                HtnSupervisor { htn_handle },
                initial_state,
            ))
            .set_parent(id)
            .id()
    }
}

#[derive(Event, Debug)]
pub struct TaskComplete {
    pub task_id: PlannedTaskId,
    pub success: bool,
}

impl TaskComplete {
    pub fn new(task_id: PlannedTaskId, success: bool) -> Self {
        Self { task_id, success }
    }
}

#[derive(Event)]
pub struct ReplanRequest;

/// This entity is the parent of the HTN operator entities.
/// It holds the HTN asset and the current plan, and is a direct child of the troll.
#[derive(Component, Reflect)]
pub struct HtnSupervisor<T: HtnStateTrait> {
    pub htn_handle: Handle<HtnAsset<T>>,
}

// #[derive(Event)]
// pub struct KillRunningTaskChildren;

// fn on_kill_running_task_children<
//     T: Reflect + Component + TypePath + Default + Clone + core::fmt::Debug,
// >(
//     t: Trigger<KillRunningTaskChildren>,
//     q: Query<&Children, With<HtnSupervisor<T>>>,
//     q_children: Query<Entity, With<PlannedTaskId>>,
//     mut commands: Commands,
// ) {
//     if let Ok(children) = q.get(t.entity()) {
//         for child in children.iter().filter(|c| q_children.contains(**c)) {
//             info!("Killing child executing old plan: {child:?}");
//             commands
//                 .entity(t.entity())
//                 .remove_children(&[*child])
//                 .remove::<PlannedTaskId>();
//             commands.entity(*child).despawn_recursive();
//         }
//     }
// }

// do we need to replan? is the current plan still valid?
fn when_to_replan_system<T: HtnStateTrait>(
    mut q: Query<(Entity, &HtnSupervisor<T>, &T, &mut Plan), Or<(Added<T>, Changed<T>)>>,
    mut commands: Commands,
    assets: Res<Assets<HtnAsset<T>>>,
    atr: Res<AppTypeRegistry>,
) {
    for (sup_entity, htn_supervisor, state, mut plan) in q.iter_mut() {
        let Some(htn) = assets.get(&htn_supervisor.htn_handle).map(|h| &h.htn) else {
            warn!("HtnAsset not found");
            return;
        };
        // the game state has changed, is the current plan still valid?
        if !plan.check_validity(htn, state.clone(), atr.as_ref()) {
            plan.abort();
            commands.trigger_targets(ReplanRequest, sup_entity);
            continue;
        }

        // replan on any change:
        commands.trigger_targets(ReplanRequest, sup_entity);
    }
}

fn check_plans_still_valid<T: HtnStateTrait>(
    mut q: Query<(Entity, &HtnSupervisor<T>, &T, &mut Plan)>,
    assets: Res<Assets<HtnAsset<T>>>,
    atr: Res<AppTypeRegistry>,
) {
    for (_sup_entity, htn_supervisor, state, mut plan) in q.iter_mut() {
        let Some(htn) = assets.get(&htn_supervisor.htn_handle).map(|h| &h.htn) else {
            warn!("HtnAsset not found");
            continue;
        };
        if !plan.check_validity(htn, state.clone(), atr.as_ref()) {
            plan.abort();
        }
    }
}

fn on_replan_request<T: HtnStateTrait>(
    t: Trigger<ReplanRequest>,
    assets: Res<Assets<HtnAsset<T>>>,
    q: Query<(&HtnSupervisor<T>, &Parent, &T, Option<&Plan>)>,
    atr: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    // these are triggering on the sup entity that has the Plan, State and HTNSupervisor.
    info!("Replan request event for entity: {:?}", t.entity());

    let Ok((htn_supervisor, _parent, state, opt_plan)) = q.get(t.entity()) else {
        warn!("HtnSupervisor not found");
        return;
    };
    let Some(htn) = assets.get(&htn_supervisor.htn_handle).map(|h| &h.htn) else {
        warn!("HtnAsset not found");
        return;
    };

    let mut planner = HtnPlanner::new(htn, atr.as_ref());
    let new_plan = planner.plan(state);

    if let Some(existing_plan) = opt_plan {
        let existing_plan_active = existing_plan.status().is_none();
        // if existing plan is finished, we'll have to replan anyway.
        if existing_plan_active {
            if *existing_plan == new_plan {
                debug!("🔂 Plan is the same as existing, skipping");
                return;
            }
            // seems ok but plans that are finished need to not exist, because finished high pri
            // plans are trumping new ones atm.
            // need to make overall plan completion work better.
            if *existing_plan > new_plan {
                debug!("Existing plan, which is active, has higher priority, ignoring new plan: {new_plan}");
                // warn!("Ignored New plan: {new_plan}");
                // warn!(
                //     "Existing plan: {existing_plan} status: {:?}",
                //     existing_plan.status()
                // );
                return;
            }
        }
    }
    commands.entity(t.entity()).insert(new_plan);
}

fn on_plan_added(t: Trigger<OnInsert, Plan>, mut commands: Commands, q: Query<&Plan>) {
    // TODO kill any children that are executing an old plan?
    // get the old plan id and kill just those children?
    let plan = q.get(t.entity()).unwrap();
    info!("🗺️ Plan Inserted: {}", plan.task_names().join(", "));
    commands.trigger_targets(ExecNextTask, t.entity());
}

fn on_task_complete<T: HtnStateTrait>(
    t: Trigger<TaskComplete>,
    mut q: Query<(&mut Plan, &HtnSupervisor<T>, &mut T, &Parent)>,
    assets: Res<Assets<HtnAsset<T>>>,
    atr: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    info!("Task complete event: {}", t.event().task_id.name());
    let TaskComplete { task_id, success } = t.event();
    let sup_entity = t.entity();
    let Ok((mut plan, htn_sup, mut state, parent)) = q.get_mut(sup_entity) else {
        error!("HtnSupervisor {sup_entity:?} not found");
        return;
    };
    let character_entity = parent.get();
    if plan.id() != task_id.plan_id() {
        warn!("Task {task_id:?} is from a different plan, ignoring result");
        return;
    }
    let htn = &assets.get(htn_sup.htn_handle.id()).unwrap().htn;
    let task_name = task_id.name();
    let Some(task) = htn.get_task_by_name(task_name) else {
        error!("Task {task_id:?} not found");
        return;
    };
    plan.report_task_completion(task_id, *success);

    if *success {
        info!("Task {task_name} completed successfully -> {character_entity:?}");
        commands.trigger_targets(
            HtnTaskEvent::Success(task_name.to_string()),
            character_entity,
        );
    } else {
        commands.trigger_targets(
            HtnTaskEvent::Failure(task_name.to_string()),
            character_entity,
        );
    }

    if *success {
        match task {
            Task::Primitive(primitive) => {
                // warn!("Applying effects for primitive task: {task_id:?}");
                // bypassing change detection here, any effect of a completed task will already
                // be anticipated by the planner, no need to cause a replan.
                primitive.apply_effects(state.bypass_change_detection(), atr.as_ref());
            }
            Task::Compound(_compound) => {}
        }
    }

    match plan.status() {
        // plan completed successfully, let's replan.
        Some(true) => {
            commands.trigger_targets(ReplanRequest, sup_entity);
        }
        // plan failed
        Some(false) => {
            error!("Plan failed, no more tasks will be executed. Replanning");
            commands.trigger_targets(ReplanRequest, sup_entity);
        }
        // plan still pending a result
        None => {
            commands.trigger_targets(ExecNextTask, sup_entity);
        }
    }
}

fn on_exec_next_task<T: HtnStateTrait>(
    t: Trigger<ExecNextTask>,
    mut q: Query<(Option<&Children>, &Parent, &HtnSupervisor<T>, &mut Plan, &T)>,
    q_children: Query<Entity, With<PlannedTaskId>>,
    assets: Res<Assets<HtnAsset<T>>>,
    type_registry: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    let sup_entity = t.entity();
    let Ok((children, parent, sup, mut plan, state)) = q.get_mut(sup_entity) else {
        error!("HtnSupervisor not found");
        return;
    };
    // kill any children executing a previous plan:
    if let Some(children) = children {
        for child in children.iter().filter(|c| q_children.contains(**c)) {
            debug!("Killing child executing old plan: {child:?}");
            commands
                .entity(t.entity())
                .remove_children(&[*child])
                .remove::<PlannedTaskId>();
            commands.entity(*child).despawn_recursive();
        }
    }
    let Some(task_id) = plan.next_task_to_execute() else {
        info!("No more tasks to execute");
        return;
    };
    let htn = &assets.get(&sup.htn_handle).unwrap().htn;
    let Some(Task::Primitive(task)) = htn.get_task_by_name(task_id.name()) else {
        panic!("Task {task_id:?} is not a primitive on this htn");
    };
    if !task.preconditions_met(state, type_registry.as_ref()) {
        debug!("Task {task_id:?} preconditions not met, failing plan - replanning.");
        plan.abort();
        commands.trigger_targets(ReplanRequest, sup_entity);
        return;
    }

    let task_strategy = task.execution_command(state, &type_registry.read(), &task_id);
    match task_strategy {
        TaskExecutionStrategy::BehaviourTree { tree, task_id } => {
            let task_name = task_id.name().to_string();
            // warn!("Executing operator: {task_name}");
            let character_entity = parent.get();
            commands
                .spawn((
                    task_id,
                    BehaveTree::new(tree),
                    BehaveTargetEntity::Entity(character_entity),
                    BehaveSupervisorEntity(t.entity()),
                ))
                .set_parent(t.entity());
            commands.trigger_targets(HtnTaskEvent::Executing(task_name), character_entity);
        }
    }
}

/// Event triggered on character entity when a task starts or completes.
#[derive(Event, Debug, Clone, Reflect)]
pub enum HtnTaskEvent {
    Executing(String),
    Success(String),
    Failure(String),
}

#[derive(Event)]
struct ExecNextTask;

fn task_finished(
    q: Query<(&BehaveFinished, &PlannedTaskId, &Parent), Added<BehaveFinished>>,
    // mut q_sup: Query<(&mut Plan, &HtnSupervisor<T>)>,
    mut commands: Commands,
) {
    for (finished, task_id, parent) in q.iter() {
        commands.trigger_targets(
            TaskComplete {
                task_id: task_id.clone(),
                success: finished.0,
            },
            parent.get(),
        );
    }
}
