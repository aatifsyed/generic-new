//! <div align="center">
//!
//! [![crates-io](https://img.shields.io/crates/v/generic-new.svg)](https://crates.io/crates/generic-new)
//! [![docs-rs](https://docs.rs/generic-new/badge.svg)](https://docs.rs/generic-new)
//! [![github](https://img.shields.io/static/v1?label=&message=github&color=grey&logo=github)](https://github.com/aatifsyed/generic-new)
//!
//! </div>
//!
//! A derive macro which generates an ergonomic constructor with shortcuts for certain types.
//!
//! ```rust
//! # use std::path::PathBuf;
//! use generic_new::GenericNew;
//!
//! #[derive(GenericNew)]
//! struct Foo {
//!     s: String,     // -> impl AsRef<str>
//!     v: Vec<usize>, // -> impl IntoIterator<Item = usize>
//!     p: PathBuf,    // -> impl AsRef<Path>
//!     #[generic_new(ignore)]
//!     i: String,     // Turn off magic conversion for some fields
//!     #[generic_new(ty = impl Into<usize>, converter = |u|Into::into(u))]
//!     u: usize,      // Custom converters are supported
//! }
//!
//! # fn _make_foo() {
//! Foo::new(
//!     "hello",
//!     [1, 2, 3],
//!     "path/to/foo",
//!     String::from("world"),
//!     1u16,
//! );
//! 
//! # }
//! ```

use field::{make_field_configs, FieldConfig};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
mod attributes;
mod config;
mod field;

#[proc_macro_error]
#[proc_macro_derive(GenericNew, attributes(generic_new))]
pub fn derive_generic_new(input: TokenStream) -> TokenStream {
    pretty_env_logger::try_init().ok();
    let derive_input = parse_macro_input!(input as DeriveInput);
    let user_ident = derive_input.ident.clone();

    match derive_input.data {
        syn::Data::Struct(ref user_struct) => {
            let field_infos = make_field_configs(user_struct);
            let inputs = field_infos.iter().map(FieldConfig::input);
            let transforms = field_infos.iter().map(FieldConfig::transform);
            let outputs = field_infos.iter().map(FieldConfig::output);

            let constructor = match user_struct.fields {
                syn::Fields::Named(_) => quote!(Self {#(#outputs,)*}),
                syn::Fields::Unnamed(_) => quote!(Self(#(#outputs,)*)),
                syn::Fields::Unit => abort!(derive_input, "Unit fields are not supported"),
            };

            let appended = quote! {
                impl #user_ident {
                    pub fn new(
                        #(#inputs,)*
                    ) -> Self {
                        #(#transforms;)*
                        #constructor
                    }
                }
            };
            appended.into()
        }
        syn::Data::Enum(_) => abort!(derive_input, "Enums are not yet supported"),
        syn::Data::Union(_) => abort!(derive_input, "Unions are not supported"),
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.pass("trybuild/*.rs")
    }
}
