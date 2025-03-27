use crate::{error::HtnErr, htn::*, HtnStateTrait};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "src/htn.pest"]
pub struct HtnParser;

fn parse_f32(val_str: &str, context: &str) -> Result<f32, HtnErr> {
    val_str.parse::<f32>().map_err(|_| HtnErr::Float {
        syntax: val_str.to_string(),
        details: format!("Invalid float `{val_str}` in: `{context}`"),
    })
}

fn parse_i32(val_str: &str, context: &str) -> Result<i32, HtnErr> {
    val_str.parse::<i32>().map_err(|_| HtnErr::Int {
        syntax: val_str.to_string(),
        details: format!("Invalid integer `{val_str}` in: `{context}`"),
    })
}

fn parse_bool(val_str: &str, context: &str) -> Result<bool, HtnErr> {
    match val_str {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(HtnErr::Bool {
            syntax: val_str.to_string(),
            details: format!("Invalid boolean `{val_str}` in: `{context}`"),
        }),
    }
}

fn parse_enum(val_str: &str, context: &str) -> Result<(String, String), HtnErr> {
    let parts: Vec<&str> = val_str.split("::").collect();
    if parts.len() != 2 {
        return Err(HtnErr::Enum {
            syntax: val_str.to_string(),
            details: format!("Invalid enum `{val_str}` in: `{context}`"),
        });
    }
    let enum_type = parts[0].to_string();
    let enum_variant = parts[1].to_string();
    if enum_type.is_empty() || enum_variant.is_empty() {
        return Err(HtnErr::Enum {
            syntax: val_str.to_string(),
            details: format!("Invalid enum `{val_str}` in: `{context}`"),
        });
    }
    Ok((enum_type, enum_variant))
}

fn parse_condition(pair: Pair<Rule>) -> Result<HtnCondition, HtnErr> {
    let syntax = pair.as_str().to_string();
    let mut pairs = pair.into_inner();
    // eg:  foo >= 10
    let field = pairs.next().unwrap().as_str().to_string(); // "foo"
    let op = pairs.next().unwrap().as_rule(); // Rule::op_gte
    let value = pairs.next().unwrap();
    let val_rule = value.as_rule(); // Rule::int_value
    let val_str = value.as_str(); // "10"

    let notted = op == Rule::op_neq;

    let condition = match (op, val_rule) {
        // >, >= of INT value
        (Rule::op_gte | Rule::op_gt, Rule::int_value) => HtnCondition::GreaterThanInt {
            field,
            threshold: parse_i32(val_str, &syntax)?,
            orequals: op == Rule::op_gte,
            syntax,
        },
        // <, <= of INT value
        (Rule::op_lte | Rule::op_lt, Rule::int_value) => HtnCondition::LessThanInt {
            field,
            threshold: parse_i32(val_str, &syntax)?,
            orequals: op == Rule::op_lte,
            syntax,
        },
        // >, >= of F32 value
        (Rule::op_gte | Rule::op_gt, Rule::float_value) => HtnCondition::GreaterThanFloat {
            field,
            threshold: parse_f32(val_str, &syntax)?,
            orequals: op == Rule::op_gte,
            syntax,
        },
        // <, <= of F32 value
        (Rule::op_lte | Rule::op_lt, Rule::float_value) => HtnCondition::LessThanFloat {
            field,
            threshold: parse_f32(val_str, &syntax)?,
            orequals: op == Rule::op_lte,
            syntax,
        },
        // >, >= of identifier
        (Rule::op_gte | Rule::op_gt, Rule::identifier) => HtnCondition::GreaterThanIdentifier {
            field,
            other_field: val_str.to_string(),
            orequals: op == Rule::op_gte,
            syntax,
        },
        // <, <= of identifier
        (Rule::op_lte | Rule::op_lt, Rule::identifier) => HtnCondition::LessThanIdentifier {
            field,
            other_field: val_str.to_string(),
            orequals: op == Rule::op_lte,
            syntax,
        },
        // equality of bool
        (Rule::op_eq | Rule::op_neq, Rule::bool_value) => HtnCondition::EqualsBool {
            field,
            value: parse_bool(val_str, &syntax)?,
            notted,
            syntax,
        },
        // equality of None
        (Rule::op_eq | Rule::op_neq, Rule::none_value) => HtnCondition::EqualsNone {
            field,
            notted,
            syntax,
        },
        // equality of i32
        (Rule::op_eq | Rule::op_neq, Rule::int_value) => HtnCondition::EqualsInt {
            field,
            value: parse_i32(val_str, &syntax)?,
            notted,
            syntax,
        },
        // equality of f32
        (Rule::op_eq | Rule::op_neq, Rule::float_value) => HtnCondition::EqualsFloat {
            field,
            value: parse_f32(val_str, &syntax)?,
            notted,
            syntax,
        },
        // equality of enum
        (Rule::op_eq | Rule::op_neq, Rule::enum_value) => {
            let (enum_type, enum_variant) = parse_enum(val_str, &syntax)?;
            HtnCondition::EqualsEnum {
                field,
                enum_type,
                enum_variant,
                notted,
                syntax,
            }
        }
        // equality of identifier
        (Rule::op_eq | Rule::op_neq, Rule::identifier) => HtnCondition::EqualsIdentifier {
            field,
            other_field: val_str.to_string(),
            notted,
            syntax,
        },
        _ => {
            return Err(HtnErr::Condition {
                syntax: syntax.clone(),
                details: format!("Unsupported condition `{syntax}`"),
            })
        }
    };
    Ok(condition)
}

