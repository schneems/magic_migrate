use container_attributes::Container;
use proc_macro::TokenStream;
use syn::DeriveInput;
mod container_attributes;

#[proc_macro_derive(TryMigrate, attributes(try_migrate))]
pub fn try_migrate(item: TokenStream) -> TokenStream {
    create_try_migrate(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn create_try_migrate(item: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let ast: DeriveInput = syn::parse2(item)?;
    let container = Container::from_ast(&ast)?;
    let error = match &container {
        Container::ErrorFromPrior { identity, prior } => {
            quote::quote! { <#identity as TryFrom<#prior>>::Error }
        }
        Container::Full {
            identity: _,
            prior: _,
            error,
        } => quote::quote! { #error },
    };

    let code = match container {
        Container::ErrorFromPrior { identity, prior }
        | Container::Full {
            identity,
            prior,
            error: _,
        } => {
            quote::quote! {
                impl std::convert::From<std::convert::Infallible> for #error {
                    fn from(_value: std::convert::Infallible) -> Self {
                        unreachable!();
                    }
                }

                impl TryMigrate for #identity {
                    type TryFrom = #prior;
                    type Error = #error;

                    fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
                        toml::Deserializer::new(input)
                    }
                }
            }
        }
    };
    Ok(code)
}
