use crate::*;

pub fn regain_los_plugin(app: &mut App) {
    app.register_type::<RegainLOSOperator>();
    app.add_observer(on_regain_los);
}

fn on_regain_los(t: Trigger<BehaveTrigger<RegainLOSOperator>>, mut commands: Commands) {
    let ctx = t.ctx();
    info!("RegainLOSOperator ran.");
    commands.trigger(ctx.failure());
}