fn parse_effect(pair: Pair<Rule>) -> Result<Effect, HtnErr> {
    let syntax = pair.as_str().to_string();
    // let inner_pair = pair.into_inner().next().unwrap();
    let effect_pair = pair.into_inner().next().unwrap();
    let effect_rule = effect_pair.as_rule(); // Rule::set_effect / inc_effect / etc
    let mut parts = effect_pair.into_inner();
    // EG: foo = 10
    // the LHS state field name, ie "foo"
    let field = parts.next().unwrap().as_str().to_string();
    // the RHS:
    let val_pair = parts.next().unwrap();
    let val_rule = val_pair.as_rule(); // Rule::int_value
    let val_str = val_pair.as_str(); // "10"
    let effect = match (effect_rule, val_rule) {
        (Rule::set_effect_literal, Rule::bool_value) => Effect::SetBool {
            field,
            value: parse_bool(val_str, &syntax)?,
            syntax,
        },
        (Rule::set_effect_literal, Rule::int_value) => Effect::SetInt {
            field,
            value: parse_i32(val_str, &syntax)?,
            syntax,
        },
        (Rule::set_effect_literal, Rule::float_value) => Effect::SetFloat {
            field,
            value: parse_f32(val_str, &syntax)?,
            syntax,
        },
        (Rule::set_effect_literal, Rule::enum_value) => {
            let (enum_type, enum_variant) = parse_enum(val_str, &syntax)?;
            Effect::SetEnum {
                field,
                enum_type,
                enum_variant,
                syntax,
            }
        }
        (Rule::set_effect_literal, Rule::none_value) => Effect::SetNone { field, syntax },
        (Rule::set_effect_identifier, Rule::identifier) => Effect::SetIdentifier {
            field,
            field_source: val_str.to_string(),
            syntax,
        },
        (Rule::set_effect_inc_literal, Rule::int_value) => Effect::IncrementInt {
            field,
            by: parse_i32(val_str, &syntax)?,
            syntax,
        },
        (Rule::set_effect_dec_literal, Rule::int_value) => Effect::IncrementInt {
            field,
            by: -parse_i32(val_str, &syntax)?,
            syntax,
        },
        (Rule::set_effect_inc_identifier, Rule::identifier) => Effect::IncrementIdentifier {
            field,
            field_source: val_str.to_string(),
            decrement: false,
            syntax,
        },
        (Rule::set_effect_dec_identifier, Rule::identifier) => Effect::IncrementIdentifier {
            field,
            field_source: val_str.to_string(),
            decrement: true,
            syntax,
        },
        _ => {
            return Err(HtnErr::Effect {
                syntax: syntax.clone(),
                details: format!("Unsupported effect type: `{syntax}`"),
            })
        }
    };
    Ok(effect)
}

