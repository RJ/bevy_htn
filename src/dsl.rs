use crate::htn::*;
use bevy::prelude::*;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "src/htn.pest"]
pub struct HtnParser;

fn parse_condition(pair: Pair<Rule>) -> HtnCondition {
    // 'condition' rule: identifier operator value
    let mut inner = pair.into_inner();
    let field = inner.next().unwrap().as_str().to_string();
    let op = inner.next().unwrap().as_str();
    let val_str = inner.next().unwrap().as_str();
    match op {
        ">" => {
            let threshold = val_str.parse::<i32>().expect("Invalid number in condition");
            HtnCondition::GreaterThanInt { field, threshold }
        }
        "==" => {
            let bool_val = match val_str {
                "true" => true,
                "false" => false,
                _ => panic!("Invalid boolean value"),
            };
            HtnCondition::EqualsBool {
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
