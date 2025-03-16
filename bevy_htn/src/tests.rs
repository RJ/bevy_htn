use crate::prelude::*;
use bevy::prelude::*;
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
    counter: i32,
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
fn test_cond_int() {
    let app = setup_app();
    let mut state = TestState::default();
    let cond = HtnCondition::EqualsInt {
        field: "counter".to_string(),
        value: 0,
    };
    assert!(cond.evaluate(&state, app.atr()));
    state.counter = 1;
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
    primitive_task "TestTask1" {
        operator: TestOperator1
        preconditions: [tog == false, location == Location::Home]
        effects: [
            tog = true,
            counter -= 1,
        ]
        expected_effects: [location = Location::Work]
    }
    compound_task "CompoundTask1" {
        method {
            preconditions: [tog == true]
            subtasks: [TestTask1]
        }
        method {
            preconditions: []
            subtasks: [FooTask,]
        }
        method {
            subtasks: [FooTask, BarTask]
        }
    }
    "#;
    let htn = parse_htn::<TestState>(src);
    assert_eq!(htn.tasks.len(), 2);
    let Task::Primitive(task1) = &htn.tasks[0] else {
        panic!("Task is not a primitive");
    };
    assert_eq!(task1.name, "TestTask1");
    assert_eq!(
        task1.expected_effects,
        vec![Effect::SetEnum {
            field: "location".to_string(),
            enum_type: "Location".into(),
            enum_variant: "Work".into(),
        }]
    );
    assert_eq!(
        task1.preconditions,
        vec![
            HtnCondition::EqualsBool {
                field: "tog".to_string(),
                value: false,
            },
            HtnCondition::EqualsEnum {
                field: "location".to_string(),
                enum_type: "Location".into(),
                enum_variant: "Home".into(),
            },
        ]
    );
    assert_eq!(
        task1.effects,
        vec![
            Effect::SetBool {
                field: "tog".to_string(),
                value: true,
            },
            Effect::IncrementInt {
                field: "counter".to_string(),
                by: -1,
            },
        ]
    );
    assert_eq!(
        task1.expected_effects[0],
        Effect::SetEnum {
            field: "location".to_string(),
            enum_type: "Location".into(),
            enum_variant: "Work".into(),
        }
    );
    let Task::Compound(task2) = &htn.tasks[1] else {
        panic!("Task is not a compound");
    };
    assert_eq!(task2.name, "CompoundTask1");
    assert_eq!(task2.methods.len(), 3);
    assert_eq!(
        task2.methods[0].preconditions,
        vec![HtnCondition::EqualsBool {
            field: "tog".to_string(),
            value: true,
        }]
    );
    assert_eq!(task2.methods[1].preconditions, vec![]);
    assert_eq!(task2.methods[1].subtasks, vec!["FooTask".to_string()]);
    assert_eq!(
        task2.methods[2].subtasks,
        vec!["FooTask".to_string(), "BarTask".to_string()]
    );

    assert_eq!(task2.methods[0].subtasks, vec!["TestTask1".to_string()]);
    assert_eq!(task2.methods[1].subtasks, vec!["FooTask".to_string()]);
}
