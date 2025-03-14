use crate::prelude::*;
use bevy::prelude::*;
use bevy_behave::prelude::*;
use std::marker::PhantomData;

pub struct HtnExecutorPlugin<T: Reflect + Component + TypePath> {
    phantom: PhantomData<T>,
}

impl<T: Reflect + Component + TypePath> Default for HtnExecutorPlugin<T> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T: Reflect + Component + TypePath> Plugin for HtnExecutorPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, task_finished::<T>);
        app.add_observer(on_exec_next_task::<T>);
        app.add_observer(on_plan_added);
    }
}

/// This entity is the parent of the HTN operator entities.
/// It holds the HTN asset and the current plan, and is a direct child of the troll.
#[derive(Component, Reflect)]
pub struct HtnSupervisor<T: Reflect + TypePath> {
    pub htn_handle: Handle<HtnAsset<T>>,
}

fn on_plan_added(t: Trigger<OnInsert, Plan>, mut commands: Commands) {
    commands.trigger_targets(ExecNextTask, t.entity());
}

fn on_exec_next_task<T: Reflect + Component + TypePath>(
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
    let Some((task_id, task_name)) = plan.execute_next_task() else {
        info!("No more tasks to execute");
        return;
    };

    let htn = &assets.get(&sup.htn_handle).unwrap().htn;
    let Some(Task::Primitive(task)) = htn.get_task_by_name(&task_name) else {
        panic!("Task {task_name} is not a primitive on this htn");
    };
    // this will trigger an HtnTaskExecute<Operator> event on our supervisor entity.
    // but if all we're doing is turning it into a tree, maybe do that internally and yield a non-generic trigger that
    // includes the plan name, task id, and tree?
    if let Some((_cmd, tree)) =
        task.execution_command(state, &type_registry.read(), Some(t.entity()))
    {
        let troll_entity = parent.get();
        commands
            .spawn((
                task_id,
                BehaveTree::new(tree),
                BehaveTargetEntity::Entity(troll_entity),
            ))
            .set_parent(t.entity());
        // commands.queue(cmd);
    } else {
        error!("Task {task_name} has no execution command");
    }
}

#[derive(Event)]
struct ExecNextTask;

fn task_finished<T: Reflect + Component + TypePath>(
    q: Query<(Entity, &BehaveFinished, &PlannedTaskId, &Parent), Added<BehaveFinished>>,
    mut q_sup: Query<(&mut Plan, &HtnSupervisor<T>)>,
    mut commands: Commands,
) {
    for (entity, finished, task_id, parent) in q.iter() {
        // info!("Task {task_id:?} finished {finished:?} on {entity}");
        // one level up is the htn sup
        let sup_entity = parent.get();
        let Ok((mut plan, sup)) = q_sup.get_mut(sup_entity) else {
            error!("HtnSupervisor {sup_entity:?} not found");
            continue;
        };
        plan.report_task_completion(*task_id, finished.0);
        commands.trigger_targets(ExecNextTask, parent.get());
        commands.entity(parent.get()).remove_children(&[entity]);
        commands.entity(entity).try_despawn_recursive();
    }
}
