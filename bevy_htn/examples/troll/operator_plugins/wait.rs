use crate::*;

pub fn wait_plugin(app: &mut App) {
    app.register_type::<WaitOperator>();
}