fn parse_primitive_task<T: HtnStateTrait>(pair: Pair<Rule>) -> Result<PrimitiveTask<T>, HtnErr> {
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
                    .collect::<Result<Vec<_>, _>>()?;

                for effect in effects {
                    builder = builder.effect(effect);
                }
            }
            Rule::expected_effects_statement => {
                let effects = stmt
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::effect)
                    .map(|p| parse_effect(p))
                    .collect::<Result<Vec<_>, _>>()?;

                for effect in effects {
                    builder = builder.expected_effect(effect);
                }
            }
            Rule::preconditions_statement => {
                let conditions = stmt
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::condition)
                    .map(|p| parse_condition(p))
                    .collect::<Result<Vec<_>, _>>()?;

                for condition in conditions {
                    builder = builder.precondition(condition);
                }
            }
            _ => {}
        }
    }

    Ok(builder.build())
}

fn parse_method<T: HtnStateTrait>(pair: Pair<Rule>) -> Result<Method<T>, HtnErr> {
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
                    .collect::<Result<Vec<_>, _>>()?;

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
    Ok(builder.build())
}

fn parse_compound_task<T: HtnStateTrait>(pair: Pair<Rule>) -> Result<CompoundTask<T>, HtnErr> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().trim_matches('"').to_string();
    let mut builder = CompoundTaskBuilder::<T>::new(name);

    for method_pair in inner {
        if method_pair.as_rule() == Rule::method {
            let method = parse_method::<T>(method_pair)?;
            builder = builder.method(method);
        }
    }

    Ok(builder.build())
}

fn parse_schema(pair: Pair<Rule>) -> Result<HtnSchema, HtnErr> {
    let mut inner_rules = pair.into_inner();
    let ver = inner_rules.next().unwrap();
    if ver.as_rule() == Rule::schema_version_statement {
        let version_pair = ver.into_inner().next().unwrap();
        if version_pair.as_rule() == Rule::SEMVER {
            let version = version_pair.as_str().to_string();
            Ok(HtnSchema { version })
        } else {
            Err(HtnErr::Schema {
                details: format!(
                    "Invalid version field `{}` in htn schema",
                    version_pair.as_str()
                ),
            })
        }
    } else {
        Err(HtnErr::Schema {
            details: format!(
                "Expected version field in htn schema, found: `{}`",
                ver.as_str()
            ),
        })
    }
}

pub fn parse_htn<T: HtnStateTrait>(input: &str) -> Result<HTN<T>, HtnErr> {
    let pairs = HtnParser::parse(Rule::domain, input).map_err(|e| HtnErr::ParserError {
        details: e.to_string(),
    })?;
    let mut htn_builder = HTN::<T>::builder();

    let htn_pair = pairs.into_iter().next().unwrap();
    for pair in htn_pair.into_inner() {
        match pair.as_rule() {
            Rule::schema => {
                let meta = parse_schema(pair)?;
                htn_builder = htn_builder.schema(meta);
            }
            Rule::primitive_task => {
                let task = parse_primitive_task::<T>(pair)?;
                htn_builder = htn_builder.primitive_task(task);
            }
            Rule::compound_task => {
                let task = parse_compound_task::<T>(pair)?;
                htn_builder = htn_builder.compound_task(task);
            }
            _ => {}
        }
    }

    Ok(htn_builder.build())
}
