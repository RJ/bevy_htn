use std::any::{Any, TypeId};

use super::*;
use bevy::{
    prelude::*,
    reflect::{DynamicEnum, DynamicVariant, VariantInfo},
};

#[derive(Clone, Debug, Reflect, PartialEq, Eq)]
pub enum Effect {
    SetBool {
        field: String,
        value: bool,
        syntax: String,
    },
    SetInt {
        field: String,
        value: i32,
        syntax: String,
    },
    // sets field to the value of field_to_copy_from
    SetIdentifier {
        field: String,
        field_to_copy_from: String,
        syntax: String,
    },
    IncrementInt {
        field: String,
        by: i32,
        syntax: String,
    },
    SetEnum {
        field: String,
        enum_type: String,
        enum_variant: String,
        syntax: String,
    },
}

impl Effect {
    pub fn syntax(&self) -> &str {
        match self {
            Effect::SetBool { syntax, .. } => syntax,
            Effect::SetInt { syntax, .. } => syntax,
            Effect::SetIdentifier { syntax, .. } => syntax,
            Effect::IncrementInt { syntax, .. } => syntax,
            Effect::SetEnum { syntax, .. } => syntax,
        }
    }
    pub fn verify_types<T: Reflect + Default + TypePath + Clone + core::fmt::Debug>(
        &self,
        state: &T,
        atr: &AppTypeRegistry,
        is_expected_effect: bool,
    ) -> Result<(), String> {
        let effect_noun = if is_expected_effect {
            "expected_effect"
        } else {
            "effect"
        };
        let reflected = state
            .reflect_ref()
            .as_struct()
            .map_err(|e| format!("State is not a struct: {e}"))?;
        match self {
            Effect::SetBool { field, syntax, .. } => {
                if reflected.field(field).is_none() {
                    return Err(format!(
                        "Unknown state field `{field}` for {effect_noun} `{syntax}`"
                    ));
                };
            }
            Effect::SetInt { field, syntax, .. } | Effect::IncrementInt { field, syntax, .. } => {
                if reflected.field(field).is_none() {
                    return Err(format!(
                        "Unknown state field `{field}` for {effect_noun} `{syntax}`"
                    ));
                };
            }
            // set a field to the value of another field
            Effect::SetIdentifier {
                field,
                field_to_copy_from,
                syntax,
                ..
            } => {
                let Some(field_val) = reflected.field(field) else {
                    return Err(format!(
                        "Unknown state field `{field}` for {effect_noun} `{syntax}`"
                    ));
                };
                let Some(field_to_copy_from_val) = reflected.field(field_to_copy_from) else {
                    return Err(format!(
                        "Unknown state field `{field_to_copy_from}` for {effect_noun} `{syntax}`"
                    ));
                };
                if field_val.type_id() != field_to_copy_from_val.type_id() {
                    return Err(format!(
                        "An {effect_noun} is trying to set '{field}' to '{field_to_copy_from}' but they are different types"
                    ));
                }
            }
            Effect::SetEnum {
                field,
                enum_type,
                enum_variant,
                syntax,
                ..
            } => {
                let Some(val) = reflected.field(field) else {
                    return Err(format!(
                        "Unknown state field `{field}` for {effect_noun} `{syntax}`"
                    ));
                };
                let state_dyn_enum = val
                    .reflect_ref()
                    .as_enum()
                    .map_err(|e| format!("{effect_noun} field '{field}' should be an enum!"))?;
                let Some(enum_info) = state_dyn_enum.get_represented_enum_info() else {
                    return Err(format!(
                        "{effect_noun} field '{field}' is not a type registered enum"
                    ));
                };
                if !enum_info.contains_variant(enum_variant) {
                    return Err(format!(
                        "{effect_noun} enum variant '{enum_variant}' not found, field name: '{field}'"
                    ));
                };
                // let b = val.represents::<Enum>();
                let Some(type_info) = atr.read().get_type_info(enum_info.type_id()) else {
                    return Err(format!(
                        "{effect_noun} enum type '{enum_type}' not found in type registry"
                    ));
                };
                if type_info.type_path_table().ident() != Some(enum_type.as_str()) {
                    return Err(format!(
                        "{effect_noun} enum type mismatch when setting field '{field}' to {enum_type}::{enum_variant}"
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn apply<T: Reflect + Default + TypePath + Clone + core::fmt::Debug>(
        &self,
        state: &mut T,
        atr: &AppTypeRegistry,
    ) {
        let reflected = state
            .reflect_mut()
            .as_struct()
            .expect("State is not a struct");
        match self {
            Effect::SetBool { field, value, .. } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(b) = val.try_downcast_mut::<bool>() {
                        *b = *value;
                    }
                } else {
                    panic!("Field {field} does not exist in the state");
                }
            }
            Effect::SetInt { field, value, .. } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(i) = val.try_downcast_mut::<i32>() {
                        *i = *value;
                    }
                } else {
                    panic!("Field {field} does not exist in the state");
                }
            }
            Effect::IncrementInt { field, by, .. } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(i) = val.try_downcast_mut::<i32>() {
                        *i += *by;
                    }
                } else {
                    panic!("Field {field} does not exist in the state");
                }
            }
            Effect::SetIdentifier {
                field,
                field_to_copy_from: value,
                ..
            } => {
                let Some(newval) = reflected.field(value) else {
                    panic!("Field {value} does not exist in the state");
                };
                let newval = newval.clone_value();
                let val = reflected.field_mut(field).unwrap();
                val.apply(newval.as_ref());
            }
            Effect::SetEnum {
                field,
                enum_type,
                enum_variant,
                ..
            } => {
                if let Some(val) = reflected.field_mut(field) {
                    let state_dyn_enum = val.reflect_mut().as_enum().expect("Field is not an enum");
                    let enum_info = state_dyn_enum
                        .get_represented_enum_info()
                        .expect("Field is not an enum");
                    let variant = enum_info.variant(enum_variant).expect("Variant not found");
                    let variant = match variant {
                        VariantInfo::Struct(..) => unimplemented!("Enum structs not supported"),
                        VariantInfo::Tuple(..) => unimplemented!("Enum tuples not supported"),
                        VariantInfo::Unit(_) => DynamicVariant::Unit,
                    };
                    // construct the new value:
                    let mut new_dyn_enum = DynamicEnum::new(enum_variant.clone(), variant);
                    let type_reg = atr
                        .get_type_by_name(enum_type)
                        .expect("Enum type not found");
                    new_dyn_enum.set_represented_type(Some(type_reg.type_info()));
                    state_dyn_enum.apply(new_dyn_enum.as_partial_reflect());
                } else {
                    panic!("Field {field} does not exist in the state");
                }
            }
        }
    }
}
