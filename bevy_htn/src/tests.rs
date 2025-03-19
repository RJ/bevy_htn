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
        syntax: "tog = true".to_string(),
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
        syntax: "location = Location::Work".to_string(),
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
        notted: false,
        syntax: "tog == false".to_string(),
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
        notted: false,
        syntax: "counter == 0".to_string(),
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
        notted: false,
        syntax: "location == Location::Home".to_string(),
    };
    assert!(cond.evaluate(&state, app.atr()));
    state.location = Location::Work;
    assert!(!cond.evaluate(&state, app.atr()));
}

#[test]
fn test_parser() {
    let src = r#"
    // the htn block defines what version of the crate your DSL is written for.
    // i  won't bother with backwards compatability unless there are lots of users on old versions,
    // but having a version string is necessary if I ever need to do that.
    schema {
        version: 0.1.0
    }

    // comment
    primitive_task "TestTask1" {
        // comment
        operator: TestOperator1
        preconditions: [tog == false, location == Location::Home]
        effects: [
            tog = true,
            // comment
            counter -= 1,
        ]
        expected_effects: [location = Location::Work]
    }
    // comment
    compound_task "CompoundTask1" {
        // comment
        method {
            preconditions: [tog == true]
            // comment
            subtasks: [TestTask1]
        }
        method {
            preconditions: [
            // comment
            ]
            subtasks: [FooTask,]
        }
        method {
            // comment
            subtasks: [
            FooTask,
            // comment
            BarTask
            ]
        }
    }
    "#;
    let htn = parse_htn::<TestState>(src);
    assert_eq!(htn.version(), "0.1.0");
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
            syntax: "location = Location::Work".to_string(),
        }]
    );
    assert_eq!(
        task1.preconditions,
        vec![
            HtnCondition::EqualsBool {
                field: "tog".to_string(),
                value: false,
                notted: false,
                syntax: "tog == false".to_string(),
            },
            HtnCondition::EqualsEnum {
                field: "location".to_string(),
                enum_type: "Location".into(),
                enum_variant: "Home".into(),
                notted: false,
                syntax: "location == Location::Home".to_string(),
            },
        ]
    );
    assert_eq!(
        task1.effects,
        vec![
            Effect::SetBool {
                field: "tog".to_string(),
                value: true,
                syntax: "tog = true".to_string(),
            },
            Effect::IncrementInt {
                field: "counter".to_string(),
                by: -1,
                syntax: "counter -= 1".to_string(),
            },
        ]
    );
    assert_eq!(
        task1.expected_effects[0],
        Effect::SetEnum {
            field: "location".to_string(),
            enum_type: "Location".into(),
            enum_variant: "Work".into(),
            syntax: "location = Location::Work".to_string(),
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
            notted: false,
            syntax: "tog == true".to_string(),
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

#[test]
fn test_travel_htn() {
    {
        // Don't need app, just want to set up the logger.
        let mut app = App::new();
        app.add_plugins(bevy::log::LogPlugin::default());
    }

    info!("Test travel HTN");
    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq)]
    #[reflect(Default)]
    enum Location {
        #[default]
        Home,
        Other,
        Park,
    }

    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq, HtnOperator)]
    #[reflect(Default, HtnOperator)]
    struct WalkOperator;

    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq, HtnOperator)]
    #[reflect(Default, HtnOperator)]
    struct TaxiOperator;

    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq, HtnOperator)]
    #[reflect(Default, HtnOperator)]
    struct RideTaxiOperator;

    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq, HtnOperator)]
    #[reflect(Default, HtnOperator)]
    struct PayTaxiOperator;

    #[derive(Reflect, Resource, Clone, Debug, Default)]
    #[reflect(Default, Resource)]
    struct TravelState {
        cash: i32,
        distance_to_park: i32,
        my_location: Location,
        taxi_location: Location,
    }

    let src = r#"
    schema {
        version: 0.1.0
    }

    compound_task "TravelToPark" {
        method {
            subtasks: [ Walk ]
        }
        method {
            subtasks: [ Taxi ]
        }
    }
            
    primitive_task "Walk" {
        operator: WalkOperator
        preconditions: [distance_to_park <= 4, my_location != Location::Park]
        effects: [
            my_location = Location::Park,
        ]
    }

    compound_task "Taxi" {
        method {
            subtasks: [CallTaxi, RideTaxi, PayTaxi]
        }
    }

    primitive_task "CallTaxi" {
        operator: TaxiOperator
        preconditions: [taxi_location != Location::Park, cash >= 1]
        effects: [taxi_location = Location::Home]
    }

    primitive_task "RideTaxi" {
        operator: RideTaxiOperator
        preconditions: [taxi_location == Location::Home, cash >= 1]
        effects: [taxi_location = Location::Park, my_location = Location::Park]
    }

    primitive_task "PayTaxi" {
        operator: PayTaxiOperator
        preconditions: [taxi_location == Location::Park, cash >= 1]
        effects: [cash -= 1]
    }
    "#;
    let atr = AppTypeRegistry::default();
    {
        let mut atr = atr.write();
        atr.register::<TravelState>();
        atr.register::<Location>();
        atr.register::<WalkOperator>();
        atr.register::<TaxiOperator>();
        atr.register::<RideTaxiOperator>();
        atr.register::<PayTaxiOperator>();
    }
    let htn = parse_htn::<TravelState>(src);
    // verify via reflection that any types used in the htn are registered:
    match htn.verify_operators(&TravelState::default(), &atr) {
        Ok(_) => {}
        Err(e) => panic!("Type verification failed: {e:?}"),
    }
    match htn.verify_conditions(&TravelState::default(), &atr) {
        Ok(_) => {}
        Err(e) => panic!("Condition verification failed: {e:?}"),
    }
    match htn.verify_effects(&TravelState::default(), &atr) {
        Ok(_) => {}
        Err(e) => panic!("Effect verification failed: {e:?}"),
    }
    // assert!(
    //     htn.verify_operators(&TravelState::default(), &atr).is_ok(),
    //     "HTN Type verification failed!"
    // );
    let mut planner = HtnPlanner::new(&htn, &atr);

    {
        warn!("Testing walking state");
        let initial_state = TravelState {
            cash: 10,
            distance_to_park: 1,
            my_location: Location::Home,
            taxi_location: Location::Other,
        };
        let plan = planner.plan(&initial_state);
        assert_eq!(plan.task_names(), vec!["Walk"]);
    }

    {
        warn!("Testing taxi state");
        let initial_state = TravelState {
            cash: 10,
            distance_to_park: 5,
            my_location: Location::Home,
            taxi_location: Location::Other,
        };
        let plan = planner.plan(&initial_state);
        assert_eq!(plan.task_names(), vec!["CallTaxi", "RideTaxi", "PayTaxi"]);
    }
}

