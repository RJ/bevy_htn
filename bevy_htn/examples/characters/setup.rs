use crate::*;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;

pub fn setup_plugin(app: &mut App) {
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 2000.,
    })
    .insert_resource(LevelConfig {
        width: 350.0,
        height: 350.0,
    })
    .register_type::<LevelConfig>()
    .add_systems(Startup, setup);
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct LevelConfig {
    pub width: f32,
    pub height: f32,
}

impl LevelConfig {
    pub fn random_position(&self) -> (f32, f32) {
        let a = rand::rng().random_range(0.0..TAU);
        let max_rad = self.width.min(self.height) / 2.0;
        let rad = rand::rng().random_range(0.0..max_rad);
        let x = rad * a.cos();
        let z = rad * a.sin();
        (x, z)
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(0., 70., 300.).looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
    ));

    // Plane
    commands.spawn((
        Name::new("Plane"),
        GroundPlaneMarker,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Light
    commands.spawn((
        Name::new("Light"),
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
    ));
}
