mod conditions;
mod effects;
mod htn_builder;
mod task_compound;
mod task_primitive;
#[cfg(test)]
mod tests;

pub use conditions::*;
pub use effects::*;
pub use htn_builder::*;
pub use task_compound::*;
pub use task_primitive::*;

use bevy::{prelude::*, reflect::TypeRegistration};

/// A wrapper around the TypeRegistry with some convenience methods.
pub trait AppTypeRegistryExt {
    fn get_type_by_name(&self, type_name: impl AsRef<str>) -> Option<TypeRegistration>;
}

impl AppTypeRegistryExt for AppTypeRegistry {
    fn get_type_by_name(&self, type_name: impl AsRef<str>) -> Option<TypeRegistration> {
        let tr = self.read();
        let type_name = type_name.as_ref();
        tr.get_with_short_type_path(type_name)
            .or_else(|| tr.get_with_type_path(type_name))
            .cloned()
    }
}
