use bevy::{prelude::*, render::camera::Viewport, window::PrimaryWindow};
use bevy_htn::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContext, EguiPlugin},
    bevy_inspector, DefaultInspectorConfigPlugin,
};
use bevy_pancam::*;

use crate::{setup_level::LoadingState, GameState, HtnSupervisor};

/// lerp factor when constraining viewport if sidebars resize/toggle
const VIEWPORT_LERP: f32 = 0.07;

pub struct TrollUiPlugin;

impl Plugin for TrollUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin);
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }
        app.add_systems(Startup, setup_camera);
        app.add_systems(
            Update,
            (initial_ui
                .pipe(left_sidebar)
                // .pipe(right_sidebar)
                .pipe(set_viewport_margins),)
                .run_if(in_state(LoadingState::Ready)),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        PanCam {
            move_keys: DirectionKeys::NONE,
            ..default()
        },
        Camera2d,
    ));
}
/// systems for sidebars and bits of egui ui are piped together, passing this struct,
/// and the final system in the pipeline modifies the camera viewport.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
struct UiMargins {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

fn initial_ui() -> UiMargins {
    UiMargins::default()
}

fn left_sidebar(
    In(mut margins): In<UiMargins>,
    world: &mut World,
    // Exclusive systems can also have a SystemState if needed:
    // ss: &mut SystemState<(Query<&mut Camera>, Query<&mut Window, With<PrimaryWindow>>)>,
) -> UiMargins {
    let mut q = world.query::<&mut EguiContext>();
    let ctx = q.single_mut(world).get_mut().clone();
    // let mut gamestate = world.query::<&mut GameState>().single_mut(world);
    // let rolodex = world.get_resource::<Rolodex>().unwrap();
    // let troll = rolodex.troll;
    let (entity, _htn_sup, opt_plan) = world
        .query_filtered::<(Entity, &HtnSupervisor<GameState>, Option<&Plan>), With<GameState>>()
        .single(world);
    let plan_id_str = opt_plan.map_or("".to_string(), |p| {
        format!(" [{}] = {:?}", p.id(), p.status())
    });
    let tasks = opt_plan
        .map(|plan| {
            plan.tasks
                .iter()
                .map(|t| format!("{} - {:?}", t.name, t.status))
                .collect::<Vec<_>>()
        })
        .unwrap_or(vec![]);
    // .plan
    // .as_ref()
    // .map(|p| p.tasks.clone())
    // .unwrap_or(vec![]);
    margins.left += egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(225.0)
        .show(&ctx, |ui| {
            ui.heading("Troll HTN Example");
            ui.separator();
            bevy_inspector::ui_for_entity(world, entity, ui);
            ui.heading(format!("Current Plan\n{plan_id_str}",));
            if tasks.is_empty() {
                ui.label("None");
            } else {
                for task in tasks.iter() {
                    ui.label(format!("> {task}"));
                }
            }
        })
        .response
        .rect
        .width();

    margins
}
/// the various sidebars will consume screen space, dimensions of which stored in UiMargins.
/// so we update the camera viewport so it doesn't render underneath the sidebars.
fn set_viewport_margins(
    In(margins): In<UiMargins>,
    mut cameras: Query<(&mut Camera, &mut Transform)>,
    q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut old_margins: Local<UiMargins>,
) {
    if *old_margins == margins {
        return;
    }
    info!("Margins changed, updating viewport: {margins:?}");
    *old_margins = margins;
    let (mut camera, mut _camera_transform) = cameras.get_single_mut().expect("No camera found");
    let window = q_window.get_single().expect("No primary window found");

    // egui gives us logical units, scale up to physical units here
    let right = margins.right * window.scale_factor();
    let left = margins.left * window.scale_factor();
    let top = margins.top * window.scale_factor();
    let bottom = margins.bottom * window.scale_factor();

    // we could just snap viewport to pos,size - but we'll lerp for a smoother transition
    // when hiding and showing sidebars.
    let mut pos = UVec2::new(left as u32, top as u32);
    let mut size = UVec2::new(window.physical_width(), window.physical_height())
        - pos
        - UVec2::new(right as u32, bottom as u32);

    let (physical_size, physical_position) = if let Some(Viewport {
        physical_size,
        physical_position,
        ..
    }) = camera.viewport
    {
        lerp_onto(physical_size, &mut size, VIEWPORT_LERP);
        lerp_onto(physical_position, &mut pos, VIEWPORT_LERP);
        (size, pos)
    } else {
        (size, pos)
    };
    info!("Setting viewport to {physical_size:?} {physical_position:?}");
    camera.viewport = Some(Viewport {
        physical_size,
        physical_position,
        ..default()
    });

    // camera_transform.translation.x = -left;
}

// lerp and overwrite b with result
fn lerp_onto(a: UVec2, b: &mut UVec2, t: f32) {
    b.x = ((1.0 - t) * a.x as f32 + t * b.x as f32) as u32;
    b.y = ((1.0 - t) * a.y as f32 + t * b.y as f32) as u32;
}
