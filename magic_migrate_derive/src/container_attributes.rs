use std::str::FromStr;
use strum::IntoEnumIterator;
use syn::spanned::Spanned;
use syn::{parse::Parse, punctuated::Punctuated, Ident, Path, Token};

const NAMESPACE: &str = "try_migrate";

/// Holds one key/value pair of a parsed container (struct/enum/etc.) attribute
#[derive(Debug, Clone, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::EnumIter, strum::Display, strum::EnumString))]
#[strum_discriminants(name(KnownAttribute))]
pub(crate) enum ParsedAttribute {
    /// #[try_migrate(prior = <struct>)]
    #[allow(non_camel_case_types)]
    prior(Path),
    /// #[try_migrate(error = <Container>)]
    #[allow(non_camel_case_types)]
    error(Path),
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
            KnownAttribute::prior => Ok(ParsedAttribute::prior(input.parse()?)),
            KnownAttribute::error => Ok(ParsedAttribute::error(input.parse()?)),
        }
    }
}

/// Holds a fully parsed container (struct, enum, etc.), including attributes
#[derive(Debug)]
pub(crate) enum Container {
    ErrorFromPrior {
        identity: Ident,
        prior: Path,
    },
    Full {
        identity: Ident,
        prior: Path,
        error: Path,
    },
}

fn checked_set(
    key: &KnownAttribute,
    value: &Path,
    container: &mut Option<Path>,
) -> Result<(), syn::Error> {
    if let Some(last_set) = container {
        let mut error = syn::Error::new(
            value.span(),
            format!("Duplicate definition for `#[{NAMESPACE}({key})]`."),
        );
        error.combine(syn::Error::new(
            last_set.span(),
            "First definition is here:",
        ));
        return Err(error);
    } else {
        let _ = container.insert(value.clone());
    }
    Ok(())
}

impl Container {
    pub(crate) fn from_ast(input: &syn::DeriveInput) -> syn::Result<Self> {
        let identity = input.ident.clone();
        let mut maybe_prior: Option<syn::Path> = None;
        let mut maybe_error: Option<syn::Path> = None;

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
                    ParsedAttribute::prior(ref ident) => {
                        checked_set(
                            &Into::<KnownAttribute>::into(attr.clone()),
                            ident,
                            &mut maybe_prior,
                        )?;
                    }
                    ParsedAttribute::error(ref ident) => {
                        checked_set(
                            &Into::<KnownAttribute>::into(attr.clone()),
                            ident,
                            &mut maybe_error,
                        )?;
                    }
                }
            }
        }

        if let Some(prior) = &maybe_prior {
            if prior.get_ident().is_some_and(|ident| ident == "None") {
                maybe_prior = Some(identity.clone().into());
            }
        }

        match (maybe_prior, maybe_error) {
            (None, None) | (None, Some(_))=> Err(syn::Error::new(
                identity.span(),
                format!(
                    "Missing required attribute `{prior}`. Use `#[{NAMESPACE}({prior} = <struct>)]` to migrate from another struct or `#[{NAMESPACE}({prior} = None)]` if this is the first struct in the migration chain",
                    prior = KnownAttribute::prior
                ),
            )),
            (Some(prior), None) => Ok(Container::ErrorFromPrior { identity, prior }),
            (Some(prior), Some(error)) => Ok(Container::Full {
                identity,
                prior,
                error,
            }),
        }
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
            r#"Missing required attribute `prior`. Use `#[try_migrate(prior = <struct>)]` to migrate from another struct or `#[try_migrate(prior = None)]` if this is the first struct in the migration chain"#
        );
    }

    #[test]
    fn test_duplicate_attribute() {
        let input = syn::parse_quote! {
            #[try_migrate(prior = MetadataV1)]
            #[try_migrate(prior = MetadataV1)]
            struct MetadataV1 {
            }
        };

        let result = Container::from_ast(&input);
        assert!(&result.is_err(), "Expected an error, got {:?}", &result);
        assert_eq!(
            format!("{}", result.err().unwrap()),
            r#"Duplicate definition for `#[try_migrate(prior)]`."#
        );
    }

    #[test]
    fn test_implicit_error() {
        let input = syn::parse_quote! {
            #[try_migrate(prior = MetadataV1)]
            struct MetadataV2 {}
        };
        let container = Container::from_ast(&input).unwrap();
        assert!(matches!(
            container,
            Container::ErrorFromPrior {
                identity: _,
                prior: _
            }
        ))
    }

    #[test]
    fn test_prior_and_error() {
        let input = syn::parse_quote! {
            #[try_migrate(prior = MetadataV1, error = CustomError)]
            struct MetadataV2 {}
        };

        let container = Container::from_ast(&input).unwrap();
        assert!(matches!(
            container,
            Container::Full {
                identity: _,
                prior: _,
                error: _
            }
        ))
    }

    #[test]
    fn test_unknown_attribute_errors() {
        let input = syn::parse_quote! {
            #[try_migrate(prior = MetadataV1, error = CustomError, unknown = "LOL")]
            struct MetadataV2 {}
        };

        let result = Container::from_ast(&input);
        assert!(&result.is_err(), "Expected an error, got {:?}", result);
        assert_eq!(
            format!("{}", &result.err().unwrap()),
            r#"Unknown try_migrate attribute: `unknown`. Must be one of `prior`, `error`"#
        );
    }

    #[test]
    fn test_prior_none() {
        let input = syn::parse_quote! {
            #[try_migrate(prior = None)]
            struct MetadataV1 {
            }
        };

        let container = Container::from_ast(&input).unwrap();
        assert!(matches!(
            container,
            Container::ErrorFromPrior {
                identity: _,
                prior: _,
            }
        ))
    }

    #[test]
    fn test_prior_explicit_self() {
        let input = syn::parse_quote! {
            #[try_migrate(prior = MetadataV1)]
            struct MetadataV1 {
            }
        };

        let container = Container::from_ast(&input).unwrap();
        assert!(matches!(
            container,
            Container::ErrorFromPrior {
                identity: _,
                prior: _,
            }
        ))
    }
}
