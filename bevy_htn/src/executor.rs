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
        app.add_systems(Update, task_finished);
        app.add_observer(on_exec_next_task::<T>);
        app.add_observer(on_plan_added);
        app.add_observer(on_task_complete::<T>);
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

/// This entity is the parent of the HTN operator entities.
/// It holds the HTN asset and the current plan, and is a direct child of the troll.
#[derive(Component, Reflect)]
pub struct HtnSupervisor<T: Reflect + TypePath + Default + Clone + core::fmt::Debug> {
    pub htn_handle: Handle<HtnAsset<T>>,
}

fn on_plan_added(t: Trigger<OnInsert, Plan>, mut commands: Commands) {
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
    let entity = t.entity();
    let Ok((mut plan, htn_sup, mut state)) = q.get_mut(entity) else {
        error!("HtnSupervisor {entity:?} not found");
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
                // TODO perhaps bypass change detection here?
                primitive.apply_effects(&mut state, atr.as_ref());
            }
            Task::Compound(_compound) => {}
        }
    } else {
        error!("Task {task_id:?} failed - replan?"); // TODO
        return;
    }
    commands.trigger_targets(ExecNextTask, t.entity());
    // commands.entity(t.entity()).remove_children(&[entity]);
    // commands.entity(entity).try_despawn_recursive();

    // };
    // commands.trigger_targets(ExecNextTask, t.entity());
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
    let task_name = task_id.name();
    let htn = &assets.get(&sup.htn_handle).unwrap().htn;
    let Some(Task::Primitive(task)) = htn.get_task_by_name(task_id.name()) else {
        panic!("Task {task_id:?} is not a primitive on this htn");
    };
    // this will trigger an HtnTaskExecute<Operator> event on our supervisor entity.
    // but if all we're doing is turning it into a tree, maybe do that internally and yield a non-generic trigger that
    // includes the plan name, task id, and tree?
    let task_strategy = task.execution_command(state, &type_registry.read(), t.entity(), &task_id);
    match task_strategy {
        TaskExecutionStrategy::Command(cmd) => {
            warn!("Executing task: {task_name} via trigger");
            commands.queue(cmd);
        }
        TaskExecutionStrategy::BehaviourTree(tree) => {
            warn!("Executing task: {task_name} via behaviour tree: {tree}");
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
        // info!("Task {task_id:?} finished {finished:?} on {entity}");
        // one level up is the htn sup
        // let sup_entity = parent.get();
        // let Ok((mut plan, sup)) = q_sup.get_mut(sup_entity) else {
        //     error!("HtnSupervisor {sup_entity:?} not found");
        //     continue;
        // };
        // plan.report_task_completion(&task_id, finished.0);
        // commands.trigger_targets(ExecNextTask, parent.get());
        // commands.entity(parent.get()).remove_children(&[entity]);
        // commands.entity(entity).try_despawn_recursive();
    }
}
