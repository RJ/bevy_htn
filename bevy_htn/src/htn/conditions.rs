use crate::HtnStateTrait;

use super::*;
use bevy::{
    prelude::*,
    reflect::{DynamicEnum, DynamicVariant, VariantInfo},
};

#[derive(Clone, Debug, Reflect, PartialEq)]
pub enum HtnCondition {
    IsNone {
        field: String,
        syntax: String,
    },
    IsSome {
        field: String,
        syntax: String,
    },
    EqualsBool {
        field: String,
        value: bool,
        notted: bool,
        syntax: String,
    },
    GreaterThanInt {
        field: String,
        threshold: i32,
        orequals: bool,
        syntax: String,
    },
    GreaterThanFloat {
        field: String,
        threshold: f32,
        orequals: bool,
        syntax: String,
    },
    GreaterThanIdentifier {
        field: String,
        other_field: String,
        orequals: bool,
        syntax: String,
    },
    LessThanInt {
        field: String,
        threshold: i32,
        orequals: bool,
        syntax: String,
    },
    LessThanFloat {
        field: String,
        threshold: f32,
        orequals: bool,
        syntax: String,
    },
    LessThanIdentifier {
        field: String,
        other_field: String,
        orequals: bool,
        syntax: String,
    },
    EqualsEnum {
        field: String,
        enum_type: String,
        enum_variant: String,
        notted: bool,
        syntax: String,
    },
    EqualsInt {
        field: String,
        value: i32,
        notted: bool,
        syntax: String,
    },
    EqualsFloat {
        field: String,
        value: f32,
        notted: bool,
        syntax: String,
    },
    EqualsIdentifier {
        field: String,
        other_field: String,
        notted: bool,
        syntax: String,
    },
}

