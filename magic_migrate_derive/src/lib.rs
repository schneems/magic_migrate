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
    let Container {
        identity,
        prior,
        error,
        deserializer,
    } = container;

    // True when it's the first TryMigrate in the chain (prior == self)
    let from_none = prior.get_ident().is_some_and(|ident| ident == &identity);

    let error_type = error.map(|e| quote::quote! {#e}).unwrap_or_else(|| {
        // If not explicit, only the first error in the chain is required.
        // This allows for reduced repetition
        if from_none {
            quote::quote! { magic_migrate::MigrateError }
        } else {
            quote::quote! { <#prior as magic_migrate::TryMigrate>::Error }
        }
    });

    // Default to toml
    let deserializer_fn = deserializer
        .map(|d| quote::quote! { #d(input) })
        .unwrap_or_else(|| {
            // If not explicit, only the first deserializer in the chain is required
            // This allows for reduced repetition
            if from_none {
                quote::quote! { toml::Deserializer::new(input) }
            } else {
                quote::quote! { <Self as magic_migrate::TryMigrate>::TryFrom::deserializer(input) }
            }
        });

    let code = quote::quote! {
        impl TryMigrate for #identity {
            type TryFrom = #prior;
            type Error = #error_type;

            fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
                #deserializer_fn
            }
        }
    };
    Ok(code)
}
