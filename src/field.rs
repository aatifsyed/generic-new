use crate::config::UserConfig;
use log::debug;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort, ResultExt};
use quote::quote;
use single::Single;
use syn::{
    spanned::Spanned, AngleBracketedGenericArguments, DataStruct, Field, GenericArgument, Ident,
    PathArguments, PathSegment, Type, TypePath,
};

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

fn magic_field_config(field: Field, input_name: Ident) -> Option<FieldConfig> {
    match field.ty {
        Type::Path(TypePath {
            qself: None,
            path:
                syn::Path {
                    leading_colon: None,
                    segments,
                },
        }) => match segments.into_iter().collect::<Vec<_>>().as_slice() {
            // String -> impl AsRef<str>
            [PathSegment {
                ident,
                arguments: PathArguments::None,
            }] if ident.to_string() == "String" => Some(FieldConfig {
                input_type: syn::parse2(quote!(impl ::std::convert::AsRef<::std::primitive::str>))
                    .unwrap(),
                input_name,
                struct_name: field.ident,
                transform: quote!(|s| ::std::string::String::from(::std::convert::AsRef::<
                    ::std::primitive::str,
                >::as_ref(&s))),
            }),
            // Vec<T> -> impl IntoIterator<Item = T>
            [PathSegment {
                ident,
                arguments:
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
            }] if ident.to_string() == "Vec" => {
                match args.into_iter().collect::<Vec<_>>().as_slice() {
                    // Single concrete type argument
                    [GenericArgument::Type(ty)] => Some(FieldConfig {
                        input_type: syn::parse2(quote!(impl ::std::iter::IntoIterator<Item = #ty>))
                            .unwrap(),
                        input_name,
                        struct_name: field.ident,
                        transform: quote!(|i| {
                            let mut v = std::vec::Vec::new();
                            for item in i {
                                v.push(item)
                            }
                            v
                        }),
                    }),
                    _ => None,
                }
            }
            // PathBuf -> impl AsRef<Path>
            [PathSegment { ident, .. }] if ident.to_string() == "PathBuf" => Some(FieldConfig {
                input_type: syn::parse2(quote!(impl ::std::convert::AsRef<::std::path::Path>))
                    .unwrap(),
                input_name,
                struct_name: field.ident,
                transform: quote!(|s| ::std::path::PathBuf::from(::std::convert::AsRef::<
                    ::std::path::Path,
                >::as_ref(&s))),
            }),
            _ => None,
        },
        _ => None,
    }
}

pub fn make_field_configs(data_struct: &DataStruct) -> Vec<FieldConfig> {
    data_struct
        .fields
        .clone()
        .into_iter()
        .enumerate()
        .map(|(n, field)| {
            // Get the #[generic_new(...)], if there is one
            let generic_new_attribute = match field
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

            debug!("{generic_new_attribute:?}");

            // Turn it into UserConfig
            let user_config = generic_new_attribute.map(|attribute| {
                attribute
                    .parse_args::<UserConfig>()
                    .expect_or_abort("Couldn't parse attributes")
            });

            let span = field.span();

            let struct_name = field.clone().ident;
            let input_name = field
                .clone()
                .ident
                .unwrap_or_else(|| Ident::new(&format!("arg{n}"), span));

            let noop_config = FieldConfig {
                input_type: field.clone().ty,
                input_name: input_name.clone(),
                struct_name: struct_name.clone(),
                transform: quote!(|i| i),
            };

            match user_config {
                // User has explicitly asked us to ignore this type, so leave as-is
                Some(UserConfig::Ignore) => noop_config,
                // User has provided their own conversion
                Some(UserConfig::Custom(ty, conv)) => FieldConfig {
                    input_type: ty,
                    input_name,
                    struct_name,
                    transform: quote!(#conv),
                },
                None => magic_field_config(field, input_name).unwrap_or(noop_config),
            }
        })
        .collect()
}