#[test]
fn test_verify_effects() {
    {
        // Don't need app, just want to set up the logger.
        let mut app = App::new();
        app.add_plugins(bevy::log::LogPlugin::default());
    }

    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq)]
    #[reflect(Default)]
    enum Location {
        #[default]
        Home,
        Other,
        Park,
    }

    #[derive(Reflect, Resource, Clone, Debug, Default)]
    #[reflect(Default, Resource)]
    struct State {
        energy: i32,
        happy: bool,
        location: Location,
    }

    let src = r#"
    schema {
        version: 0.1.0
    }
            
    primitive_task "Walk" {
        operator: WalkOperator
        preconditions: [energy > 10, location != Location::Park, happy == false]
        effects: [
            location = Location::Park,
            energy -= 1,
            happy = true
        ]
    }
    "#;
    let atr = AppTypeRegistry::default();
    {
        let mut atr = atr.write();
        atr.register::<State>();
        atr.register::<Location>();
    }
    let htn = parse_htn::<State>(src);
    match htn.verify_effects(&State::default(), &atr) {
        Ok(_) => {}
        Err(e) => panic!("Effect verification failed: {e:?}"),
    }
    // verify via reflection that any types used in the htn are registered:
    match htn.verify_operators(&State::default(), &atr) {
        Ok(_) => {}
        Err(e) => error!("Type verification failed: {e:?}"),
    }
}
