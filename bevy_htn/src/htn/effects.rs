use super::*;
use bevy::{
    prelude::*,
    reflect::{DynamicEnum, DynamicVariant, VariantInfo},
};

#[derive(Clone, Debug, Reflect)]
pub enum Effect {
    SetBool {
        field: String,
        value: bool,
    },
    SetInt {
        field: String,
        value: i32,
    },
    SetIdentifier {
        field: String,
        value: String,
    },
    IncrementInt {
        field: String,
        by: i32,
    },
    SetEnum {
        field: String,
        enum_type: String,
        enum_variant: String,
    },
}

impl Effect {
    pub fn apply<T: Reflect + Default + TypePath + Clone + core::fmt::Debug>(
        &self,
        state: &mut T,
        mirror: &Mirror,
    ) {
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
            Effect::SetIdentifier { field, value } => {
                let newval = reflected.field(value).unwrap().clone_value();
                let val = reflected.field_mut(field).unwrap();
                val.apply(newval.as_ref());
            }
            Effect::SetEnum {
                field,
                enum_type,
                enum_variant,
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
                    let type_reg = mirror
                        .get_type_by_name(enum_type.to_string())
                        .expect("Enum type not found");
                    new_dyn_enum.set_represented_type(Some(type_reg.type_info()));
                    state_dyn_enum.apply(new_dyn_enum.as_partial_reflect());
                }
            }
        }
    }
}
