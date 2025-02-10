//! Automagically load and migrate deserialized structs to the latest version.
//!
//! > 🎵 If you believe in magic, come along with me
//! >
//! > We'll dance until morning 'til there's just you and me 🎵
//! >
//!
//! ## What
//!
//! Provides a migration path for deserializing older structs into newer ones. For example, if you
//! have a `struct MetadataV1 { name: String }` that is serialized to TOML and loaded,
//! this crate allows you to make a change to things field names without invalidating the already serialized data:
//!
//! ```
//! use magic_migrate::{MigrateError, TryMigrate};
//! use serde::{Deserialize};
//!
//! #[derive(TryMigrate, Debug, Deserialize)]
//! #[try_migrate(from = None)]
//! #[serde(deny_unknown_fields)]
//! struct MetadataV1 { name: String }
//!
//! #[derive(TryMigrate, Debug, Deserialize)]
//! #[try_migrate(from = MetadataV1)]
//! #[serde(deny_unknown_fields)]
//! struct MetadataV2 { full_name: String }
//!
//! impl std::convert::TryFrom<MetadataV1> for MetadataV2 {
//!     type Error = NameIsEmpty;
//!
//!     fn try_from(value: MetadataV1) -> Result<Self, Self::Error> {
//!         if value.name.is_empty() {
//!             Err(NameIsEmpty)
//!         } else {
//!           Ok(MetadataV2 { full_name: value.name })
//!         }
//!     }
//! }
//!
//! #[derive(Debug, thiserror::Error)]
//! #[error("Name cannot be empty")]
//! struct NameIsEmpty;
//!
//! // Note that the field is `name` which `MetadataV2` does not have but V1 does
//! let v2: Result<MetadataV2, MigrateError> =
//!     MetadataV2::try_from_str_migrations("name = 'Richard'").unwrap();
//!
//! assert!(matches!(v2, Ok(MetadataV2 { .. })));
//! ```
//!
//! The main use case is for [building Cloud Native Buildpacks (CNBs) in Rust](https://crates.io/crates/libcnb).
//! In this environment, cache keys are serialized as TOML to disk and if they're unable to be deserialized
//! then the cache is cleared. This [`TryMigrate`] trait gives total flexability to the author to support
//! one or many data layouts.
//!
//! You can see an [interface that relies on this behavior here](https://github.com/heroku/buildpacks-ruby/blob/99305fbf30918b1e0657d7bedbf5cd4859a4eb74/commons/src/layer/diff_migrate.rs#L121).
//!
//! ## Concepts
//!
//! The core migration concept is inspired by [database migrations](https://guides.rubyonrails.org/active_record_migrations.html).
//! Here, the overall change is represented as a series of modifications that can be played in order
//! to reach the final desired data representation. Each change is represented by a [`std::convert::TryFrom`]
//! implementation, and the whole chain of migrations are tied together with [TryMigrate].
//!
//! ## Use
//!
//! ```term
//! $ cargo add magic_migrate
//! ```
//!
//! ## Derive [TryMigrate] quick start
//!
//! The derive macro is enabled by default. To add
//!
//! - Import the trait `use magic_migrate::TryMigrate;`
//! - Add the derive declaration `#[derive(TryMigrate)]` to your structs
//! - Annotate the first struct in the chain with `#[try_migrate(from = None)]`
//! - Annotate the next struct in the chain to point at the one before it e.g. `#[try_migrate(from = MetadataV1)]`
//! - Add a [std::convert::TryFrom] implementation between the two structs.
//!
//! That's all you need to get up and running. Keep reading
//!
//! ## Derive [TryMigrate] details
//!
//! The macro can be configured with attributes on the container (struct).
//!
//! Container Attributes:
//!
//! - `#[try_migrate(from = <previous struct> | None)]` (Required) Tells the struct what previous struct it should migrate from.
//!   When there are no previous structs use `None`.
//! - `#[try_migrate(error = <error enum>)]` (Optional) Tells the [`TryMigrate`] trait how to hold error information
//!   from all [TryFrom] errors in the chain. The default value is [crate::MigrateError] which holds anything that
//!   implements the [`std::error::Error`] trait. It behaves similarly to [Anyhow](https://docs.rs/anyhow/latest/anyhow/).
//!   To provide your own explicit error type see the error section below.
//! - `#[try_migrate(deserializer = <deserializer function>)` (Optional) The default deserialization format is TOML
//!    using the [toml](https://docs.rs/toml/latest/toml/) crate. This interface will likely need to change to
//!   [support adjusting to use different serialization formats](https://github.com/schneems/magic_migrate/issues/16).
//!
//! The macro does not currently allow for any field level customization.
//!
//! Field Attributes:
//!
//! - None
//!
//! ## Derive Error docs
//!
//! You can specify an explicit error using the `#[try_migrate(error = <enum>)]` attribute.
//!
//! This error must be able to hold every error raised by [TryFrom] in the chain. Which
//! includes [std::convert::Infallible] (which is used for the base case as every struct can
//! infallibly migrate to itself).
//!
//! Only the base case must declare a custom error, all other migrations will inherit it by default.
//!
//! ```
//! use magic_migrate::TryMigrate;
//! use serde::{Deserialize};
//!
//! #[derive(TryMigrate, Debug, Deserialize)]
//! #[try_migrate(from = None, error = CustomError )]
//! #[serde(deny_unknown_fields)]
//! struct MetadataV1 { name: String }
//!
//! // ...
//! # #[derive(TryMigrate, Debug, Deserialize)]
//! # #[try_migrate(from = MetadataV1)]
//! # #[serde(deny_unknown_fields)]
//! # struct MetadataV2 { full_name: String }
//!
//! # impl std::convert::TryFrom<MetadataV1> for MetadataV2 {
//! #     type Error = NameIsEmpty;
//!
//! #     fn try_from(value: MetadataV1) -> Result<Self, Self::Error> {
//! #         if value.name.is_empty() {
//! #             Err(NameIsEmpty)
//! #         } else {
//! #           Ok(MetadataV2 { full_name: value.name })
//! #         }
//! #     }
//! # }
//!
//! # #[derive(Debug, thiserror::Error)]
//! # #[error("Name cannot be empty")]
//! # struct NameIsEmpty;
//!
//! #[derive(Debug, thiserror::Error)]
//! enum CustomError {
//!   #[error("Cannot migrate due to error: {0}")]
//!   EmptyName(NameIsEmpty)
//! }
//!
//! impl From<NameIsEmpty> for CustomError {
//!   fn from(value: NameIsEmpty) -> Self {
//!       CustomError::EmptyName(value)
//!   }
//! }
//!
//! impl From<std::convert::Infallible> for CustomError {
//!     fn from(_value: std::convert::Infallible) -> Self {
//!         unreachable!()
//!     }
//! }
//!
//! // Logic is adjusted to return an error
//! let v2: Result<MetadataV2, CustomError> =
//!     MetadataV2::try_from_str_migrations("name = ''").unwrap();
//! assert!(matches!(v2, Err(CustomError::EmptyName(_))));
//! ```
//!
//! ## What won't it do? (The ABA problem)
//!
//! This library cannot ensure that if a `PersonV1` struct was serialized, it cannot be loaded into `PersonV2` without migration. I.e. it does not guarantee that the [`From`] or [`TryFrom`] code was run.
//!
//! For example, if the `PersonV2` struct introduced an `Option<String>` field, instead of `DateTime<Utc>` then the string `"name = 'Richard'"` could be deserialized to either PersonV1 or PersonV2 without needing to call a migration.
//!
//! - [Playground demonstration of the ABA problem](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e26033d3c8c3c34414fe594674f6d053)
//!
//! There are more links in a related discussion in Serde:
//!
//! - [serde-rs/serde issue trying to use `tag` and `deny_unknown_fields`](https://github.com/serde-rs/serde/issues/2666)
//!
//! ## What can you do to harden your code against this (ABA) issue?
//!
//! - Use [deny_unknown_fields](https://serde.rs/container-attrs.html) from serde. This setting prevents silently dropping additional struct fields. This strategy would handle the case where V1 has two fields and V2 has only one field [playground example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=75c6f06234e1d64aea7b37c448321abf). However, it will **not** protect the case where we've added an optional field, [playground example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=47dde9f52b0c5114ef28f35bb019969c).
//! - Add tests that ensure one struct cannot deserialize into a later one in the chain. Writing tests might be difficult if your structs have many optional fields and you want to generate permutations of all of them.
//! - Add a [version marker field](https://stackoverflow.com/a/77700752/147390). This strategy works, but you must notice and keep the field name updated when creating a new struct (possible programmer error). And it will leak an implementation detail to anyone who might see your serialized data (which may or may not matter) to you.
//! - Read these docs and understand the underlying reason why this happens.
//! - If you have another suggestion to harden a codebase, open an issue.
//!
//! ## Other possible "migration" solutions and their differences
//!
//! - Using Serde's [container attributes from and try_from](https://serde.rs/container-attrs.html). This feature only works if you never want to store and deserialize the latest version in the chain. [playground example showing you when this fails](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b6ea1cd054bab5d7df62a04cbd7c6284).
//!
//! Compared to using Serde's `from` and `try_from` container attribute features, magic migrate will always try to convert to the target struct first, then migrate using the latest possible struct in the chain, allowing structs to migrate through the entire chain or storing and using the latest value.
//!
//! - The [Serde version crate](https://docs.rs/serde-version/latest/serde_version/) seems to have overlapping goals. Differences are unclear. If you've tried it, update these docs.

mod declarative_macros;
mod mini_how;
mod traits;

/// See the [`crate`] docs for examples
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
