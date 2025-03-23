//use bevy_htn::prelude::*;

mod boxy_dude;
mod coins;
mod cursor;
mod setup;
use bevy::{
    color::palettes::css, input::common_conditions::input_toggle_active,
    pbr::CascadeShadowConfigBuilder, prelude::*, window::PrimaryWindow,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use boxy_dude::*;
use coins::*;
use cursor::*;
use setup::*;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add camera, ground plane, light, etc
        .add_plugins(setup_plugin)
        // Track cursor position on the ground plane, make player look towards cursor
        .add_plugins(cursor_plugin)
        // Periodic coin spawner and coin collision events
        .add_plugins(coin_plugin)
        // Character controller for all little boxy dudes, and input controller for human.
        .add_plugins(dude_plugin)
        .add_plugins(
            // Show inspector with F12
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F12)),
        )
        .add_systems(Update, draw_debug)
        .run();
}

fn draw_debug(mut gizmos: Gizmos, level_config: Res<LevelConfig>) {
    let iso = Quat::from_rotation_arc(Vec3::Z, Vec3::Y);
    gizmos.rect(
        iso,
        Vec2::new(level_config.width, level_config.height),
        css::BLUE,
    );
}
