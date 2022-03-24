use std::iter::zip;

use log::debug;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident};

fn field_names(fields: &syn::FieldsNamed) -> Vec<Ident> {
    fields
        .named
        .clone()
        .into_iter()
        .map(|field| field.ident.unwrap())
        .collect()
}

fn argument_names(fields: &syn::Fields) -> Vec<Ident> {
    match fields {
        syn::Fields::Named(fields) => field_names(fields),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .clone()
            .into_iter()
            .enumerate()
            .map(|(n, _)| Ident::new(&format!("arg{n}"), fields.span()))
            .collect(),
        syn::Fields::Unit => abort!(fields, "Unit types are not supported"),
    }
}

fn argument_types(fields: &syn::Fields) -> Vec<syn::Type> {
    field_types(fields)
}

fn make_constructor(
    derive_input: &DeriveInput,
    constructor_args: Vec<TokenStream2>,
) -> TokenStream2 {
    match &derive_input.data {
        syn::Data::Struct(user_struct) => match &user_struct.fields {
            syn::Fields::Named(fields) => {
                let args = zip(field_names(fields), constructor_args)
                    .map(|(name, args)| quote!(#name: #args));
                quote! {
                    Self{
                        #(#args,)*
                    }
                }
            }
            syn::Fields::Unnamed(_) => quote! {
                Self(
                    #(#constructor_args,)*
                )
            },
            syn::Fields::Unit => abort!(derive_input, "Unit types are not supported"),
        },
        syn::Data::Enum(_) => abort!(derive_input, "Enums are not yet supported"),
        syn::Data::Union(_) => abort!(derive_input, "Unions are not supported"),
    }
}

fn field_types(fields: &syn::Fields) -> Vec<syn::Type> {
    fields.iter().map(|field| field.ty.clone()).collect()
}

#[proc_macro_error]
#[proc_macro_derive(GenericNew)]
pub fn derive_generic_new(input: TokenStream) -> TokenStream {
    pretty_env_logger::try_init().ok();
    let derive_input = parse_macro_input!(input as DeriveInput);
    debug!("{derive_input:#?}");
    let user_ident = derive_input.ident.clone();

    match derive_input.data {
        syn::Data::Struct(ref user_struct) => {
            let arguments_and_types = zip(
                argument_names(&user_struct.fields),
                argument_types(&user_struct.fields),
            )
            .map(|(name, ty)| quote!(#name: #ty));
            let constructor = make_constructor(
                &derive_input,
                argument_names(&user_struct.fields)
                    .iter()
                    .map(|id| quote!(#id))
                    .collect(),
            );
            let appended = quote! {
                impl #user_ident {
                    pub fn new(
                        #(#arguments_and_types,)*
                    ) -> Self {
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
