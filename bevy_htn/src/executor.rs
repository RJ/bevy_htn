use crate::prelude::*;
use bevy::prelude::*;
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
        app.add_systems(Update, plan_added::<T>);
    }
}

/// This entity is the parent of the HTN operator entities.
/// It holds the HTN asset and the current plan, and is a direct child of the troll.
#[derive(Component, Reflect)]
pub struct HtnSupervisor<T: Reflect + TypePath> {
    pub htn_handle: Handle<HtnAsset<T>>,
}

fn plan_added<T: Reflect + Component + TypePath>(
    mut commands: Commands,
    mut q: Query<(Entity, Option<&Children>, &HtnSupervisor<T>, &mut Plan, &T), Added<Plan>>,
    q_children: Query<Entity, With<PlannedTaskId>>,
    assets: Res<Assets<HtnAsset<T>>>,
    type_registry: Res<AppTypeRegistry>,
) {
    let type_registry = type_registry.read();
    for (entity, children, sup, mut plan, state) in q.iter_mut() {
        info!("New plan added on {entity:?}: {}", *plan);
        // kill any children executing a previous plan:
        if let Some(children) = children {
            for child in children.iter().filter(|c| q_children.contains(**c)) {
                info!("Killing child executing old plan: {child:?}");
                commands
                    .entity(entity)
                    .remove_children(&[*child])
                    .remove::<PlannedTaskId>();
                commands.entity(*child).despawn_recursive();
            }
        }

        let Some((task_id, task_name)) = plan.execute_next_task() else {
            info!("No more tasks to execute");
            continue;
        };

        let htn = &assets.get(&sup.htn_handle).unwrap().htn;
        let Some(Task::Primitive(task)) = htn.get_task_by_name(&task_name) else {
            panic!("Task {task_name} is not a primitive on this htn");
        };
        // this will trigger an HtnTaskExecute<Operator> event on our supervisor entity.
        // but if all we're doing is turning it into a tree, maybe do that internally and yield a non-generic trigger that
        // includes the plan name, task id, and tree?
        if let Some(cmd) = task.execution_command(state, &type_registry, Some(entity)) {
            commands.entity(entity).insert(task_id);
            commands.queue(cmd);
        } else {
            error!("Task {task_name} has no execution command");
        }
    }
}
