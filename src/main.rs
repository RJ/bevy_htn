use bevy::{prelude::*, reflect::TypeRegistry};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use std::marker::PhantomData;

// ---------- HTN Builder API ----------

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct GameState {
    pub gold: bool,
    pub energy: i32,
}

#[derive(Clone, Debug, Reflect)]
pub enum Condition {
    EqualsBool { field: String, value: bool },
    GreaterThanInt { field: String, threshold: i32 },
}

impl Condition {
    pub fn evaluate<T: Reflect>(&self, state: &T) -> bool {
        let reflected = state
            .reflect_ref()
            .as_struct()
            .expect("State is not a struct");
        match self {
            Condition::EqualsBool { field, value } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(b) = val.try_downcast_ref::<bool>() {
                        *b == *value
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Condition::GreaterThanInt { field, threshold } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(i) = val.try_downcast_ref::<i32>() {
                        *i > *threshold
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub enum Effect {
    SetBool { field: String, value: bool },
    SetInt { field: String, value: i32 },
    IncrementInt { field: String, by: i32 },
}

impl Effect {
    pub fn apply<T: Reflect>(&self, state: &mut T) {
        let reflected = state
            .reflect_mut()
            .as_struct()
            .expect("State is not a struct");
        match self {
            Effect::SetBool { field, value } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(b) = val.try_downcast_mut::<bool>() {
                        *b = *value;
                    }
                }
            }
            Effect::SetInt { field, value } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(i) = val.try_downcast_mut::<i32>() {
                        *i = *value;
                    }
                }
            }
            Effect::IncrementInt { field, by } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(i) = val.try_downcast_mut::<i32>() {
                        *i += *by;
                    }
                }
            }
        }
    }
}

// Make Task generic over any state type T that implements Reflect.
#[derive(Clone, Debug, Reflect)]
pub struct Task<T: Reflect> {
    pub name: String,
    pub operator_type: Option<String>,
    pub operator_params: Vec<String>,
    pub preconditions: Vec<Condition>,
    pub effects: Vec<Effect>,
    pub subtasks: Vec<Task<T>>,
    _phantom: PhantomData<T>,
}

impl<T: Reflect> Task<T> {
    pub fn builder(name: impl Into<String>) -> TaskBuilder<T> {
        TaskBuilder {
            name: name.into(),
            operator_type: None,
            operator_params: Vec::new(),
            preconditions: Vec::new(),
            effects: Vec::new(),
            subtasks: Vec::new(),
        }
    }

    pub fn create_operator(
        &self,
        state: &T,
        type_registry: &TypeRegistry,
    ) -> Option<Box<dyn Reflect>> {
        let op_type = self.operator_type.as_ref()?;

        let registration = type_registry.get_with_name(op_type)?;

        let mut component = registration.data::<ReflectComponent>()?.spawn();

        if let Some(param_field) = self.operator_params.first() {
            if let Some(state_val) = state
                .reflect_ref()
                .as_struct()
                .and_then(|s| s.field(param_field))
            {
                if let Some(tuple) = component.as_reflect_mut().as_tuple_struct_mut() {
                    if let Some(field) = tuple.field_mut(0) {
                        field.apply(state_val);
                    }
                }
            }
        }

        Some(component)
    }
}

pub struct TaskBuilder<T: Reflect> {
    name: String,
    operator_type: Option<String>,
    operator_params: Vec<String>,
    preconditions: Vec<Condition>,
    effects: Vec<Effect>,
    subtasks: Vec<Task<T>>,
}

