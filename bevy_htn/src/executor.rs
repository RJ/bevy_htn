use crate::prelude::*;
use bevy::prelude::*;
use bevy_behave::prelude::*;
use std::marker::PhantomData;

pub struct HtnExecutorPlugin<T: Reflect + Component + TypePath> {
    phantom: PhantomData<T>,
}

impl<T: Reflect + Component + TypePath + Default + Clone + core::fmt::Debug> Default
    for HtnExecutorPlugin<T>
{
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T: Reflect + Component + TypePath + Default + Clone + core::fmt::Debug> Plugin
    for HtnExecutorPlugin<T>
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (task_finished, when_to_replan_system::<T>));
        app.add_observer(on_exec_next_task::<T>);
        app.add_observer(on_plan_added);
        app.add_observer(on_task_complete::<T>);
        app.add_observer(on_replan_request::<T>);
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
pub struct HtnSupervisor<T: Reflect + TypePath + Default + Clone + core::fmt::Debug> {
    pub htn_handle: Handle<HtnAsset<T>>,
}

#[derive(Event)]
pub struct KillRunningTaskChildren;

fn on_kill_running_task_children<
    T: Reflect + Component + TypePath + Default + Clone + core::fmt::Debug,
>(
    t: Trigger<KillRunningTaskChildren>,
    q: Query<&Children, With<HtnSupervisor<T>>>,
    q_children: Query<Entity, With<PlannedTaskId>>,
    mut commands: Commands,
) {
    if let Ok(children) = q.get(t.entity()) {
        for child in children.iter().filter(|c| q_children.contains(**c)) {
            info!("Killing child executing old plan: {child:?}");
            commands
                .entity(t.entity())
                .remove_children(&[*child])
                .remove::<PlannedTaskId>();
            commands.entity(*child).despawn_recursive();
        }
    }
}

// do we need to replan? is the current plan still valid?
fn when_to_replan_system<T: Reflect + Component + TypePath + Default + Clone + core::fmt::Debug>(
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

        let mut existing_plan_still_valid = true;
        for task_name in plan.tasks.iter() {
            let task = htn.get_task_by_name(task_name.name.as_str()).unwrap();
            if let Task::Primitive(task) = task {
                if !task.preconditions_met(state, atr.as_ref()) {
                    info!(
                        "Aborting current plan, preconditions not met: {}",
                        task_name.name
                    );
                    existing_plan_still_valid = false;
                    break;
                }
            } else {
                panic!("Non primitive task in plan, should not happen");
            }
        }

        if !existing_plan_still_valid {
            info!("🚫 Plan is no longer valid, aborting and replanning.");
            plan.abort();

            commands.trigger_targets(ReplanRequest, sup_entity);
            continue;
        }

        // replan on any change:
        commands.trigger_targets(ReplanRequest, sup_entity);
    }
}

fn on_replan_request<T: Reflect + Component + TypePath + Default + Clone + core::fmt::Debug>(
    t: Trigger<ReplanRequest>,
    assets: Res<Assets<HtnAsset<T>>>,
    q: Query<(&HtnSupervisor<T>, &Parent, &T, Option<&Plan>)>,
    atr: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    // these are triggering on the sup entity that has the Plan, State and HTNSupervisor.
    info!("Replan request event");

    let Ok((htn_supervisor, _parent, state, opt_plan)) = q.get(t.entity()) else {
        warn!("HtnSupervisor not found");
        return;
    };
    let Some(htn) = assets.get(&htn_supervisor.htn_handle).map(|h| &h.htn) else {
        warn!("HtnAsset not found");
        return;
    };

    let mut planner = HtnPlanner::new(htn, atr.as_ref());
    let plan = planner.plan(state);
    let existing_plan_active = plan.status().is_none();
    if existing_plan_active {
        if let Some(existing_plan) = opt_plan {
            if *existing_plan == plan {
                info!("🔂 Plan is the same as existing, skipping");
                return;
            }
            // seems ok but plans that are finished need to not exist, because finished high pri
            // plans are trumping new ones atm.
            // need to make overall plan completion work better.
            if *existing_plan > plan {
                warn!("Existing plan has higher priority, ignoring new plan.");
                warn!("Ignored New plan: {plan}");
                warn!(
                    "Existing plan: {existing_plan} status: {:?}",
                    existing_plan.status()
                );
                return;
            }
        }
    }
    info!("🗺️ Inserting New Plan: {plan}");
    commands.entity(t.entity()).insert(plan);
}

