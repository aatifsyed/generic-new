use field::{make_field_infos, FieldConfig};
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
            let field_infos = make_field_infos(user_struct);
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
