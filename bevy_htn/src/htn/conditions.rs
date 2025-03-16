use super::*;
use bevy::{
    prelude::*,
    reflect::{DynamicEnum, DynamicVariant, VariantInfo},
};

#[derive(Clone, Debug, Reflect)]
pub enum HtnCondition {
    EqualsBool {
        field: String,
        value: bool,
    },
    GreaterThanInt {
        field: String,
        threshold: i32,
    },
    EqualsEnum {
        field: String,
        enum_type: String,
        enum_variant: String,
    },
}

impl HtnCondition {
    pub fn evaluate<T: Reflect + Default + TypePath + Clone + core::fmt::Debug>(
        &self,
        state: &T,
        mirror: &Mirror,
    ) -> bool {
        let reflected = state
            .reflect_ref()
            .as_struct()
            .expect("State is not a struct");
        match self {
            HtnCondition::EqualsBool { field, value } => {
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
            HtnCondition::GreaterThanInt { field, threshold } => {
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
            HtnCondition::EqualsEnum {
                field,
                enum_type,
                enum_variant,
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
                    let type_reg = mirror
                        .get_type_by_name(enum_type.to_string())
                        .expect("Enum type not found");
                    dynamic.set_represented_type(Some(type_reg.type_info()));
                    dynamic.reflect_partial_eq(val).unwrap()
                } else {
                    false
                }
            }
        }
    }
}
