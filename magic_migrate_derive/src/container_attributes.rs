use std::str::FromStr;
use strum::IntoEnumIterator;
use syn::{parse::Parse, punctuated::Punctuated, Ident, Token};

const NAMESPACE: &str = "try_migrate";

/// Holds one key/value pair of a parsed container (struct/enum/etc.) attribute
#[derive(Debug, Clone, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::EnumIter, strum::Display, strum::EnumString))]
#[strum_discriminants(name(KnownAttribute))]
pub(crate) enum ParsedAttribute {
    /// #[try_migrate(from = MetadataV1)] or #[try_migrate(from = None)]
    #[allow(non_camel_case_types)]
    from(syn::Path),
    /// #[try_migrate(error = magic_migrate::MigrateError)]
    #[allow(non_camel_case_types)]
    error(syn::Path),
    /// #[try_migrate(deserializer = toml::Deserializer::new)]
    #[allow(non_camel_case_types)]
    deserializer(syn::Path),
}

impl Parse for KnownAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let identity: Ident = input.parse()?;
        KnownAttribute::from_str(&identity.to_string()).map_err(|_| {
            syn::Error::new(
                identity.span(),
                format!(
                    "Unknown {NAMESPACE} attribute: `{identity}`. Must be one of {valid_keys}",
                    valid_keys = KnownAttribute::iter()
                        .map(|key| format!("`{key}`"))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
            )
        })
    }
}

impl Parse for ParsedAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key: KnownAttribute = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        match key {
            KnownAttribute::from => Ok(ParsedAttribute::from(input.parse()?)),
            KnownAttribute::error => Ok(ParsedAttribute::error(input.parse()?)),
            KnownAttribute::deserializer => Ok(ParsedAttribute::deserializer(input.parse()?)),
        }
    }
}

/// Holds a fully parsed container (struct, enum, etc.), including attributes
#[derive(Debug)]
pub(crate) struct Container {
    pub(crate) identity: Ident,
    pub(crate) prior: syn::Path,
    pub(crate) error: Option<syn::Path>,
    pub(crate) deserializer: Option<syn::Path>,
}

impl Container {
    pub(crate) fn from_ast(input: &syn::DeriveInput) -> syn::Result<Self> {
        let identity = input.ident.clone();
        let mut maybe_prior: Option<syn::Path> = None;
        let mut maybe_error: Option<syn::Path> = None;
        let mut maybe_deserializer: Option<syn::Path> = None;

        for attribute_ast in input
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident(NAMESPACE))
        {
            // Multiple attribute declarations may match, treat them as if they were defined all in one row
            for attr in attribute_ast
                .parse_args_with(Punctuated::<ParsedAttribute, Token![,]>::parse_terminated)?
                .into_iter()
            {
                match attr {
                    ParsedAttribute::from(path) => {
                        maybe_prior = Some(path);
                    }
                    ParsedAttribute::error(path) => maybe_error = Some(path),
                    ParsedAttribute::deserializer(path) => maybe_deserializer = Some(path),
                }
            }
        }

        let prior = maybe_prior.ok_or_else(|| {
            syn::Error::new(
                identity.span(),
                format!(
                    "Missing required attribute `{from}`. Use `#[{NAMESPACE}({from} = <struct>)]` to migrate from another struct or `#[{NAMESPACE}({from} = None)]` if this is the first struct in the migration chain",
                    from = KnownAttribute::from
                )
            )
        })
        .map(|prior| if prior.get_ident().is_some_and(|ident| ident == "None") { identity.clone().into() } else { prior })
        ?;

        Ok(Container {
            identity,
            prior,
            error: maybe_error,
            deserializer: maybe_deserializer,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_prior() {
        let input = syn::parse_quote! {
            #[try_migrate(error = CustomError)]
            struct MetadataV2 {
            }
        };
        let result = Container::from_ast(&input);
        assert!(&result.is_err(), "Expected an error, got {:?}", &result);
        assert_eq!(
            format!("{}", result.err().unwrap()),
            r#"Missing required attribute `from`. Use `#[try_migrate(from = <struct>)]` to migrate from another struct or `#[try_migrate(from = None)]` if this is the first struct in the migration chain"#
        );
    }

    #[test]
    fn test_implicit_error() {
        let input = syn::parse_quote! {
            #[try_migrate(from =  MetadataV1)]
            struct MetadataV2 {}
        };
        let container = Container::from_ast(&input).unwrap();
        assert_eq!(container.error, None)
    }

    #[test]
    fn test_prior_and_error() {
        let input = syn::parse_quote! {
            #[try_migrate(from =  MetadataV1, error = CustomError)]
            struct MetadataV2 {}
        };

        let container = Container::from_ast(&input).unwrap();
        assert!(matches!(
            container,
            Container {
                identity: _,
                prior: _,
                error: Some(_),
                deserializer: None
            }
        ))
    }

    #[test]
    fn test_unknown_attribute_errors() {
        let input = syn::parse_quote! {
            #[try_migrate(from =  MetadataV1, error = CustomError, unknown = "LOL")]
            struct MetadataV2 {}
        };

        let result = Container::from_ast(&input);
        assert!(&result.is_err(), "Expected an error, got {:?}", result);
        assert_eq!(
            format!("{}", &result.err().unwrap()),
            r#"Unknown try_migrate attribute: `unknown`. Must be one of `from`, `error`, `deserializer`"#
        );
    }

    #[test]
    fn test_prior_none() {
        let input = syn::parse_quote! {
            #[try_migrate(from =  None)]
            struct MetadataV1 {
            }
        };

        let container = Container::from_ast(&input).unwrap();
        assert!(matches!(
            container,
            Container {
                identity: _,
                prior: _,
                error: None,
                deserializer: None
            }
        ))
    }

    #[test]
    fn test_prior_explicit_self() {
        let input = syn::parse_quote! {
            #[try_migrate(from =  MetadataV1)]
            struct MetadataV1 {
            }
        };

        let container = Container::from_ast(&input).unwrap();
        assert!(matches!(
            container,
            Container {
                identity: _,
                prior: _,
                error: None,
                deserializer: None
            }
        ))
    }

    #[test]
    fn test_explicit_deserializer() {
        let input = syn::parse_quote! {
            #[try_migrate(from =  MetadataV1, deserializer = serde_json::de::Deserializer::from_str)]
            struct MetadataV1 {
            }
        };

        let container = Container::from_ast(&input).unwrap();
        assert!(matches!(
            container,
            Container {
                identity: _,
                prior: _,
                error: None,
                deserializer: Some(_)
            }
        ))
    }
}