fn on_plan_added(t: Trigger<OnInsert, Plan>, mut commands: Commands) {
    // TODO kill any children that are executing an old plan?
    // get the old plan id and kill just those children?
    commands.trigger_targets(ExecNextTask, t.entity());
}

fn on_task_complete<T: Reflect + Component + TypePath + Default + Clone + core::fmt::Debug>(
    t: Trigger<TaskComplete>,
    mut q: Query<(&mut Plan, &HtnSupervisor<T>, &mut T)>,
    assets: Res<Assets<HtnAsset<T>>>,
    atr: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    info!("Task complete event: {t:?}");
    let TaskComplete { task_id, success } = t.event();
    let sup_entity = t.entity();
    let Ok((mut plan, htn_sup, mut state)) = q.get_mut(sup_entity) else {
        error!("HtnSupervisor {sup_entity:?} not found");
        return;
    };
    if plan.id() != task_id.plan_id() {
        info!("Task {task_id:?} is from a different plan, ignoring result");
        return;
    }
    let htn = &assets.get(htn_sup.htn_handle.id()).unwrap().htn;
    let Some(task) = htn.get_task_by_name(task_id.name()) else {
        error!("Task {task_id:?} not found");
        return;
    };
    plan.report_task_completion(task_id, *success);
    if *success {
        match task {
            Task::Primitive(primitive) => {
                warn!("Applying effects for primitive task: {task_id:?}");
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

fn on_exec_next_task<T: Reflect + Component + TypePath + Default + Clone + core::fmt::Debug>(
    t: Trigger<ExecNextTask>,
    mut q: Query<(Option<&Children>, &Parent, &HtnSupervisor<T>, &mut Plan, &T)>,
    q_children: Query<Entity, With<PlannedTaskId>>,
    assets: Res<Assets<HtnAsset<T>>>,
    type_registry: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    let Ok((children, parent, sup, mut plan, state)) = q.get_mut(t.entity()) else {
        error!("HtnSupervisor not found");
        return;
    };
    // kill any children executing a previous plan:
    if let Some(children) = children {
        for child in children.iter().filter(|c| q_children.contains(**c)) {
            info!("Killing child executing old plan: {child:?}");
            commands
                .entity(t.entity())
                .remove_children(&[*child])
                .remove::<PlannedTaskId>();
            commands.entity(*child).despawn_recursive();
        }
    }
    let Some(task_id) = plan.execute_next_task() else {
        info!("No more tasks to execute");
        return;
    };
    let htn = &assets.get(&sup.htn_handle).unwrap().htn;
    let Some(Task::Primitive(task)) = htn.get_task_by_name(task_id.name()) else {
        panic!("Task {task_id:?} is not a primitive on this htn");
    };
    // this will trigger an HtnTaskExecute<Operator> event on our supervisor entity.
    // but if all we're doing is turning it into a tree, maybe do that internally and yield a non-generic trigger that
    // includes the plan name, task id, and tree?
    let task_strategy = task.execution_command(state, &type_registry.read(), &task_id);
    match task_strategy {
        TaskExecutionStrategy::BehaviourTree { tree, task_id } => {
            warn!("Executing task: {task_id:?} via behaviour tree: {tree}");
            let troll_entity = parent.get();
            commands
                .spawn((
                    task_id,
                    BehaveTree::new(tree),
                    BehaveTargetEntity::Entity(troll_entity),
                    BehaveSupervisorEntity(t.entity()),
                ))
                .set_parent(t.entity());
        }
    }
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
