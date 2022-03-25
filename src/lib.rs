use derive_syn_parse::Parse;
use log::debug;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use single::Single;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    DataStruct, DeriveInput, Expr, Ident, Token, Type,
};
use smart_default::SmartDefault;

#[derive(Debug, Clone)]
struct FieldInfo {
    input_type: Type,
    input_name: Ident,
    struct_name: Option<Ident>,
    transform: TokenStream2,
}

#[derive(Debug, SmartDefault)]
enum UserConfig {
    #[default]
    None,
    Ignore,
    Custom(Type, Expr)
}

impl Parse for UserConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let p = input.parse_terminated::<_, Token![,]>(UserAttribute::parse)?;
        let mut ignore = false;
        let mut input_type = None;
        let mut converter = None;
        for user_attribute in p {
            match user_attribute {
                UserAttribute::Ignore(_) => match ignore {
                    true => abort!(input.span(), "Cannot specify `ignore` more than once"),
                    false => ignore = true,
                },
                UserAttribute::InputType(_, _, t) => {
                    if let Some(_) = input_type.replace(t) {
                        abort!(input.span(), "Can't specify `input_type` more than once")
                    }
                }
                UserAttribute::Converter(_, _, e) => {
                    if let Some(_) = converter.replace(e) {
                        abort!(input.span(), "Can't specify `converter` more than once")
                    }
                }
            }
        }
        match (ignore, input_type, converter ) {
            (false, None, None) => Ok(UserConfig::None),
            (true, None, None) => Ok(UserConfig::Ignore),
            (true, _, _) => abort!(input.span(), "`ignore` is mutually exclusive with other options"),
            (false, Some(t), Some(e)) => Ok(UserConfig::Custom(t, e)),
            (false, _, _) => abort!(input.span(), "Must provide both `ty` and `converter`")

        }
    }
}

fn ident_is(s: &str) -> impl Fn(ParseStream) -> bool + '_ {
    move |parse_stream| match parse_stream.parse::<Ident>() {
        Ok(ident) => ident.to_string().as_str() == s,
        Err(_) => false,
    }
}

#[derive(Debug, Parse)]
enum UserAttribute {
    #[peek_with(ident_is("ignore"), name = "ignore")]
    Ignore(Ident),
    #[peek_with(ident_is("ty"), name = "ty")]
    InputType(Ident, Token![=], Type),
    #[peek_with(ident_is("converter"), name = "converter")]
    Converter(Ident, Token![=], Expr),
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

            let config = attr.map_or(UserConfig::default(), |attr|{
                match attr.parse_args() {
                    Ok(o) => o,
                    Err(e)=> abort!(field.span(), "Couldn't parse attributes"; note = e)
                }
            });

            let span = field.span();

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
