# Bevy Hierarchical Task Network (UNRELEASED)

Working on an HTN that uses [bevy_behave](https://github.com/RJ/bevy_behave) behaviour trees as
operators, and can hot reload the `.htn` definition via the asset server.

 `test.rs` best place to see it in action for now.

 RJ in bevy #ai discord.

```rust
#[derive(Reflect, Resource, Clone, Debug, Default)]
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

let htn = parse_htn::<TravelState>(src);
// verify via reflection that any types used in the htn are registered:
htn.verify_all(&TravelState::default(), &atr)?;

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
```

### ...
Need sensors that project the ECS state onto the simplified planner state struct, which can just be a component of the NPC.

PLanning can be triggered when:
* no plan present
* plan finished
* world state changed

## plan runner

desribe the entity hierarchy and marker components used to host the htn / subtrees.

# behave.

Operators are [bevy_behave](https://github.com/RJ/bevy_behave) behaviour trees.

# .htn Capabilities

* ..

## FAQ

### How do i set planner goals?

You can't really set a goal state and have the planner search for it like with GOAP.
The goal state of an HTN is kind of encoded into the task definitions.
Design your tasks to elicit the desired state.

Might be interesting to add a backwards goal-state based planner like goap atop the htn structure, but it would be slow compared to the forwards HTN planning approach, and i want to have loads of NPC entities.

## TODO

while a plan is running, if gamestate changes, verify all preconditions of current and remaining tasks in the plan still pass,
and if not, fail the plan.

### Reading

* [Exploring HTN Planners through Example](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter12_Exploring_HTN_Planners_through_Example.pdf) from GameAIPro
* [Building a Planner: A Survey of Planning Systems Used in Commercial Video Games](https://www.is.ovgu.de/is_media/Research/Publications/IEEE_ToG_2018_Neufeld-p-4450.pdf)
* [A Hybrid Approach to Planning and Execution in Dynamic Environments Through Hierarchical Task Networks and Behavior Trees](https://cdn.aaai.org/ojs/13044/13044-52-16561-1-2-20201228.pdf) (meh)
* [bevy_mod_scripting is an example of some reflection and dynamic shennanigans](https://github.com/makspll/bevy_mod_scripting/blob/a4d1ffbcae98f42393ab447d73efe9b0b543426f/crates/bevy_mod_scripting_core/src/bindings/world.rs#L642). A useful reference.