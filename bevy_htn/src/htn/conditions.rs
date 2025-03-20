use std::any::Any;

use super::*;
use bevy::{
    prelude::*,
    reflect::{DynamicEnum, DynamicVariant, VariantInfo},
};

#[derive(Clone, Debug, Reflect, PartialEq, Eq)]
pub enum HtnCondition {
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
    LessThanInt {
        field: String,
        threshold: i32,
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
    EqualsIdentifier {
        field: String,
        other_field: String,
        notted: bool,
        syntax: String,
    },
}

impl HtnCondition {
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
    pub fn verify_types<T: Reflect + Default + TypePath + Clone + core::fmt::Debug>(
        &self,
        state: &T,
        atr: &AppTypeRegistry,
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
    pub fn evaluate<T: Reflect + Default + TypePath + Clone + core::fmt::Debug>(
        &self,
        state: &T,
        atr: &AppTypeRegistry,
    ) -> bool {
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
            HtnCondition::GreaterThanInt {
                field,
                threshold,
                orequals,
                ..
            } => {
                if let Some(val) = reflected.field(field) {
                    if let Some(i) = val.try_downcast_ref::<i32>() {
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
        }
    }
}
