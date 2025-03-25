mod dsl;
mod executor;
mod htn;
mod htn_assets;
mod planner;
mod reflect_operator;
#[cfg(test)]
mod tests;

/// Auto-implemented trait that HTN Planner state must abide by. Used as a trait alias.
pub trait HtnStateTrait:
    Reflect + GetTypeRegistration + Default + TypePath + Clone + core::fmt::Debug + Component
{
}
impl<
        T: Reflect + GetTypeRegistration + Default + TypePath + Clone + core::fmt::Debug + Component,
    > HtnStateTrait for T
{
}

pub mod prelude {
    pub use super::dsl::*;
    pub use super::executor::*;
    pub use super::htn::*;
    pub use super::htn_assets::*;
    pub use super::planner::*;
    pub use super::reflect_operator::*;
    pub use super::HtnPlugin;
    pub use bevy_behave::prelude::*;
    pub use bevy_htn_macros::HtnOperator;
}

use bevy::{prelude::*, reflect::GetTypeRegistration};
pub use bevy_behave;
use bevy_behave::prelude::BehavePlugin;
use prelude::*;
use std::marker::PhantomData;

pub struct HtnPlugin<T: HtnStateTrait> {
    phantom: PhantomData<T>,
}

impl<T: HtnStateTrait> Default for HtnPlugin<T> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T: HtnStateTrait> Plugin for HtnPlugin<T> {
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
