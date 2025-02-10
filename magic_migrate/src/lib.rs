#![doc = include_str!("../README.md")]

mod derive_interface;
mod mini_how;
mod traits;
pub use magic_migrate_derive::TryMigrate;
pub use traits::{Migrate, TryMigrate};

/// A generic wrapper when TryFrom::Error is raised on Migration
///
/// Acts somewhat like anyhow::Error, to store `TryFrom::Error`-s
/// as your structs try to migrate. Works with anything that implements
/// `std::error::Error`.
///
/// ```
/// use magic_migrate::{MigrateError, TryMigrate};
///
#[doc = include_str!("fixtures/try_personV1_V2.txt")]
/// # impl magic_migrate::TryMigrate for PersonV1 {
/// #     type TryFrom = PersonV1;
/// #     type Error = MigrateError;
/// #
/// #     fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
/// #         toml::Deserializer::new(input)
/// #     }
/// # }
///
/// impl magic_migrate::TryMigrate for PersonV2 {
///     type TryFrom = PersonV1;
///     type Error = MigrateError;
///
///     fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
///         toml::Deserializer::new(input)
///     }
/// }
///
/// let result: Result<PersonV2, MigrateError> = PersonV2::try_from_str_migrations("name = 'Richard'").unwrap();
/// # assert!(&result.is_err(), "Expected an error, got {:?}", &result);
/// # assert!(matches!(&result, Err(MigrateError)), "Expected MagicMigrateError, got {:?}", &result)
/// ```
pub use mini_how::MagicError as MigrateError;
