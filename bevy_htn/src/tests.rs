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

#[derive(Reflect, Resource, Clone, Debug, Component)]
#[reflect(Default, Resource)]
struct TestState {
    tog: bool,
    location: Location,
    counter: i32,
    e1: Entity,
    e2: Entity,
}

impl Default for TestState {
    fn default() -> Self {
        Self {
            tog: false,
            location: Location::Home,
            counter: 0,
            e1: Entity::PLACEHOLDER,
            e2: Entity::PLACEHOLDER,
        }
    }
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
        preconditions: [tog == false, location == Location::Home, e1 == e2]
        effects: [
            tog = true,
            // comment
            counter -= 1,
        ]
        expected_effects: [location = Location::Work, e1 = e2]
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
        vec![
            Effect::SetEnum {
                field: "location".to_string(),
                enum_type: "Location".into(),
                enum_variant: "Work".into(),
                syntax: "location = Location::Work".to_string(),
            },
            Effect::SetIdentifier {
                field: "e1".to_string(),
                field_source: "e2".to_string(),
                syntax: "e1 = e2".to_string(),
            },
        ]
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
            HtnCondition::EqualsIdentifier {
                field: "e1".to_string(),
                other_field: "e2".to_string(),
                notted: false,
                syntax: "e1 == e2".to_string(),
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
        // Don't need app for test, just want to set up the logger.
        let mut app = App::new();
        app.add_plugins(bevy::log::LogPlugin::default());
    }

    // DEFINE OPERATORS (which are behaviour trees)

    // the default behaviour of operators is to be emitted as triggers,
    // ie. Behave::trigger(WalkOperator)
    // but you can also make them components to be spawned, or implement HtnOperator
    // yourself to provide a custom behaviour tree.
    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq, HtnOperator)]
    #[reflect(Default, HtnOperator)]
    struct WalkOperator;

    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq, HtnOperator)]
    #[reflect(Default, HtnOperator)]
    struct TaxiOperator;

    // an operator that returns a custom behaviour tree.
    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq)]
    #[reflect(Default, HtnOperator)]
    struct RideTaxiOperator(i32);
    impl HtnOperator for RideTaxiOperator {
        fn to_tree(&self) -> Tree<Behave> {
            behave! { Behave::Wait(self.0 as f32) }
        }
    }

    // this one would get spawned into an entity using Behave::spawn_named.
    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq, HtnOperator, Component)]
    #[reflect(Default, HtnOperator)]
    #[spawn_named = "Paying the taxi!"]
    struct PayTaxiOperator;

    // DEFINE PLANNER STATE

    #[derive(Reflect, Default, Clone, Debug, PartialEq, Eq)]
    #[reflect(Default)]
    enum Location {
        #[default]
        Home,
        Other,
        Park,
    }

    #[derive(Reflect, Resource, Clone, Debug, Default, Component)]
    #[reflect(Default, Resource)]
    struct TravelState {
        cash: i32,
        distance_to_park: i32,
        happy: bool,
        my_location: Location,
        taxi_location: Location,
    }

    // DEFINE HTN (can be loaded from an .htn file by asset loader too)

    // for an initial state with distance > 4 this will cause the planner to try the first
    // TravelToPark method, then backtrack when the precondition for Walk is not met,
    // then try the second method, which succeeds (get a taxi).
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
        preconditions: [distance_to_park <= 4, my_location != Location::Park, happy == false]
        effects: [
            my_location = Location::Park,
            happy = true,
        ]
    }

    compound_task "Taxi" {
        method {
            subtasks: [CallTaxi, RideTaxi, PayTaxi]
        }
    }

    primitive_task "CallTaxi" {
        operator: TaxiOperator
        preconditions: [cash >= 1]
        effects: [taxi_location = my_location]
    }

    primitive_task "RideTaxi" {
        operator: RideTaxiOperator(distance_to_park)
        preconditions: [taxi_location == my_location, cash >= 1]
        effects: [taxi_location = Location::Park, my_location = Location::Park, happy = true]
    }

    primitive_task "PayTaxi" {
        operator: PayTaxiOperator
        preconditions: [taxi_location == Location::Park, cash >= 1]
        effects: [cash -= 1]
    }
    "#;

    // REGISTER TYPES USED IN HTN

    // normally you'd use app.register_type or Res<AppTypeRegistry>
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
    match htn.verify_all(&TravelState::default(), &atr) {
        Ok(_) => {}
        Err(e) => panic!("HTN type verification failed: {e:?}"),
    }

    let mut planner = HtnPlanner::new(&htn, &atr);

    // Run the planner with alternative starting states to see different outcomes:
    {
        warn!("Testing walking state");
        let initial_state = TravelState {
            cash: 10,
            distance_to_park: 1,
            my_location: Location::Home,
            taxi_location: Location::Other,
            happy: false,
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
            happy: false,
        };
        let plan = planner.plan(&initial_state);
        assert_eq!(plan.task_names(), vec!["CallTaxi", "RideTaxi", "PayTaxi"]);
    }
}

