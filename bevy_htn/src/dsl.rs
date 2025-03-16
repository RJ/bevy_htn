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
            if val_str.contains("::") {
                let parts: Vec<&str> = val_str.split("::").collect();
                let enum_type = parts[0].to_string();
                let enum_variant = parts[1].to_string();
                HtnCondition::EqualsEnum {
                    field,
                    enum_type,
                    enum_variant,
                }
            } else {
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
        }
        _ => panic!("Unsupported operator: {}", op),
    }
}

fn parse_effect(pair: Pair<Rule>) -> Effect {
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
            } else if let Ok(int_val) = val_str.parse::<i32>() {
                Effect::SetInt {
                    field,
                    value: int_val,
                }
            } else if val_str.contains("::") {
                let parts: Vec<&str> = val_str.split("::").collect();
                let enum_type = parts[0].to_string();
                let enum_variant = parts[1].to_string();
                Effect::SetEnum {
                    field,
                    enum_type,
                    enum_variant,
                }
            } else {
                let identifier = val_str.to_string();
                Effect::SetIdentifier {
                    field,
                    value: identifier,
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

fn parse_primitive_task<T: Reflect>(pair: Pair<Rule>) -> PrimitiveTask<T> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().trim_matches('"').to_string();
    let mut builder = PrimitiveTaskBuilder::<T>::new(name);

    for stmt in inner {
        match stmt.as_rule() {
            Rule::operator_statement => {
                let mut op_inner = stmt.into_inner();
                let op_type = op_inner.next().unwrap();

                match op_type.as_rule() {
                    Rule::spawn_operator => {
                        let op_def = op_type.into_inner().next().unwrap();
                        let mut op_parts = op_def.into_inner();
                        let op_name = op_parts.next().unwrap().as_str().to_string();
                        let params: Vec<String> =
                            op_parts.map(|param| param.as_str().to_string()).collect();

                        builder = builder.operator(Operator::Spawn {
                            name: op_name,
                            params,
                        });
                    }
                    Rule::trigger_operator => {
                        let op_def = op_type.into_inner().next().unwrap();
                        let mut op_parts = op_def.into_inner();
                        let op_name = op_parts.next().unwrap().as_str().to_string();
                        let params: Vec<String> =
                            op_parts.map(|param| param.as_str().to_string()).collect();

                        builder = builder.operator(Operator::Trigger {
                            name: op_name,
                            params,
                        });
                    }
                    _ => unreachable!("Invalid operator type"),
                }
            }
            Rule::effect_statement => {
                let effect = stmt
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::effect)
                    .unwrap();
                builder = builder.effect(parse_effect(effect));
            }
            Rule::expected_effect_statement => {
                let effect = stmt
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::effect)
                    .unwrap();
                builder = builder.expected_effect(parse_effect(effect));
            }
            Rule::precondition_statement => {
                let condition = stmt
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::condition)
                    .unwrap();
                builder = builder.precondition(parse_condition(condition));
            }
            _ => {}
        }
    }

    builder.build()
}

fn parse_method<T: Reflect>(pair: Pair<Rule>) -> Method<T> {
    let mut builder = MethodBuilder::<T>::new();

    for stmt in pair.into_inner() {
        match stmt.as_rule() {
            Rule::precondition_statement => {
                let condition = stmt
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::condition)
                    .unwrap();
                builder = builder.precondition(parse_condition(condition));
            }
            Rule::subtask_statement => {
                let mut inner = stmt.into_inner();
                let task_name = inner.next().unwrap().as_str().to_string();
                builder = builder.subtask(task_name);
            }
            _ => {}
        }
    }

    builder.build()
}

fn parse_compound_task<T: Reflect + Default + TypePath + Clone + core::fmt::Debug>(
    pair: Pair<Rule>,
) -> CompoundTask<T> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().trim_matches('"').to_string();
    let mut builder = CompoundTaskBuilder::<T>::new(name);

    for method_pair in inner {
        if method_pair.as_rule() == Rule::method {
            let method = parse_method::<T>(method_pair);
            builder = builder.method(method);
        }
    }

    builder.build()
}

// TODO error handling
pub fn parse_htn<T: Reflect + Default + TypePath + Clone + core::fmt::Debug>(
    input: &str,
) -> HTN<T> {
    let pairs = HtnParser::parse(Rule::htn, input).expect("Failed to parse DSL");
    let mut htn_builder = HTN::<T>::builder();

    let htn_pair = pairs.into_iter().next().unwrap();
    for pair in htn_pair.into_inner() {
        match pair.as_rule() {
            Rule::primitive_task => {
                let task = parse_primitive_task::<T>(pair);
                htn_builder = htn_builder.primitive_task(task);
            }
            Rule::compound_task => {
                let task = parse_compound_task::<T>(pair);
                htn_builder = htn_builder.compound_task(task);
            }
            _ => {}
        }
    }

    htn_builder.build()
}
