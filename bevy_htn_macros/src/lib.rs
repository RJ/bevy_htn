use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Lit, Meta};

/// The default implementation of HtnOperator is to generate a behaviour tree that just
/// executes `Behave::trigger(self.clone())`.
///
/// If the `spawn_named` attribute is present, the behaviour tree will be generated to
/// execute `Behave::spawn_named(name, self.clone())` instead:
///
/// ```rust
/// #[derive(Debug, Reflect, Default, Clone, HtnOperator)]
/// #[reflect(Default, HtnOperator)]
/// #[spawn_named = "Eat a suculent meal"]
/// pub struct EatOperator;
/// ```
#[proc_macro_derive(HtnOperator, attributes(spawn_named))]
pub fn derive_htn_operator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Look for spawn_named attribute
    let spawn_named = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("spawn_named"))
        .and_then(|attr| {
            if let Meta::NameValue(name_value) = &attr.meta {
                if let Expr::Lit(expr_lit) = &name_value.value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        return Some(lit_str.value());
                    }
                }
            }
            None
        });
    let name = &input.ident;
    let tree_impl = if let Some(spawn_name) = spawn_named {
        quote! {
            Behave::spawn_named(#spawn_name, self.clone())
        }
    } else {
        quote! {
            Behave::trigger(self.clone())
        }
    };
    let expanded = quote! {
        impl HtnOperator for #name {
            fn to_tree(&self) -> Option<Tree<Behave>> {
                Some(behave! {
                    #tree_impl
                })
            }
        }
    };
    TokenStream::from(expanded)
}
