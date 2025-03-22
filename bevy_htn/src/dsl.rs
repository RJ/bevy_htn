use crate::{htn::*, HtnStateTrait};
use bevy::prelude::*;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "src/htn.pest"]
pub struct HtnParser;

// inner = [Pair { rule: value_condition, span: Span { str: "energy > 10", start: 228, end: 239 }
// inner: [Pair { rule: identifier, span: Span { str: "energy", start: 228, end: 234 }, inner: [] },
// Pair { rule: operator, span: Span { str: ">", start: 235, end: 236 }, inner: [] },
// Pair { rule: value, span: Span { str: "10", start: 237, end: 239 }, inner: [] }] }]
// ' panicked at bevy_htn/src/dsl.rs2025-03-21T21:40:04.408699Z  INFO bevy_htn::dsl: field = energy > 10

fn parse_condition(pair: Pair<Rule>) -> HtnCondition {
    let op = pair.into_inner().next().unwrap();
    let rule = op.as_rule();
    let inner_pairs = op.into_inner();
    match rule {
        Rule::value_condition => parse_value_condition(inner_pairs),
        Rule::option_condition => parse_option_condition(inner_pairs),
        _ => panic!("Invalid condition {}", inner_pairs.as_str()),
    }
}

fn parse_value_condition(mut pairs: Pairs<Rule>) -> HtnCondition {
    let syntax = pairs.as_str().to_string();
    // eg:  foo >= 10
    let field = pairs.next().unwrap().as_str().to_string(); // foo
    let op = pairs.next().unwrap().as_rule(); // Rule::op_gte
    let value = pairs.next().unwrap();
    let val_rule = value.as_rule(); // Rule::value
    let val_str = value.as_str(); // 10

    match op {
        // >, >= of value
        Rule::op_gte | Rule::op_gt if val_rule == Rule::value => {
            let threshold = val_str.parse::<i32>().expect("Invalid number in condition");
            HtnCondition::GreaterThanInt {
                field,
                threshold,
                orequals: op == Rule::op_gte,
                syntax,
            }
        }
        // <, <= of value
        Rule::op_lte | Rule::op_lt if val_rule == Rule::value => {
            let threshold = val_str.parse::<i32>().expect("Invalid number in condition");
            HtnCondition::LessThanInt {
                field,
                threshold,
                orequals: op == Rule::op_lte,
                syntax,
            }
        }
        // >, >= of identifier
        Rule::op_gte | Rule::op_gt if val_rule == Rule::identifier => {
            HtnCondition::GreaterThanIdentifier {
                field,
                other_field: val_str.to_string(),
                orequals: op == Rule::op_gte,
                syntax,
            }
        }
        // <, <= of identifier
        Rule::op_lte | Rule::op_lt if val_rule == Rule::identifier => {
            HtnCondition::LessThanIdentifier {
                field,
                other_field: val_str.to_string(),
                orequals: op == Rule::op_lte,
                syntax,
            }
        }
        // equality of bool
        Rule::op_eq | Rule::op_neq
            if val_rule == Rule::value && (val_str == "true" || val_str == "false") =>
        {
            let notted = op == Rule::op_neq;
            let bool_val = match val_str {
                "true" => true,
                "false" => false,
                _ => unreachable!(),
            };
            HtnCondition::EqualsBool {
                field,
                value: bool_val,
                notted,
                syntax,
            }
        }
        // equality of i32
        Rule::op_eq | Rule::op_neq if val_rule == Rule::value => {
            let notted = op == Rule::op_neq;
            if let Ok(int_val) = val_str.parse::<i32>() {
                HtnCondition::EqualsInt {
                    field,
                    value: int_val,
                    notted,
                    syntax,
                }
            } else {
                panic!("Invalid integer value: {}", val_str);
            }
        }
        // equality of enum
        Rule::op_eq | Rule::op_neq if val_rule == Rule::enum_value => {
            let notted = op == Rule::op_neq;
            // safety: parser ensures a well formed enum containing ::
            let parts: Vec<&str> = val_str.split("::").collect();
            let enum_type = parts[0].to_string();
            let enum_variant = parts[1].to_string();
            HtnCondition::EqualsEnum {
                field,
                enum_type,
                enum_variant,
                notted,
                syntax,
            }
        }
        // equality of identifier
        Rule::op_eq | Rule::op_neq if val_rule == Rule::identifier => {
            let notted = op == Rule::op_neq;
            HtnCondition::EqualsIdentifier {
                field,
                other_field: val_str.to_string(),
                notted,
                syntax,
            }
        }

        _ => panic!("Unsupported operator: {:?}", op),
    }
}

fn parse_option_condition(mut pairs: Pairs<Rule>) -> HtnCondition {
    let syntax = pairs.as_str().to_string();
    info!("parse_option_condition: {syntax}");
    // eg:  foo is None 10
    let field = pairs.next().unwrap().as_str().to_string(); // foo
    let op = pairs.next().unwrap().as_str(); // is
    assert_eq!(op, "is");
    let val_str = pairs.next().unwrap().as_str(); // None or Some
    match val_str {
        "None" => HtnCondition::EqualsNone { field, syntax },
        "Some" => HtnCondition::EqualsSome { field, syntax },
        _ => panic!("Invalid value for 'is' operator: {syntax}"),
    }
}