impl<T: Reflect> TaskBuilder<T> {
    pub fn operator(mut self, op_type: impl Into<String>, params: Vec<String>) -> Self {
        self.operator_type = Some(op_type.into());
        self.operator_params = params;
        self
    }
    pub fn precondition(mut self, cond: Condition) -> Self {
        self.preconditions.push(cond);
        self
    }
    pub fn effect(mut self, eff: Effect) -> Self {
        self.effects.push(eff);
        self
    }
    pub fn subtask(mut self, task: Task<T>) -> Self {
        self.subtasks.push(task);
        self
    }
    pub fn build(self) -> Task<T> {
        Task {
            name: self.name,
            operator_type: self.operator_type,
            operator_params: self.operator_params,
            preconditions: self.preconditions,
            effects: self.effects,
            subtasks: self.subtasks,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct HTN<T: Reflect> {
    pub tasks: Vec<Task<T>>,
}

impl<T: Reflect> HTN<T> {
    pub fn builder() -> HTNBuilder<T> {
        HTNBuilder { tasks: Vec::new() }
    }
}

pub struct HTNBuilder<T: Reflect> {
    tasks: Vec<Task<T>>,
}

impl<T: Reflect> HTNBuilder<T> {
    pub fn task(mut self, task: Task<T>) -> Self {
        self.tasks.push(task);
        self
    }
    pub fn build(self) -> HTN<T> {
        HTN { tasks: self.tasks }
    }
}

// ---------- DSL Parser using Pest ----------

#[derive(Parser)]
#[grammar = "src/htn.pest"]
pub struct HtnParser;

fn parse_condition(pair: Pair<Rule>) -> Condition {
    // 'condition' rule: identifier operator value
    let mut inner = pair.into_inner();
    let field = inner.next().unwrap().as_str().to_string();
    let op = inner.next().unwrap().as_str();
    let val_str = inner.next().unwrap().as_str();
    match op {
        ">" => {
            let threshold = val_str.parse::<i32>().expect("Invalid number in condition");
            Condition::GreaterThanInt { field, threshold }
        }
        "==" => {
            let bool_val = match val_str {
                "true" => true,
                "false" => false,
                _ => panic!("Invalid boolean value"),
            };
            Condition::EqualsBool {
                field,
                value: bool_val,
            }
        }
        _ => panic!("Unsupported operator: {}", op),
    }
}

fn parse_effect(pair: Pair<Rule>) -> Effect {
    // 'effect' rule can be set_effect or inc_effect
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::set_effect => {
            let mut parts = inner_pair.into_inner();
            let field = parts.next().unwrap().as_str().to_string();
            let val_str = parts.next().unwrap().as_str();
            if val_str == "true" || val_str == "false" {
                let bool_val = val_str == "true";
                Effect::SetBool {
                    field,
                    value: bool_val,
                }
            } else {
                let int_val = val_str
                    .parse::<i32>()
                    .expect("Invalid integer in set effect");
                Effect::SetInt {
                    field,
                    value: int_val,
                }
            }
        }
        Rule::inc_effect => {
            let mut parts = inner_pair.into_inner();
            let field = parts.next().unwrap().as_str().to_string();
            let amt_str = parts.next().unwrap().as_str();
            let amount = amt_str
                .parse::<i32>()
                .expect("Invalid integer in inc effect");
            Effect::IncrementInt { field, by: amount }
        }
        _ => panic!("Unsupported effect type"),
    }
}

fn parse_task<T: Reflect>(pair: Pair<Rule>) -> Task<T> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().trim_matches('"').to_string();
    let mut builder = Task::<T>::builder(name);

    for stmt in inner {
        match stmt.as_rule() {
            Rule::operator_statement => {
                let op_def = stmt
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::operator_def)
                    .unwrap();
                let mut op_parts = op_def.into_inner();
                let op_name = op_parts.next().unwrap().as_str().to_string();
                let params: Vec<String> =
                    op_parts.map(|param| param.as_str().to_string()).collect();
                builder = builder.operator(op_name, params);
            }
            Rule::precondition_statement => {
                let condition = stmt
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::condition)
                    .unwrap();
                builder = builder.precondition(parse_condition(condition));
            }
            Rule::effect_statement => {
                let effect = stmt
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::effect)
                    .unwrap();
                builder = builder.effect(parse_effect(effect));
            }
            Rule::task => {
                let subtask = parse_task::<T>(stmt);
                builder = builder.subtask(subtask);
            }
            _ => {}
        }
    }
    builder.build()
}

pub fn parse_htn<T: Reflect>(input: &str) -> HTN<T> {
    let pairs = HtnParser::parse(Rule::htn, input).expect("Failed to parse DSL");
    let mut htn_builder = HTN::<T>::builder();

    // Get the first (and only) htn pair, then iterate through its tasks
    let htn_pair = pairs.into_iter().next().unwrap();
    for pair in htn_pair.into_inner() {
        if pair.as_rule() == Rule::task {
            let task = parse_task::<T>(pair);
            htn_builder = htn_builder.task(task);
        }
    }
    htn_builder.build()
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

fn startup(mut commands: Commands, type_registry: TypeRegistry) {
    let dsl = r#"
    task "Acquire Gold" {
        precondition: energy > 3;
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

    let entity = commands.spawn(()).id();

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
            if let Some(operator) = task.create_operator(&state, &type_registry) {
                let registration = type_registry
                    .get_with_short_type_path(task.operator_type.as_ref().unwrap())
                    .unwrap();

                let refcomp = registration.data::<ReflectComponent>().unwrap().clone();
                commands.queue(move |world: &mut World| {
                    refcomp.insert(world.get_entity_mut(entity), operator.as_ref());
                });

                // .insert(&mut commands.entity(entity), operator.as_ref());
            }

            for eff in task.effects.iter() {
                eff.apply(state);
            }
        }

        if task.preconditions.iter().all(|c| c.evaluate(&state)) {
            println!("Executing task: {}", task.name);
            for eff in task.effects.iter() {
                eff.apply(&mut state);
            }
        } else {
            println!("Skipping task: {}", task.name);
        }
    }
    println!("Final state: {:#?}", state);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct SellGold(i32);
