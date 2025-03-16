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

impl<
        T: Reflect + Component + TypePath + Default + Clone + core::fmt::Debug + GetTypeRegistration,
    > Plugin for HtnPlugin<T>
{
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
