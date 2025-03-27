use crate::HtnStateTrait;

use super::*;
use crate::error::HtnErr;
use bevy::{
    prelude::*,
    reflect::{DynamicEnum, DynamicVariant, VariantInfo},
};
// use float_eq::*;

// #[derive_float_eq(
//     ulps_tol = "PointUlps",
//     ulps_tol_derive = "Clone, Copy, Debug, PartialEq, Eq",
//     debug_ulps_diff = "PointDebugUlpsDiff",
//     debug_ulps_diff_derive = "Clone, Copy, Reflect, Debug, PartialEq, Eq"
// )]
// #[derive(Debug, PartialEq, Clone, Copy, Reflect)]
// pub struct Float(pub f32);

#[derive(Clone, Debug, Reflect, PartialEq)]
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
    SetFloat {
        field: String,
        value: f32,
        syntax: String,
    },
    // sets state.field to the value of state.field_source, so long as they are equal types.
    SetIdentifier {
        field: String,
        field_source: String,
        syntax: String,
    },
    IncrementInt {
        field: String,
        by: i32,
        syntax: String,
    },
    IncrementFloat {
        field: String,
        by: f32,
        syntax: String,
    },
    IncrementIdentifier {
        field: String,
        field_source: String,
        decrement: bool,
        syntax: String,
    },
    SetEnum {
        field: String,
        enum_type: String,
        enum_variant: String,
        syntax: String,
    },
    SetNone {
        field: String,
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
            Effect::IncrementIdentifier { syntax, .. } => syntax,
            Effect::SetEnum { syntax, .. } => syntax,
            Effect::SetNone { syntax, .. } => syntax,
            Effect::SetFloat { syntax, .. } => syntax,
            Effect::IncrementFloat { syntax, .. } => syntax,
        }
    }
    pub fn verify_types<T: HtnStateTrait>(
        &self,
        state: &T,
        atr: &AppTypeRegistry,
        is_expected_effect: bool,
    ) -> Result<(), HtnErr> {
        let effect_noun = if is_expected_effect {
            "expected_effect"
        } else {
            "effect"
        };
        let reflected = state
            .reflect_ref()
            .as_struct()
            .map_err(|e| HtnErr::Condition {
                syntax: format!("{:?}", self),
                details: format!("State is not a struct: {e}"),
            })?;
        match self {
            Effect::SetBool { field, syntax, .. } => {
                if reflected.field(field).is_none() {
                    return Err(HtnErr::Bool {
                        syntax: syntax.clone(),
                        details: format!("Unknown state field `{field}` for {effect_noun}"),
                    });
                };
            }
            Effect::SetInt { field, syntax, .. } | Effect::IncrementInt { field, syntax, .. } => {
                if reflected.field(field).is_none() {
                    return Err(HtnErr::Int {
                        syntax: syntax.clone(),
                        details: format!("Unknown state field `{field}` for {effect_noun}"),
                    });
                };
            }
            Effect::SetFloat { field, syntax, .. }
            | Effect::IncrementFloat { field, syntax, .. } => {
                if reflected.field(field).is_none() {
                    return Err(HtnErr::Float {
                        syntax: syntax.clone(),
                        details: format!("Unknown state field `{field}` for {effect_noun}"),
                    });
                };
            }
            // increment field by another field
            Effect::IncrementIdentifier {
                field,
                field_source,
                syntax,
                ..
            } => {
                let Some(field_val) = reflected.field(field) else {
                    return Err(HtnErr::Condition {
                        syntax: syntax.clone(),
                        details: format!("Unknown state field `{field}` for {effect_noun}"),
                    });
                };
                let Some(field_src_val) = reflected.field(field_source) else {
                    return Err(HtnErr::Condition {
                        syntax: syntax.clone(),
                        details: format!("Unknown state field `{field_source}` for {effect_noun}"),
                    });
                };
                // reflected fields known to exist due to above code, so unwrap:
                let field_type = field_val.get_represented_type_info().unwrap().type_id();
                let field_src_type = field_src_val.get_represented_type_info().unwrap().type_id();

                if field_type != field_src_type {
                    return Err(HtnErr::Condition {
                         syntax: syntax.clone(),
                         details: format!(
                             "An {effect_noun} is trying to +/- '{field}' to '{field_source}' but they are different types"
                         ),
                     });
                }
            }
            // set a field to the value of another field
            Effect::SetIdentifier {
                field,
                field_source,
                syntax,
                ..
            } => {
                let Some(field_val) = reflected.field(field) else {
                    return Err(HtnErr::Condition {
                        syntax: syntax.clone(),
                        details: format!("Unknown state field `{field}` for {effect_noun}"),
                    });
                };
                let Some(field_src_val) = reflected.field(field_source) else {
                    return Err(HtnErr::Condition {
                        syntax: syntax.clone(),
                        details: format!("Unknown state field `{field_source}` for {effect_noun}"),
                    });
                };
                // reflected fields known to exist due to above code, so unwrap:
                let field_type = field_val.get_represented_type_info().unwrap().type_id();
                let field_src_type = field_src_val.get_represented_type_info().unwrap().type_id();

                if field_type != field_src_type {
                    return Err(HtnErr::Condition {
                        syntax: syntax.clone(),
                        details: format!(
                            "An {effect_noun} is trying to set '{field}' to '{field_source}' but they are different types"
                        ),
                    });
                }
            }
            Effect::SetNone { field, syntax, .. } => {
                let Some(val) = reflected.field(field) else {
                    return Err(HtnErr::Enum {
                        syntax: syntax.clone(),
                        details: format!("Unknown state field `{field}` for {effect_noun}"),
                    });
                };
                let state_dyn_enum = val.reflect_ref().as_enum().map_err(|_e| HtnErr::Enum {
                    syntax: syntax.clone(),
                    details: format!("{effect_noun} field '{field}' should be an enum!"),
                })?;
                let Some(enum_info) = state_dyn_enum.get_represented_enum_info() else {
                    return Err(HtnErr::Enum {
                        syntax: syntax.clone(),
                        details: format!(
                            "{effect_noun} field '{field}' is not a type registered enum"
                        ),
                    });
                };
                if !enum_info.contains_variant("None") || !enum_info.contains_variant("Some") {
                    return Err(HtnErr::Enum {
                        syntax: syntax.clone(),
                        details: format!("{effect_noun} field '{field}' is not an Option enum"),
                    });
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
                    return Err(HtnErr::Enum {
                        syntax: syntax.clone(),
                        details: format!("Unknown state field `{field}` for {effect_noun}"),
                    });
                };
                let state_dyn_enum = val.reflect_ref().as_enum().map_err(|_e| HtnErr::Enum {
                    syntax: syntax.clone(),
                    details: format!("{effect_noun} field '{field}' should be an enum!"),
                })?;
                let Some(enum_info) = state_dyn_enum.get_represented_enum_info() else {
                    return Err(HtnErr::Enum {
                        syntax: syntax.clone(),
                        details: format!(
                            "{effect_noun} field '{field}' is not a type registered enum"
                        ),
                    });
                };
                if !enum_info.contains_variant(enum_variant) {
                    return Err(HtnErr::Enum {
                        syntax: syntax.clone(),
                        details: format!("{effect_noun} enum variant '{enum_variant}' not found, field name: '{field}'"),
                    });
                };
                // let b = val.represents::<Enum>();
                let Some(type_info) = atr.read().get_type_info(enum_info.type_id()) else {
                    return Err(HtnErr::Enum {
                        syntax: syntax.clone(),
                        details: format!(
                            "{effect_noun} enum type '{enum_type}' not found in type registry"
                        ),
                    });
                };
                if type_info.type_path_table().ident() != Some(enum_type.as_str()) {
                    return Err(HtnErr::Enum {
                        syntax: syntax.clone(),
                        details: format!("{effect_noun} enum type mismatch when setting field '{field}' to {enum_type}::{enum_variant}"),
                    });
                }
            }
        }
        Ok(())
    }

    /// We should have verified the effect will work, so it's ok to panic in here?
    pub fn apply<T: HtnStateTrait>(&self, state: &mut T, atr: &AppTypeRegistry) {
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
            Effect::SetFloat { field, value, .. } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(f) = val.try_downcast_mut::<f32>() {
                        *f = *value;
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
            Effect::IncrementFloat { field, by, .. } => {
                if let Some(val) = reflected.field_mut(field) {
                    if let Some(f) = val.try_downcast_mut::<f32>() {
                        *f += *by;
                    }
                } else {
                    panic!("Field {field} does not exist in the state");
                }
            }
            Effect::IncrementIdentifier {
                field,
                field_source,
                decrement,
                ..
            } => {
                let Some(newval) = reflected.field(field_source) else {
                    panic!("Field {field_source} does not exist in the state");
                };
                let newval = newval.clone_value();
                let Some(val) = reflected.field_mut(field) else {
                    panic!("Field {field} does not exist in the state");
                };
                fn try_inc<T: std::ops::AddAssign + std::ops::SubAssign + Copy + 'static>(
                    val: &mut dyn PartialReflect,
                    decrement: bool,
                    newval: &dyn PartialReflect,
                ) -> bool {
                    if let (Some(f), Some(f2)) =
                        (val.try_downcast_mut::<T>(), newval.try_downcast_ref::<T>())
                    {
                        if decrement {
                            *f -= *f2;
                        } else {
                            *f += *f2;
                        }
                        true
                    } else {
                        false
                    }
                }
                let ok = try_inc::<i32>(val, *decrement, newval.as_ref())
                    || try_inc::<u32>(val, *decrement, newval.as_ref())
                    || try_inc::<usize>(val, *decrement, newval.as_ref())
                    || try_inc::<i8>(val, *decrement, newval.as_ref())
                    || try_inc::<i16>(val, *decrement, newval.as_ref())
                    || try_inc::<i64>(val, *decrement, newval.as_ref())
                    || try_inc::<i128>(val, *decrement, newval.as_ref())
                    || try_inc::<f32>(val, *decrement, newval.as_ref())
                    || try_inc::<f64>(val, *decrement, newval.as_ref())
                    || try_inc::<u8>(val, *decrement, newval.as_ref())
                    || try_inc::<u16>(val, *decrement, newval.as_ref())
                    || try_inc::<u64>(val, *decrement, newval.as_ref())
                    || try_inc::<u128>(val, *decrement, newval.as_ref());
                if !ok {
                    panic!("Field {field_source} cannot be applied to {field}");
                }
            }
            Effect::SetIdentifier {
                field,
                field_source,
                ..
            } => {
                let Some(newval) = reflected.field(field_source) else {
                    panic!("Field {field_source} does not exist in the state");
                };
                let newval = newval.clone_value();
                let val = reflected.field_mut(field).unwrap();
                val.apply(newval.as_ref());
            }
            Effect::SetNone { field, .. } => {
                let val = reflected.field_mut(field).unwrap();
                let enum_variant = "None";
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
                let new_dyn_enum = DynamicEnum::new(enum_variant, variant);
                state_dyn_enum.apply(new_dyn_enum.as_partial_reflect());
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

#[cfg(test)]
mod tests {
    use crate::dsl::parse_htn;

    use super::*;

    #[test]
    fn test_effects() {
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
            opt: Option<f32>,
            opt2: Option<f32>,
        }

        let src = r#"
    schema {
        version: 0.1.0
    }
            
    primitive_task "Effects Test" {
        operator: DummyOperator
        preconditions: []
        effects: [
            happy = true,
            energy = 200,
            e1 = e2,
            energy -= 50,
            location = Location::Park,
            floatyness = 2.0,
            opt = None,
            energy += e1,
        ]
    }
    "#;
        let atr = AppTypeRegistry::default();
        {
            let mut atr = atr.write();
            atr.register::<State>();
            atr.register::<Location>();
        }

        let htn = parse_htn::<State>(src).expect("Failed to parse htn");
        let Some(Task::Primitive(pt)) = &htn.tasks.first() else {
            panic!("Task should exist");
        };
        assert_eq!(
            pt.effects,
            vec![
                Effect::SetBool {
                    field: "happy".to_string(),
                    value: true,
                    syntax: "happy = true".to_string(),
                },
                Effect::SetInt {
                    field: "energy".to_string(),
                    value: 200,
                    syntax: "energy = 200".to_string(),
                },
                Effect::SetIdentifier {
                    field: "e1".to_string(),
                    field_source: "e2".to_string(),
                    syntax: "e1 = e2".to_string(),
                },
                Effect::IncrementInt {
                    field: "energy".to_string(),
                    by: -50,
                    syntax: "energy -= 50".to_string(),
                },
                Effect::SetEnum {
                    field: "location".to_string(),
                    enum_type: "Location".to_string(),
                    enum_variant: "Park".to_string(),
                    syntax: "location = Location::Park".to_string(),
                },
                Effect::SetFloat {
                    field: "floatyness".to_string(),
                    value: 2.0,
                    syntax: "floatyness = 2.0".to_string(),
                },
                Effect::SetNone {
                    field: "opt".to_string(),
                    syntax: "opt = None".to_string(),
                },
                Effect::IncrementIdentifier {
                    field: "energy".to_string(),
                    field_source: "e1".to_string(),
                    decrement: false,
                    syntax: "energy += e1".to_string(),
                },
            ]
        );

        let initial_state = State {
            energy: 10,
            floatyness: 1.0,
            happy: false,
            location: Location::Home,
            e1: 1,
            e2: 2,
            opt: Some(1.0),
            opt2: None,
        };

        let mut state = initial_state.clone();
        let effect = Effect::SetBool {
            field: "happy".to_string(),
            value: true,
            syntax: "happy = true".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert!(state.happy);

        let mut state = initial_state.clone();
        let effect = Effect::SetInt {
            field: "energy".to_string(),
            value: 100,
            syntax: "energy = 100".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.energy, 100);

        let mut state = initial_state.clone();
        let effect = Effect::SetIdentifier {
            field: "e1".to_string(),
            field_source: "e2".to_string(),
            syntax: "e1 = e2".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.e1, 2);

        let mut state = initial_state.clone();
        let effect = Effect::SetEnum {
            field: "location".to_string(),
            enum_type: "Location".to_string(),
            enum_variant: "Park".to_string(),
            syntax: "location = Location::Park".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.location, Location::Park);

        let mut state = initial_state.clone();
        let effect = Effect::IncrementInt {
            field: "energy".to_string(),
            by: 10,
            syntax: "energy += 10".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.energy, 20);

        let mut state = initial_state.clone();
        let effect = Effect::IncrementIdentifier {
            field: "energy".to_string(),
            field_source: "e1".to_string(),
            decrement: false,
            syntax: "energy += e1".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.energy, 11);

        let mut state = initial_state.clone();
        let effect = Effect::IncrementIdentifier {
            field: "energy".to_string(),
            field_source: "e1".to_string(),
            decrement: true,
            syntax: "energy -= e1".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.energy, 9);

        let mut state = initial_state.clone();
        let effect = Effect::IncrementInt {
            field: "energy".to_string(),
            by: -10,
            syntax: "energy -= 10".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.energy, 0);

        let mut state = initial_state.clone();
        let effect = Effect::SetNone {
            field: "opt".to_string(),
            syntax: "opt = None".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.opt, None);

        // deliberately using powers of two here to avoid float point shennanigans
        let mut state = initial_state.clone();
        let effect = Effect::SetFloat {
            field: "floatyness".to_string(),
            value: 4.0,
            syntax: "floatyness = 4.0".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.floatyness, 4.0);

        let mut state = initial_state.clone();
        let effect = Effect::SetNone {
            field: "opt".to_string(),
            syntax: "opt = None".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.opt, None);

        // there is no SetSome yet. can maybe do it be constructing the default value of the Option Some,
        // but that should probably be restricted to expected_effects. bit unpleasant.
    }
}
