use crate::*;

pub fn trunk_slam_plugin(app: &mut App) {
    app.register_type::<DoTrunkSlamOperator>();
    app.add_systems(Update, trunk_slam_system);
}

const ANIM_SECS: f32 = 2.0;

fn trunk_slam_system(
    mut q_behave: Query<(&BehaveCtx, &mut DoTrunkSlamOperator)>,
    mut q_troll: Query<&mut Transform, With<Troll>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (ctx, mut trunk_slam) in q_behave.iter_mut() {
        if trunk_slam.start == 0.0 {
            trunk_slam.start = time.elapsed_secs();
        }
        let mut troll_transform = q_troll
            .get_mut(ctx.target_entity())
            .expect("Troll not found");

        let progress = (time.elapsed_secs() - trunk_slam.start) / ANIM_SECS;
        if progress >= 1.0 {
            troll_transform.rotation = Quat::from_rotation_z(0.0);
            commands.trigger(ctx.success());
            continue;
        }
        troll_transform.rotation = Quat::from_rotation_z(progress * std::f32::consts::TAU);
    }
}
