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
    },
    GreaterThanInt {
        field: String,
        threshold: i32,
        orequals: bool,
    },
    LessThanInt {
        field: String,
        threshold: i32,
        orequals: bool,
    },
    EqualsEnum {
        field: String,
        enum_type: String,
        enum_variant: String,
        notted: bool,
    },
    EqualsInt {
        field: String,
        value: i32,
        notted: bool,
    },
}

impl HtnCondition {
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
        }
    }
}
