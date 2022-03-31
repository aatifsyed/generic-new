use derive_syn_parse::Parse;
use syn::{parse::ParseStream, Expr, Ident, Token, Type};

fn ident_is(s: &str) -> impl Fn(ParseStream) -> bool + '_ {
    move |parse_stream| match parse_stream.fork().parse::<Ident>() {
        Ok(ident) => ident.to_string().as_str() == s,
        Err(_) => false,
    }
}

/// Type of expression users can add
#[derive(Debug, Parse)]
pub enum UserAttribute {
    #[peek_with(ident_is("ignore"), name = "ignore")]
    Ignore(Ident),
    #[peek_with(ident_is("ty"), name = "ty")]
    InputType(Ident, Token![=], Type),
    #[peek_with(ident_is("converter"), name = "converter")]
    Converter(Ident, Token![=], Expr),
}

#[cfg(test)]
mod tests {
    use super::UserAttribute;
    use quote::quote;

    #[test]
    fn parse_ignore() -> anyhow::Result<()> {
        let parsed = syn::parse2::<UserAttribute>(quote!(ignore))?;
        println!("{parsed:?}");
        assert!(matches!(parsed, UserAttribute::Ignore(_)));
        Ok(())
    }

    #[test]
    fn parse_input_type() -> anyhow::Result<()> {
        let parsed = syn::parse2::<UserAttribute>(quote!(ty = impl IntoIterator<Item = usize>))?;
        println!("{parsed:?}");
        assert!(matches!(parsed, UserAttribute::InputType(_, _, _)));
        Ok(())
    }

    #[test]
    fn parse_converter() -> anyhow::Result<()> {
        let parsed = syn::parse2::<UserAttribute>(quote!(converter = |t| { t.to_vec() }))?;
        println!("{parsed:?}");
        assert!(matches!(parsed, UserAttribute::Converter(_, _, _)));
        Ok(())
    }

    #[test]
    fn dont_parse_unknown() {
        let res = syn::parse2::<UserAttribute>(quote!(foo));
        println!("{res:?}");
        assert!(res.is_err());
    }
}
