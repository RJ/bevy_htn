mod dsl;
mod executor;
mod htn;
mod htn_assets;
mod planner;
mod reflect_operator;

pub mod prelude {
    pub use super::dsl::*;
    pub use super::executor::*;
    pub use super::htn::*;
    pub use super::htn_assets::*;
    pub use super::planner::*;
    pub use super::reflect_operator::*;
    pub use super::HtnPlugin;
    pub use bevy_behave;
    pub use bevy_htn_macros::HtnOperator;
}

use bevy::{prelude::*, reflect::GetTypeRegistration};
use bevy_behave::prelude::BehavePlugin;
use prelude::*;
use std::marker::PhantomData;

pub struct HtnPlugin<T: Reflect + Component + TypePath> {
    phantom: PhantomData<T>,
}

impl<T: Reflect + Component + TypePath> Default for HtnPlugin<T> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T: Reflect + Component + TypePath + GetTypeRegistration> Plugin for HtnPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BehavePlugin>() {
            app.add_plugins(BehavePlugin::new(Update));
        }
        app.register_type::<T>();
        app.register_type::<PlannedTaskId>();
        app.register_type::<PlannedTask>();
        app.register_type::<Plan>();
        app.add_plugins(executor::HtnExecutorPlugin::<T>::default());
    }
}

// pub struct HtnPlugin<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> {
//     _phantom: std::marker::PhantomData<T>,
// }

// impl<T: Reflect + Default + TypePath + Clone + core::fmt::Debug> Plugin for HtnPlugin<T> {
//     fn build(&self, app: &mut App) {
//         app.register_type::<T>();

//     }
// }

// fn startup(world: &mut World) {
//     // Here we specify that our HTN is for GameState.
//     let htn = parse_htn::<GameState>(dsl);
//     println!("Parsed HTN: {:#?}", htn);

//     // Example execution of top-level tasks (subtask execution omitted for brevity):
//     // let mut state = GameState {
//     //     gold: false,
//     //     energy: 1,
//     // };
//     // println!("Initial state: {:#?}", state);
//     // for task in htn.tasks.iter() {
//     //     if task.preconditions.iter().all(|c| c.evaluate(&state)) {
//     //         info!("Executing task: {}", task.name);
//     //         println!("State: {:#?}", state);
//     //         let mut entity = world.spawn(());
//     //         task.insert_operator(&state, &app_type_registry.read(), &mut entity);
//     //         let eid = entity.id();
//     //         world.commands().entity(eid).log_components();

//     //         for eff in task.effects.iter() {
//     //             eff.apply(&mut state);
//     //         }
//     //     } else {
//     //         println!("Skipping task: {}", task.name);
//     //         println!("State: {:#?}", state);
//     //     }
//     // }
//     // println!("Final state: {:#?}", state);
// }
