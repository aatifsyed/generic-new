use proc_macro_error::abort;
use smart_default::SmartDefault;
use syn::{
    parse::{Parse, ParseStream},
    Expr, Token, Type,
};

use crate::attributes::UserAttribute;

/// Config added by the user
#[derive(Debug, SmartDefault)]
pub enum UserConfig {
    #[default]
    None,
    Ignore,
    Custom(Type, Expr),
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
        match (ignore, input_type, converter) {
            (false, None, None) => Ok(UserConfig::None),
            (true, None, None) => Ok(UserConfig::Ignore),
            (true, _, _) => abort!(
                input.span(),
                "`ignore` is mutually exclusive with other options"
            ),
            (false, Some(t), Some(e)) => Ok(UserConfig::Custom(t, e)),
            (false, _, _) => abort!(input.span(), "Must provide both `ty` and `converter`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::UserConfig;
    #[test]
    fn parse_ignore() -> anyhow::Result<()> {
        let config = syn::parse2::<UserConfig>(quote!(ignore))?;
        println!("{config:?}");
        assert!(matches!(config, UserConfig::Ignore));
        Ok(())
    }

    #[test]
    fn parse_nothing() -> anyhow::Result<()> {
        let config = syn::parse2::<UserConfig>(quote!())?;
        println!("{config:?}");
        assert!(matches!(config, UserConfig::None));
        Ok(())
    }

    #[test]
    fn parse_custom() -> anyhow::Result<()> {
        let config = syn::parse2::<UserConfig>(quote!(ty = usize, converter = |u| format!("{u}")))?;
        println!("{config:?}");
        assert!(matches!(config, UserConfig::Custom(_, _)));
        Ok(())
    }

    #[should_panic]
    #[test]
    fn parse_incomplete() {
        let _ = syn::parse2::<UserConfig>(quote!(ty = usize));
    }
}
