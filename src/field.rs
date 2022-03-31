use log::debug;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use single::Single;
use syn::{Type, Ident, DataStruct, spanned::Spanned};
use quote::quote;

use crate::config::UserConfig;


/// A description of how this field should be handled when generating `new`
#[derive(Debug, Clone)]
pub struct FieldConfig {
    /// Argument type in `new`
    input_type: Type,
    /// Argument name in `new`
    input_name: Ident,
    /// Name of this field in the struct.
    /// None for tuple structs
    struct_name: Option<Ident>,
    /// Transform to apply in body of `new`
    transform: TokenStream2,
}

impl FieldConfig {
    /// Argument to `new`
    pub fn input(&self) -> TokenStream2 {
        let input_name = self.input_name.clone();
        let input_type = self.input_type.clone();
        quote!(#input_name: #input_type)
    }
    /// Body inside `new`
    pub fn transform(&self) -> TokenStream2 {
        let input_name = self.input_name.clone();
        let transform = self.transform.clone();
        quote!(let #input_name = (#transform)(#input_name))
    }
    /// Argument to constructor
    pub fn output(&self) -> TokenStream2 {
        let input_name = self.input_name.clone();
        match self.struct_name.clone() {
            Some(struct_name) => quote!( #struct_name: #input_name ),
            None => quote!(#input_name),
        }
    }
}

pub fn make_field_infos(data_struct: &DataStruct) -> Vec<FieldConfig> {
    data_struct
        .fields
        .clone()
        .into_iter()
        .enumerate()
        .map(|(n, field)| {
            let attr = match field
                .attrs
                .iter()
                .filter(|attr| {
                    attr.path
                        .segments
                        .first()
                        .map(|segment| segment.ident.to_string().as_str() == "generic_new")
                        .unwrap_or(false)
                })
                .single()
            {
                Ok(a) => Some(a),
                Err(e) => match e {
                    single::Error::NoElements => None,
                    single::Error::MultipleElements => {
                        abort!(field.span(), "Can't specify `generic_new` more than once")
                    }
                },
            };

            debug!("{attr:?}");

            let config = attr.map_or(UserConfig::default(), |attr| match attr.parse_args() {
                Ok(o) => o,
                Err(e) => abort!(field.span(), "Couldn't parse attributes"; note = e),
            });

            let span = field.span();

            let struct_name = field.clone().ident;
            FieldConfig {
                input_type: field.ty,
                input_name: field
                    .ident
                    .unwrap_or_else(|| Ident::new(&format!("arg{n}"), span)),
                struct_name,
                transform: quote!(|i| i),
            }
        })
        .collect()
}