fn parse_effect(pair: Pair<Rule>) -> Effect {
    let syntax = pair.as_str().to_string();
    let inner_pair = pair.into_inner().next().unwrap();
    let rule = inner_pair.as_rule();
    match rule {
        Rule::set_effect => {
            let mut parts = inner_pair.into_inner();
            let field = parts.next().unwrap().as_str().to_string();
            let val_str = parts.next().unwrap().as_str();

            if val_str == "true" || val_str == "false" {
                let bool_val = val_str == "true";
                Effect::SetBool {
                    field,
                    value: bool_val,
                    syntax,
                }
            } else if let Ok(int_val) = val_str.parse::<i32>() {
                Effect::SetInt {
                    field,
                    value: int_val,
                    syntax,
                }
            } else if val_str.contains("::") {
                let parts: Vec<&str> = val_str.split("::").collect();
                let enum_type = parts[0].to_string();
                let enum_variant = parts[1].to_string();
                Effect::SetEnum {
                    field,
                    enum_type,
                    enum_variant,
                    syntax,
                }
            } else {
                let identifier = val_str.to_string();
                Effect::SetIdentifier {
                    field,
                    field_source: identifier,
                    syntax,
                }
            }
        }
        Rule::inc_effect | Rule::dec_effect => {
            let mut parts = inner_pair.into_inner();
            let field = parts.next().unwrap().as_str().to_string();
            let amt_str = parts.next().unwrap().as_str();
            let amount = amt_str
                .parse::<i32>()
                .expect("Invalid integer in inc effect");
            if rule == Rule::dec_effect {
                Effect::IncrementInt {
                    field,
                    by: -amount,
                    syntax,
                }
            } else {
                Effect::IncrementInt {
                    field,
                    by: amount,
                    syntax,
                }
            }
        }
        _ => panic!("Unsupported effect type"),
    }
}

fn parse_primitive_task<T: HtnStateTrait>(pair: Pair<Rule>) -> PrimitiveTask<T> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().trim_matches('"').to_string();
    let mut builder = PrimitiveTaskBuilder::<T>::new(name);

    for stmt in inner {
        match stmt.as_rule() {
            Rule::operator_statement => {
                let mut op_inner = stmt.into_inner();
                let op_def = op_inner.next().unwrap();
                let mut op_parts = op_def.into_inner();
                let op_name = op_parts.next().unwrap().as_str().to_string();
                let params: Vec<String> =
                    op_parts.map(|param| param.as_str().to_string()).collect();
                bevy::log::warn!("params = {params:?}");
                builder = builder.operator(Operator::Trigger {
                    name: op_name,
                    params,
                });
            }
            Rule::effects_statement => {
                let effects = stmt
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::effect)
                    .map(|p| parse_effect(p))
                    .collect::<Vec<_>>();

                for effect in effects {
                    builder = builder.effect(effect);
                }
            }
            Rule::expected_effects_statement => {
                let effects = stmt
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::effect)
                    .map(|p| parse_effect(p))
                    .collect::<Vec<_>>();

                for effect in effects {
                    builder = builder.expected_effect(effect);
                }
            }
            Rule::preconditions_statement => {
                let conditions = stmt
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::condition)
                    .map(|p| parse_condition(p))
                    .collect::<Vec<_>>();

                for condition in conditions {
                    builder = builder.precondition(condition);
                }
            }
            _ => {}
        }
    }

    builder.build()
}

fn parse_method<T: HtnStateTrait>(pair: Pair<Rule>) -> Method<T> {
    let mut builder = MethodBuilder::<T>::new();
    let mut inner = pair.into_inner().peekable();

    // Optional method name
    if let Some(pair) = inner.peek() {
        if pair.as_rule() == Rule::STRING {
            let name = inner.next().unwrap().as_str().trim_matches('"').to_string();
            builder = builder.name(name);
        }
    }

    for stmt in inner {
        match stmt.as_rule() {
            Rule::preconditions_statement => {
                let conditions = stmt
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::condition)
                    .map(|p| parse_condition(p))
                    .collect::<Vec<_>>();

                for condition in conditions {
                    builder = builder.precondition(condition);
                }
            }
            Rule::subtasks_statement => {
                let subtasks = stmt
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::identifier)
                    .map(|p| p.as_str().to_string())
                    .collect::<Vec<_>>();

                for subtask in subtasks {
                    builder = builder.subtask(subtask);
                }
            }
            _ => {}
        }
    }
    builder.build()
}

fn parse_compound_task<T: HtnStateTrait>(pair: Pair<Rule>) -> CompoundTask<T> {
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

fn parse_schema(pair: Pair<Rule>) -> HtnSchema {
    let mut inner_rules = pair.into_inner();
    let schema_version_statement = inner_rules.next().unwrap();
    if schema_version_statement.as_rule() == Rule::schema_version_statement {
        let version_pair = schema_version_statement.into_inner().next().unwrap();
        if version_pair.as_rule() == Rule::SEMVER {
            let version = version_pair.as_str().to_string();
            HtnSchema { version }
        } else {
            panic!("Invalid version: {}", version_pair.as_str());
        }
    } else {
        panic!(
            "Expected schema_version_statement, found: {}",
            schema_version_statement.as_str()
        );
    }
}

// TODO error handling. return Result..
pub fn parse_htn<T: HtnStateTrait>(input: &str) -> HTN<T> {
    let pairs = HtnParser::parse(Rule::domain, input).expect("Failed to parse DSL");
    let mut htn_builder = HTN::<T>::builder();

    let htn_pair = pairs.into_iter().next().unwrap();
    for pair in htn_pair.into_inner() {
        match pair.as_rule() {
            Rule::schema => {
                let meta = parse_schema(pair);
                htn_builder = htn_builder.schema(meta);
            }
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