impl HtnCondition {
    pub fn syntax(&self) -> String {
        match self {
            HtnCondition::EqualsBool { syntax, .. } => syntax.clone(),
            HtnCondition::GreaterThanInt { syntax, .. } => syntax.clone(),
            HtnCondition::GreaterThanIdentifier { syntax, .. } => syntax.clone(),
            HtnCondition::LessThanInt { syntax, .. } => syntax.clone(),
            HtnCondition::LessThanIdentifier { syntax, .. } => syntax.clone(),
            HtnCondition::EqualsEnum { syntax, .. } => syntax.clone(),
            HtnCondition::EqualsInt { syntax, .. } => syntax.clone(),
            HtnCondition::EqualsIdentifier { syntax, .. } => syntax.clone(),
            HtnCondition::IsNone { syntax, .. } => syntax.clone(),
            HtnCondition::IsSome { syntax, .. } => syntax.clone(),
            HtnCondition::EqualsFloat { syntax, .. } => syntax.clone(),
            HtnCondition::GreaterThanFloat { syntax, .. } => syntax.clone(),
            HtnCondition::LessThanFloat { syntax, .. } => syntax.clone(),
        }
    }
    fn verify_field_type<FieldType: 'static>(
        state_struct: &dyn Struct,
        field: &str,
        syntax: &str,
    ) -> Result<(), String> {
        let Some(val) = state_struct.field(field) else {
            return Err(format!(
                "Unknown state field `{field}` for condition `{syntax}`"
            ));
        };
        if val.try_downcast_ref::<FieldType>().is_none() {
            return Err(format!(
                "State field `{field}` for condition `{syntax}` is not a {}",
                std::any::type_name::<FieldType>()
            ));
        }
        Ok(())
    }
    pub fn verify_types<T: HtnStateTrait>(
        &self,
        state: &T,
        _atr: &AppTypeRegistry,
    ) -> Result<(), String> {
        let reflected = state
            .reflect_ref()
            .as_struct()
            .expect("State is not a struct");
        match self {
            HtnCondition::EqualsBool { field, syntax, .. } => {
                Self::verify_field_type::<bool>(reflected, field, syntax)
            }
            HtnCondition::GreaterThanInt { field, syntax, .. } => {
                Self::verify_field_type::<i32>(reflected, field, syntax)
            }
            HtnCondition::LessThanInt { field, syntax, .. } => {
                Self::verify_field_type::<i32>(reflected, field, syntax)
            }
            HtnCondition::EqualsInt { field, syntax, .. } => {
                Self::verify_field_type::<i32>(reflected, field, syntax)
            }
            HtnCondition::EqualsFloat { field, syntax, .. } => {
                Self::verify_field_type::<f32>(reflected, field, syntax)
            }
            HtnCondition::GreaterThanFloat { field, syntax, .. } => {
                Self::verify_field_type::<f32>(reflected, field, syntax)
            }
            HtnCondition::LessThanFloat { field, syntax, .. } => {
                Self::verify_field_type::<f32>(reflected, field, syntax)
            }
            HtnCondition::IsNone { field, syntax, .. }
            | HtnCondition::IsSome { field, syntax, .. } => {
                if let Some(val) = reflected.field(field) {
                    let dyn_enum = val.reflect_ref().as_enum().map_err(|_| {
                        format!(
                            "Field `{field}` is expected to be an Enum, in condition: `{syntax}`"
                        )
                    })?;
                    let enum_info = dyn_enum.get_represented_enum_info().ok_or_else(|| {
                        format!(
                            "Field `{field}` is expected to be an Option Enum, in condition: `{syntax}`"
                        )
                    })?;
                    let is_state_field_an_option = enum_info.variant_names().len() == 2
                        && enum_info.variant_names()[0] == "None"
                        && enum_info.variant_names()[1] == "Some";
                    if !is_state_field_an_option {
                        return Err(format!(
                            "Field `{field}` is expected to be an Option, in condition: `{syntax}`"
                        ));
                    }
                    Ok(())
                } else {
                    Err(format!(
                        "Unknown state field `{field}` for condition `{syntax}`"
                    ))
                }
            }

            HtnCondition::EqualsEnum {
                field,
                enum_type,
                enum_variant,
                syntax,
                ..
            } => {
                if let Some(state_val) = reflected.field(field) {
                    let dyn_enum = state_val.reflect_ref().as_enum().map_err(|_| {
                        format!(
                            "Field `{field}` is expected to be an Enum, in condition: `{syntax}`"
                        )
                    })?;
                    let enum_info = dyn_enum
                        .get_represented_enum_info()
                        .expect("Field is not an enum");
                    let Some(variant) = enum_info.variant(enum_variant) else {
                        return Err(format!(
                            "Variant '{enum_type}::{enum_variant}' not found in enum for condition: '{syntax}'"
                        ));
                    };
                    match variant {
                        VariantInfo::Struct(..) | VariantInfo::Tuple(..) => {
                            return Err(format!(
                            "Struct enums and Tuple enums are not supported. condition: `{syntax}`"
                        ))
                        }
                        VariantInfo::Unit(_) => (),
                    }
                    if enum_info.type_path_table().ident() != Some(enum_type) {
                        return Err(format!("Enum type mismatch for condition: `{syntax}`"));
                    }
                    Ok(())
                } else {
                    Err(format!(
                        "Unknown state field `{field}` for condition `{syntax}`"
                    ))
                }
            }
            HtnCondition::EqualsIdentifier {
                field: field1,
                other_field: field2,
                syntax,
                ..
            }
            | HtnCondition::GreaterThanIdentifier {
                field: field1,
                other_field: field2,
                syntax,
                ..
            }
            | HtnCondition::LessThanIdentifier {
                field: field1,
                other_field: field2,
                syntax,
                ..
            } => {
                let Some(val1) = reflected.field(field1) else {
                    return Err(format!(
                        "Unknown state field `{field1}` for condition `{syntax}`"
                    ));
                };
                let Some(val2) = reflected.field(field2) else {
                    return Err(format!(
                        "Unknown state field `{field2}` for condition `{syntax}`"
                    ));
                };

                // reflected fields known to exist due to above code, so unwrap:
                let val1_type = val1.get_represented_type_info().unwrap().type_id();
                let val2_type = val2.get_represented_type_info().unwrap().type_id();

                if val1_type != val2_type {
                    return Err(format!(
                        "Fields `{field1}` and `{field2}` are not of the same type for condition `{syntax}`"
                    ));
                }
                Ok(())
            }
        }
    }
    pub fn evaluate<T: HtnStateTrait>(&self, state: &T, atr: &AppTypeRegistry) -> bool {
        let reflected = state
            .reflect_ref()
            .as_struct()
            .expect("State is not a struct");
        match self {
            HtnCondition::EqualsBool {
                field,
                value,
                notted,
                ..
            } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(b) = val.try_downcast_ref::<bool>() {
                        if *notted {
                            *b != *value
                        } else {
                            *b == *value
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HtnCondition::EqualsFloat {
                field,
                value,
                notted,
                ..
            } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(f) = val.try_downcast_ref::<f32>() {
                        if *notted {
                            *f != *value
                        } else {
                            *f == *value
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HtnCondition::GreaterThanFloat {
                field,
                threshold,
                orequals,
                ..
            } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(f) = val.try_downcast_ref::<f32>() {
                        if *orequals {
                            *f >= *threshold
                        } else {
                            *f > *threshold
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HtnCondition::LessThanFloat {
                field,
                threshold,
                orequals,
                ..
            } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(i) = val.try_downcast_ref::<f32>() {
                        if *orequals {
                            *i >= *threshold
                        } else {
                            *i > *threshold
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HtnCondition::LessThanInt {
                field,
                threshold,
                orequals,
                ..
            } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(i) = val.try_downcast_ref::<i32>() {
                        if *orequals {
                            *i <= *threshold
                        } else {
                            *i < *threshold
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HtnCondition::GreaterThanInt {
                field,
                threshold,
                orequals,
                ..
            } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(f) = val.try_downcast_ref::<i32>() {
                        if *orequals {
                            *f >= *threshold
                        } else {
                            *f > *threshold
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HtnCondition::EqualsInt {
                field,
                value,
                notted,
                ..
            } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(i) = val.try_downcast_ref::<i32>() {
                        if *notted {
                            *i != *value
                        } else {
                            *i == *value
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HtnCondition::EqualsEnum {
                field,
                enum_type,
                enum_variant,
                notted,
                ..
            } => {
                // https://github.com/makspll/bevy_mod_scripting/blob/a4d1ffbcae98f42393ab447d73efe9b0b543426f/crates/bevy_mod_scripting_core/src/bindings/world.rs#L642
                if let Some(val) = reflected.field(field) {
                    let dyn_enum = val.reflect_ref().as_enum().expect("Field is not an enum");
                    let enum_info = dyn_enum
                        .get_represented_enum_info()
                        .expect("Field is not an enum");
                    let variant = enum_info.variant(enum_variant).expect("Variant not found");
                    let variant = match variant {
                        VariantInfo::Struct(..) => unimplemented!("Enum structs not supported"),
                        VariantInfo::Tuple(..) => unimplemented!("Enum tuples not supported"),
                        VariantInfo::Unit(_) => DynamicVariant::Unit,
                    };
                    let mut dynamic = DynamicEnum::new(enum_variant.clone(), variant);
                    let type_reg = atr
                        .get_type_by_name(enum_type)
                        .expect("Enum type not found");
                    dynamic.set_represented_type(Some(type_reg.type_info()));
                    if *notted {
                        !dynamic.reflect_partial_eq(val).unwrap()
                    } else {
                        dynamic.reflect_partial_eq(val).unwrap()
                    }
                } else {
                    false
                }
            }
            HtnCondition::EqualsIdentifier {
                field: field1,
                other_field: field2,
                ..
            } => {
                if let (Some(val1), Some(val2)) = (reflected.field(field1), reflected.field(field2))
                {
                    val1.reflect_partial_eq(val2).unwrap_or(false)
                } else {
                    false
                }
            }
            HtnCondition::GreaterThanIdentifier {
                field: field1,
                orequals,
                other_field: field2,
                syntax,
                ..
            }
            | HtnCondition::LessThanIdentifier {
                field: field1,
                orequals,
                other_field: field2,
                syntax,
                ..
            } => {
                let Some(val1) = reflected.field(field1) else {
                    return false;
                };
                let Some(val2) = reflected.field(field2) else {
                    return false;
                };
                let type_id = val1
                    .get_represented_type_info()
                    .map(|inf| inf.type_id())
                    .unwrap();
                if val1
                    .get_represented_type_info()
                    .map(|inf| inf.type_id())
                    .unwrap()
                    != type_id
                {
                    warn!("Type mismatch for condition: `{syntax}`");
                    return false;
                }
                // don't know how to dynamically do this, there isn't a ReflectPartialOrd.
                // so for now i'll just support numbers.
                let type_path = val1
                    .get_represented_type_info()
                    .unwrap()
                    .type_path_table()
                    .short_path();
                let ordering = match type_path {
                    "i32" => val1.try_downcast_ref::<i32>().unwrap().partial_cmp(val2.try_downcast_ref::<i32>().unwrap()).unwrap(),
                    // not supported by effects (yet?) but could still be used in conditions:
                    "i8" => val1.try_downcast_ref::<i8>().unwrap().partial_cmp(val2.try_downcast_ref::<i8>().unwrap()).unwrap(),
                    "i16" => val1.try_downcast_ref::<i16>().unwrap().partial_cmp(val2.try_downcast_ref::<i16>().unwrap()).unwrap(),
                    "i64" => val1.try_downcast_ref::<i64>().unwrap().partial_cmp(val2.try_downcast_ref::<i64>().unwrap()).unwrap(),
                    "i128" => val1.try_downcast_ref::<i128>().unwrap().partial_cmp(val2.try_downcast_ref::<i128>().unwrap()).unwrap(),
                    // f
                    "f32" => val1.try_downcast_ref::<f32>().unwrap().partial_cmp(val2.try_downcast_ref::<f32>().unwrap()).unwrap(),
                    "f64" => val1.try_downcast_ref::<f64>().unwrap().partial_cmp(val2.try_downcast_ref::<f64>().unwrap()).unwrap(),
                    // u
                    "u8" => val1.try_downcast_ref::<u8>().unwrap().partial_cmp(val2.try_downcast_ref::<u8>().unwrap()).unwrap(),
                    "u16" => val1.try_downcast_ref::<u16>().unwrap().partial_cmp(val2.try_downcast_ref::<u16>().unwrap()).unwrap(),
                    "u32" => val1.try_downcast_ref::<u32>().unwrap().partial_cmp(val2.try_downcast_ref::<u32>().unwrap()).unwrap(),
                    "u64" => val1.try_downcast_ref::<u64>().unwrap().partial_cmp(val2.try_downcast_ref::<u64>().unwrap()).unwrap(),
                    "u128" => val1.try_downcast_ref::<u128>().unwrap().partial_cmp(val2.try_downcast_ref::<u128>().unwrap()).unwrap(),
                    _ => unimplemented!("GreaterThanIdentifier | LessThanIdentifier not implemented for type: {type_path} for condition: `{syntax}`"),
                };

                match self {
                    HtnCondition::GreaterThanIdentifier { .. } => {
                        if *orequals {
                            ordering == std::cmp::Ordering::Greater
                                || ordering == std::cmp::Ordering::Equal
                        } else {
                            ordering == std::cmp::Ordering::Greater
                        }
                    }
                    HtnCondition::LessThanIdentifier { .. } => {
                        if *orequals {
                            ordering == std::cmp::Ordering::Less
                                || ordering == std::cmp::Ordering::Equal
                        } else {
                            ordering == std::cmp::Ordering::Less
                        }
                    }
                    _ => unreachable!(),
                }
            }
            HtnCondition::IsNone { field, .. } | HtnCondition::IsSome { field, .. } => {
                if let Some(val) = reflected.field(field) {
                    let dyn_enum = val
                        .reflect_ref()
                        .as_enum()
                        .expect("Field is not an enum (option)");
                    let enum_info = dyn_enum
                        .get_represented_enum_info()
                        .expect("Field is not an enum");
                    let is_state_field_an_option = enum_info.variant_names().len() == 2
                        && enum_info.variant_names()[0] == "None"
                        && enum_info.variant_names()[1] == "Some";

                    if !is_state_field_an_option {
                        return false;
                    }
                    let var_name = dyn_enum.variant_name();
                    match self {
                        HtnCondition::IsNone { .. } if var_name == "None" => true,
                        HtnCondition::IsSome { .. } if var_name == "Some" => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dsl::parse_htn;

    use super::*;

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
            floatyness: f32,
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
                    e1 > e2,
                    floatyness > 2.0,
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
                HtnCondition::GreaterThanIdentifier {
                    field: "e1".to_string(),
                    other_field: "e2".to_string(),
                    orequals: false,
                    syntax: "e1 > e2".to_string(),
                },
                HtnCondition::GreaterThanFloat {
                    field: "floatyness".to_string(),
                    threshold: 2.0,
                    orequals: false,
                    syntax: "floatyness > 2.0".to_string(),
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
            floatyness: 2.0,
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
        let state2 = State {
            e1: 2,
            ..state.clone()
        };
        assert!(condition.evaluate(&state2, &atr));

        let condition = HtnCondition::GreaterThanIdentifier {
            field: "e1".to_string(),
            other_field: "e2".to_string(),
            orequals: false,
            syntax: "e1 > e2".to_string(),
        };
        assert!(!condition.evaluate(&state, &atr));
        let state2 = State {
            e1: 3,
            ..state.clone()
        };
        assert!(condition.evaluate(&state2, &atr));

        // deliberately using powers of two here to avoid floating point shennanigans
        let condition = HtnCondition::EqualsFloat {
            field: "floatyness".to_string(),
            value: 2.0,
            notted: false,
            syntax: "floatyness == 2.0".to_string(),
        };
        assert!(condition.evaluate(&state, &atr));
        let state2 = State {
            floatyness: 2.0,
            ..state.clone()
        };
        assert!(condition.evaluate(&state2, &atr));
    }
}
