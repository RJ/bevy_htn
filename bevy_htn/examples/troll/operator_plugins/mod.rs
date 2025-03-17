use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

mod check_bridge;
mod choose_bridge;
mod find_trunk;
mod navigate_to;
mod regain_los;
mod trunk_slam;
mod uproot_trunk;
mod wait;

// if we implemented to_tree manually for an operator we'd probably need to expose the executor
// structs here to build the tree:
pub mod prelude {}

pub struct OperatorPlugins;

impl PluginGroup for OperatorPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(navigate_to::navigate_to_plugin)
            .add(trunk_slam::trunk_slam_plugin)
            .add(check_bridge::check_bridge_plugin)
            .add(uproot_trunk::uproot_trunk_plugin)
            .add(find_trunk::find_trunk_plugin)
            .add(choose_bridge::choose_bridge_plugin)
            .add(regain_los::regain_los_plugin)
            .add(wait::wait_plugin)
    }
}