#[test]
fn test_conditions() {
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

    #[derive(Reflect, Resource, Clone, Debug, Default, Component)]
    #[reflect(Default, Resource)]
    struct State {
        energy: i32,
        happy: bool,
        location: Location,
        e1: i32,
        e2: i32,
    }

    let src = r#"
    schema {
        version: 0.1.0
    }
            
    primitive_task "Conditions Test" {
        operator: DummyOperator
        preconditions: [
            energy > 10,
            energy <= 100,
            location != Location::Park,
            happy == false,
            e1 != e2,
        ]
        effects: [
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
    let state = State::default();
    assert!(htn.verify_without_operators(&state, &atr).is_ok());
    // info!("{htn:#?}");
    let Some(Task::Primitive(pt)) = &htn.tasks.first() else {
        panic!("Task should exist");
    };
    assert_eq!(
        pt.preconditions,
        vec![
            HtnCondition::GreaterThanInt {
                field: "energy".to_string(),
                threshold: 10,
                orequals: false,
                syntax: "energy > 10".to_string(),
            },
            HtnCondition::LessThanInt {
                field: "energy".to_string(),
                threshold: 100,
                orequals: true,
                syntax: "energy <= 100".to_string(),
            },
            HtnCondition::EqualsEnum {
                field: "location".to_string(),
                enum_type: "Location".to_string(),
                enum_variant: "Park".to_string(),
                notted: true,
                syntax: "location != Location::Park".to_string(),
            },
            HtnCondition::EqualsBool {
                field: "happy".to_string(),
                value: false,
                notted: false,
                syntax: "happy == false".to_string(),
            },
            HtnCondition::EqualsIdentifier {
                field: "e1".to_string(),
                other_field: "e2".to_string(),
                notted: true,
                syntax: "e1 != e2".to_string(),
            },
        ]
    );
    assert_eq!(pt.name, "Conditions Test");
    assert_eq!(pt.operator.name(), "DummyOperator");
    assert_eq!(pt.effects.len(), 0);
    assert_eq!(pt.expected_effects.len(), 0);

    let state = State {
        energy: 10,
        happy: false,
        location: Location::Home,
        e1: 1,
        e2: 2,
    };

    let condition = HtnCondition::EqualsBool {
        field: "happy".to_string(),
        value: false,
        notted: false,
        syntax: "happy == false".to_string(),
    };
    assert!(condition.evaluate(&state, &atr));

    let condition = HtnCondition::EqualsInt {
        field: "energy".to_string(),
        value: 10,
        notted: false,
        syntax: "energy == 10".to_string(),
    };
    assert!(condition.evaluate(&state, &atr));
    let state2 = State {
        energy: 999,
        ..state.clone()
    };
    assert!(!condition.evaluate(&state2, &atr));

    let condition = HtnCondition::GreaterThanInt {
        field: "energy".to_string(),
        threshold: 10,
        orequals: true,
        syntax: "energy >= 10".to_string(),
    };
    assert!(condition.evaluate(&state, &atr));

    let condition = HtnCondition::LessThanInt {
        field: "energy".to_string(),
        threshold: 10,
        orequals: false,
        syntax: "energy < 10".to_string(),
    };
    assert!(!condition.evaluate(&state, &atr));

    let condition = HtnCondition::EqualsEnum {
        field: "location".to_string(),
        enum_type: "Location".to_string(),
        enum_variant: "Park".to_string(),
        notted: true,
        syntax: "location != Location::Park".to_string(),
    };
    assert!(condition.evaluate(&state, &atr));
    let state2 = State {
        location: Location::Park,
        ..state
    };
    assert!(!condition.evaluate(&state2, &atr));

    let condition = HtnCondition::EqualsIdentifier {
        field: "e1".to_string(),
        other_field: "e2".to_string(),
        notted: false,
        syntax: "e1 == e2".to_string(),
    };
    assert!(!condition.evaluate(&state, &atr));
    let state2 = State { e1: 2, ..state };
    assert!(condition.evaluate(&state2, &atr));
}

#[test]
fn test_effects() {
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

    #[derive(Reflect, Resource, Clone, Debug, Default, Component)]
    #[reflect(Default, Resource)]
    struct State {
        energy: i32,
        happy: bool,
        location: Location,
        e1: i32,
        e2: i32,
    }

    let src = r#"
    schema {
        version: 0.1.0
    }
            
    primitive_task "Effects Test" {
        operator: DummyOperator
        preconditions: []
        effects: [
            happy = true,
            energy = 200,
            e1 = e2,
            energy -= 50,
            location = Location::Park,
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
    let Some(Task::Primitive(pt)) = &htn.tasks.first() else {
        panic!("Task should exist");
    };
    assert_eq!(
        pt.effects,
        vec![
            Effect::SetBool {
                field: "happy".to_string(),
                value: true,
                syntax: "happy = true".to_string(),
            },
            Effect::SetInt {
                field: "energy".to_string(),
                value: 200,
                syntax: "energy = 200".to_string(),
            },
            Effect::SetIdentifier {
                field: "e1".to_string(),
                field_source: "e2".to_string(),
                syntax: "e1 = e2".to_string(),
            },
            Effect::IncrementInt {
                field: "energy".to_string(),
                by: -50,
                syntax: "energy -= 50".to_string(),
            },
            Effect::SetEnum {
                field: "location".to_string(),
                enum_type: "Location".to_string(),
                enum_variant: "Park".to_string(),
                syntax: "location = Location::Park".to_string(),
            },
        ]
    );

    let initial_state = State {
        energy: 10,
        happy: false,
        location: Location::Home,
        e1: 1,
        e2: 2,
    };

    let mut state = initial_state.clone();
    let effect = Effect::SetBool {
        field: "happy".to_string(),
        value: true,
        syntax: "happy = true".to_string(),
    };
    effect.apply(&mut state, &atr);
    assert!(state.happy);

    let mut state = initial_state.clone();
    let effect = Effect::SetInt {
        field: "energy".to_string(),
        value: 100,
        syntax: "energy = 100".to_string(),
    };
    effect.apply(&mut state, &atr);
    assert_eq!(state.energy, 100);

    let mut state = initial_state.clone();
    let effect = Effect::SetIdentifier {
        field: "e1".to_string(),
        field_source: "e2".to_string(),
        syntax: "e1 = e2".to_string(),
    };
    effect.apply(&mut state, &atr);
    assert_eq!(state.e1, 2);

    let mut state = initial_state.clone();
    let effect = Effect::SetEnum {
        field: "location".to_string(),
        enum_type: "Location".to_string(),
        enum_variant: "Park".to_string(),
        syntax: "location = Location::Park".to_string(),
    };
    effect.apply(&mut state, &atr);
    assert_eq!(state.location, Location::Park);

    let mut state = initial_state.clone();
    let effect = Effect::IncrementInt {
        field: "energy".to_string(),
        by: 10,
        syntax: "energy += 10".to_string(),
    };
    effect.apply(&mut state, &atr);
    assert_eq!(state.energy, 20);

    let mut state = initial_state.clone();
    let effect = Effect::IncrementInt {
        field: "energy".to_string(),
        by: -10,
        syntax: "energy -= 10".to_string(),
    };
    effect.apply(&mut state, &atr);
    assert_eq!(state.energy, 0);
}
