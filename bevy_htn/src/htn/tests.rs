use super::*;
use crate::prelude::*;

trait AppTestExt {
    fn atr(&self) -> &AppTypeRegistry;
}

impl AppTestExt for App {
    fn atr(&self) -> &AppTypeRegistry {
        self.world().get_resource::<AppTypeRegistry>().unwrap()
    }
}

#[derive(Reflect, Default, Clone, Debug, PartialEq, Eq)]
#[reflect(Default)]
enum Location {
    #[default]
    Home,
    Work,
}

#[derive(Reflect, Resource, Clone, Debug, Default)]
#[reflect(Default, Resource)]
struct TestState {
    tog: bool,
    location: Location,
}

#[derive(Debug, Reflect, Default, Clone, HtnOperator)]
#[reflect(Default, HtnOperator)]
pub struct TestOperator1;

fn setup_app() -> App {
    let mut app = App::new();
    app.register_type::<TestState>();
    app.register_type::<TestOperator1>();
    app
}

#[test]
fn test_set_bool() {
    let app = setup_app();
    let mut state = TestState::default();
    let effect = Effect::SetBool {
        field: "tog".to_string(),
        value: true,
    };
    effect.apply(&mut state, app.atr());
    assert!(state.tog);
}

#[test]
fn test_set_enum() {
    let app = setup_app();
    let mut state = TestState::default();
    let effect = Effect::SetEnum {
        field: "location".to_string(),
        enum_type: "Location".into(),
        enum_variant: "Work".into(),
    };
    effect.apply(&mut state, app.atr());
    assert_eq!(state.location, Location::Work);
}

#[test]
fn test_cond_bool() {
    let app = setup_app();
    let mut state = TestState::default();
    let cond = HtnCondition::EqualsBool {
        field: "tog".to_string(),
        value: false,
    };
    assert!(cond.evaluate(&state, app.atr()));
    state.tog = true;
    assert!(!cond.evaluate(&state, app.atr()));
}

#[test]
fn test_cond_enum() {
    let app = setup_app();
    let mut state = TestState::default();
    let cond = HtnCondition::EqualsEnum {
        field: "location".to_string(),
        enum_type: "Location".into(),
        enum_variant: "Home".into(),
    };
    assert!(cond.evaluate(&state, app.atr()));
    state.location = Location::Work;
    assert!(!cond.evaluate(&state, app.atr()));
}

#[test]
fn test_parser() {
    let src = r#"
    primitive_task "test_operator1" {
        operator: trigger: TestOperator1;
        precondition: tog == false;
        effect: set tog = true;
        expected_effect: set location = Location::Work;
    }
    "#;
    let htn = parse_htn::<TestState>(src);
    assert_eq!(htn.tasks.len(), 1);
    let Task::Primitive(task) = &htn.tasks[0] else {
        panic!("Task is not a primitive");
    };
    assert_eq!(task.name, "test_operator1");
    assert_eq!(task.preconditions.len(), 1);
    assert_eq!(task.effects.len(), 1);
    assert_eq!(task.expected_effects.len(), 1);
    assert_eq!(
        task.preconditions[0],
        HtnCondition::EqualsBool {
            field: "tog".to_string(),
            value: false,
        }
    );
    assert_eq!(
        task.effects[0],
        Effect::SetBool {
            field: "tog".to_string(),
            value: true,
        }
    );
    assert_eq!(
        task.expected_effects[0],
        Effect::SetEnum {
            field: "location".to_string(),
            enum_type: "Location".into(),
            enum_variant: "Work".into(),
        }
    );
}
