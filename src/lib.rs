use log::debug;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    DataStruct, DeriveInput, Expr, Ident, Token, Type,
};

#[derive(Debug, Clone)]
struct FieldInfo {
    input_type: Type,
    input_name: Ident,
    struct_name: Option<Ident>,
    transform: TokenStream2,
}

#[derive(Debug, Clone)]
enum GenericNewAttr {
    InputType(Type),
    Transform(Expr),
}

impl Parse for GenericNewAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        match name.to_string().as_str() {
            "input_type" => Ok(GenericNewAttr::InputType(input.parse()?)),
            "transform" => Ok(GenericNewAttr::Transform(input.parse()?)),
            _ => abort!(
                input.span(),
                "Unsupported attribute. Did you mean 'input_type' or 'transform'?"
            ),
        }
    }
}

#[derive(Debug, Clone)]
struct GenericNewAttrs(Punctuated<GenericNewAttr, Token![,]>);

impl Parse for GenericNewAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse_terminated(GenericNewAttr::parse)?))
    }
}

impl FieldInfo {
    fn input(&self) -> TokenStream2 {
        let input_name = self.input_name.clone();
        let input_type = self.input_type.clone();
        quote!(#input_name: #input_type)
    }
    fn transform(&self) -> TokenStream2 {
        let input_name = self.input_name.clone();
        let transform = self.transform.clone();
        quote!(let #input_name = (#transform)(#input_name))
    }
    fn output(&self) -> TokenStream2 {
        let input_name = self.input_name.clone();
        match self.struct_name.clone() {
            Some(struct_name) => quote!( #struct_name: #input_name ),
            None => quote!(#input_name),
        }
    }
}

fn make_field_infos(data_struct: &DataStruct) -> Vec<FieldInfo> {
    data_struct
        .fields
        .clone()
        .into_iter()
        .enumerate()
        .map(|(n, field)| {
            let span = field.span();
            let attrs: Vec<_> = field
                .attrs
                .iter()
                .filter(|attr| {
                    attr.path
                        .segments
                        .first()
                        .map(|segment| segment.ident.to_string().as_str() == "generic_new")
                        .unwrap_or(false)
                })
                .map(|attr| match attr.parse_args::<GenericNewAttrs>() {
                    Ok(parsed) => parsed.0,
                    Err(e) => abort!(attr, "Couldn't parse attributes: {}", e),
                })
                .flatten()
                .collect();

            debug!("{attrs:?}");

            let struct_name = field.clone().ident;
            FieldInfo {
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

#[proc_macro_error]
#[proc_macro_derive(GenericNew, attributes(generic_new))]
pub fn derive_generic_new(input: TokenStream) -> TokenStream {
    pretty_env_logger::try_init().ok();
    let derive_input = parse_macro_input!(input as DeriveInput);
    let user_ident = derive_input.ident.clone();

    match derive_input.data {
        syn::Data::Struct(ref user_struct) => {
            let field_infos = make_field_infos(user_struct);
            let inputs = field_infos.iter().map(FieldInfo::input);
            let transforms = field_infos.iter().map(FieldInfo::transform);
            let outputs = field_infos.iter().map(FieldInfo::output);

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
