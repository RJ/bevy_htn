use crate::HtnStateTrait;

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
    pub fn verify_types<T: HtnStateTrait>(
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
                field_source,
                syntax,
                ..
            } => {
                let Some(field_val) = reflected.field(field) else {
                    return Err(format!(
                        "Unknown state field `{field}` for {effect_noun} `{syntax}`"
                    ));
                };
                let Some(field_src_val) = reflected.field(field_source) else {
                    return Err(format!(
                        "Unknown state field `{field_source}` for {effect_noun} `{syntax}`"
                    ));
                };
                // reflected fields known to exist due to above code, so unwrap:
                let field_type = field_val.get_represented_type_info().unwrap().type_id();
                let field_src_type = field_src_val.get_represented_type_info().unwrap().type_id();

                if field_type != field_src_type {
                    return Err(format!(
                        "An {effect_noun} is trying to set '{field}' to '{field_source}' but they are different types"
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
                    .map_err(|_e| format!("{effect_noun} field '{field}' should be an enum!"))?;
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
            happy: bool,
            location: Location,
            e1: i32,
            e2: i32,
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
            ]
        );

        let initial_state = State {
            energy: 10,
            happy: false,
            location: Location::Home,
            e1: 1,
            e2: 2,
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
        let effect = Effect::IncrementInt {
            field: "energy".to_string(),
            by: -10,
            syntax: "energy -= 10".to_string(),
        };
        effect.apply(&mut state, &atr);
        assert_eq!(state.energy, 0);
    }
}
