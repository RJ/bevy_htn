use bevy::{prelude::*, reflect::TypeRegistry};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use std::marker::PhantomData;

mod dsl;
mod htn;
use dsl::*;
use htn::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct GameState {
    pub gold: bool,
    pub energy: i32,
}

// ---------- Example Usage ----------

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::log::LogPlugin::default());
    app.register_type::<SellGold>();

    app.add_systems(Startup, startup);
    app.run();
}

fn startup(world: &mut World) {
    let app_type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = app_type_registry.read();
    let dsl = r#"
    task "Acquire Gold" {
        effect: set gold = true;
        effect: inc energy by 1;
    }
    task "Recharge" {
        effect: inc energy by 5;
        task "Find Energy Source" {
            precondition: gold == false;
            effect: set energy = 10;
        }
    }
    task "Sell Gold" {
        operator: SellGold(energy);
        precondition: gold == true;
        effect: set gold = false;
    }
    "#;

    // Here we specify that our HTN is for GameState.
    let htn = parse_htn::<GameState>(dsl);
    println!("Parsed HTN: {:#?}", htn);

    // Example execution of top-level tasks (subtask execution omitted for brevity):
    let mut state = GameState {
        gold: false,
        energy: 1,
    };
    println!("Initial state: {:#?}", state);
    for task in htn.tasks.iter() {
        if task.preconditions.iter().all(|c| c.evaluate(&state)) {
            info!("Executing task: {}", task.name);
            println!("State: {:#?}", state);
            let mut entity = world.spawn(());
            task.insert_operator(&state, &type_registry, &mut entity);
            let eid = entity.id();
            world.commands().entity(eid).log_components();

            for eff in task.effects.iter() {
                eff.apply(&mut state);
            }
        } else {
            println!("Skipping task: {}", task.name);
            println!("State: {:#?}", state);
        }
    }
    println!("Final state: {:#?}", state);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
struct SellGold {
    energy: i32,
}
