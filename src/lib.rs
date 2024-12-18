#![doc = include_str!("../README.md")]

use serde::de::DeserializeOwned;
use serde::Deserializer;
use std::any::{Any, TypeId};
use std::fmt::{Debug, Display};

/// Use the [`Migrate`] trait when structs can be infallibly migrated
/// from one version to the next. Use the [`TryMigrate`] trait when
/// struct migration may fail.
///
/// To help you out, you can also use the following macros:
///
/// - [`migrate_toml_chain!`] to link structs together in a migration chain for TOML data.
/// - [`migrate_deserializer_chain!`] to link structs together in a migration chain, BYO deserializer.
///
/// These macros essentially automate the process you'll see below.
///
/// Each [`Migrate`] implementation will create one link. To build a
/// complete chain, you will need 2 or more structs linked together.
/// The first struct in the chain must be linked to itself to indicate
/// it is aware it's being used in the verison migration pattern, and
/// to assure us that there's no version that comes before it.
///
/// If you cannot infallibly convert from one struct to another
/// you can implement the [`TryMigrate`] trait instead.
///
/// Both migration traits can be used with any deserializer format (i.e. toml,
/// json, YAML, etc.). To create a migration, you'll have to tell Rust which
/// deserializer you want to use.
///
/// Also see:
///   - [`migrate_link`] macro for quickly building all links but the first
///   - [`migrate_toml_chain`] macro for building an entire chain with the toml deserializer
///
/// For infailable migrations, you can use the [`Migrate`] trait. For failable migrations,
/// use the [`TryMigrate`] trait.
///
/// ## Infailable migration Example ([`Migrate`] trait)
///
/// ```rust
/// use magic_migrate::{Migrate};
///
/// use serde::{Deserialize, Serialize, de::Deserializer};
/// use chrono::{DateTime, Utc};
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV1 {
///     name: String
/// }
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV2 {
///     name: String,
///     updated_at: DateTime<Utc>
/// }
///
/// // First define how to map from one struct to another
/// impl From<PersonV1> for PersonV2 {
///     fn from(value: PersonV1) -> Self {
///         PersonV2 {
///             name: value.name.clone(),
///             updated_at: Utc::now()
///         }
///     }
/// }
///
/// // First define a migration on the beginning of the chain
/// //
/// // In this scenario `PersonV1` only converts from itself.
/// //
/// // Implement the `deserializer` function to tell magic migrate
/// // the data format that the input string will be in. In this case
/// // we are using `toml`.
/// impl Migrate for PersonV1 {
///     type From = Self;
///
///     fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
///         toml::Deserializer::new(input)
///     }
/// }
///
/// // Now define the second link in the migration chain by
/// // specifying that `PersonV2` can be built from `PersonV1`.
/// //
/// // The deserializer function body can be reused from `PersonV1`
/// impl Migrate for PersonV2 {
///     type From = PersonV1;
///
///     fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
///         <Self as Migrate>::From::deserializer(input)
///     }
/// }
///
/// // That's it! This is basically the same thing that the `migrate_toml_chain`
/// // macro did for you, but using the trait directly allows for any deserializer
/// // you want.
///
/// // Now, given a serialized struct
/// let toml_string = toml::to_string(&PersonV1 { name: "Schneems".to_string() }).unwrap();
///
/// // Cannot deserialize PersonV1 toml into PersonV2
/// let result = toml::from_str::<PersonV2>(&toml_string);
///  assert!(result.is_err());
///
/// // Can deserialize to PersonV1 then migrate to PersonV2
/// let person: PersonV2 = PersonV2::from_str_migrations(&toml_string).unwrap();
/// assert_eq!(person.name, "Schneems".to_string());
/// ```
pub trait Migrate: From<Self::From> + Any + DeserializeOwned + Debug {
    type From: Migrate;

    fn deserializer<'de>(input: &str) -> impl Deserializer<'de>;

    fn from_str_migrations(input: &str) -> Option<Self> {
        if let Ok(instance) = Self::deserialize(Self::deserializer(input)) {
            Some(instance)
        } else if TypeId::of::<Self>() == TypeId::of::<Self::From>() {
            return None;
        } else {
            <Self::From as Migrate>::from_str_migrations(input).map(Into::into)
        }
    }
}

/// Use the [`TryMigrate`] trait when structs CANNOT be infallibly migrated
/// from one version to the next and an error may be returned. For infallible
/// migration see [`Migrate`].
///
/// To help you out, you can use the following macros:
///
/// - [`try_migrate_toml_chain!`] to link structs together in a migration chain for TOML data.
/// - [`try_migrate_deserializer_chain!`] to link structs together in a migration chain, BYO deserializer.
///
/// Like [`Migrate`] each implementation of this trait creates a link in a migration
/// chain. To have a full chain, the first struct must implement this trait
/// ([`TryMigrate`]) on itself.
///
/// In addition to specifying the struct links and the deserializer (like [`Migrate`])
/// you must also specify what error to return when the migration chain fails. This
/// error must be able to hold any error created in the migration chain.
///
/// In practice that means [`From`] must be implemented
/// on error types in the migration chain going into the error.
//
/// # Example
///
/// ```rust
/// use magic_migrate::{TryMigrate};
///
/// use serde::{Deserialize, Serialize, de::Deserializer};
/// use chrono::{DateTime, Utc};
/// use std::convert::Infallible;
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV1 {
///     name: String
/// }
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV2 {
///     name: String,
///     updated_at: DateTime<Utc>
/// }
///
/// // First define how to map from one struct to another
/// impl TryFrom<PersonV1> for PersonV2 {
///     type Error = NotRichard;
///
///     fn try_from(value: PersonV1) -> Result<Self, NotRichard> {
///         if &value.name == "Schneems" {
///             Ok(PersonV2 {
///                     name: value.name.clone(),
///                     updated_at: Utc::now()
///                })
///         } else {
///             Err(NotRichard { name: value.name.clone() })
///         }
///     }
/// }
///
/// #[derive(Debug, Eq, PartialEq)]
/// struct NotRichard {
///   name: String
/// }
///
/// // Create an error struct for return type
/// //
/// // Because the migration can fail we need to resolve
/// // error types.
/// #[derive(Debug, thiserror::Error, Eq, PartialEq)]
/// enum PersonMigrationError {
///     #[error("Not Richard {0:?}")]
///     NotRichard(NotRichard),
/// }
///
///
/// // The first struct in the chain always
/// // references itself, so the error type must always
/// // support `From<Infallible>`
/// impl From<Infallible> for PersonMigrationError {
///     fn from(_value: Infallible) -> Self {
///       unreachable!();
///     }
/// }
///
/// impl From<NotRichard> for PersonMigrationError {
///     fn from(value: NotRichard) -> Self {
///         PersonMigrationError::NotRichard(value)
///     }
/// }
///
/// // First define a migration on the beginning of the chain
/// //
/// // In this scenario `PersonV1` only converts from itself.
/// //
/// // Implement the `deserializer` function to tell magic migrate
/// // the data format that the input string will be in. In this case
/// // we are using `toml`.
/// impl TryMigrate for PersonV1 {
///     type TryFrom = Self;
///     type Error = PersonMigrationError;
///
///     fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
///         toml::Deserializer::new(input)
///     }
/// }
///
/// // Now define the second link in the migration chain by
/// // specifying that `PersonV2` can be built from `PersonV1`.
/// //
/// // The deserializer function body can be reused from `PersonV1`
/// impl TryMigrate for PersonV2 {
///     type TryFrom = PersonV1;
///     type Error = PersonMigrationError;
///
///     fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
///         <Self as TryMigrate>::TryFrom::deserializer(input)
///     }
/// }
///
/// // That's it! Now, you can use it.
///
/// // Given a serialized struct
/// let toml_string = toml::to_string(&PersonV1 { name: "Schneems".to_string() }).unwrap();
///
/// // Cannot deserialize PersonV1 toml into PersonV2
/// let result = toml::from_str::<PersonV2>(&toml_string);
///  assert!(result.is_err());
///
/// // Can deserialize to PersonV1 then migrate to PersonV2
/// let person: PersonV2 = PersonV2::try_from_str_migrations(&toml_string).unwrap().unwrap();
/// assert_eq!(person.name, "Schneems".to_string());
///
/// // Conversion can fail
/// let result = PersonV2::try_from_str_migrations(&"name = 'Should Fail'").unwrap();
/// assert!(result.is_err());
/// ```
pub trait TryMigrate: TryFrom<Self::TryFrom> + Any + DeserializeOwned + Debug {
    type TryFrom: TryMigrate;

    /// Tell magic migrate how you want to deserialize your strings
    /// into structs
    fn deserializer<'de>(input: &str) -> impl Deserializer<'de>;

    type Error: From<<Self as TryFrom<<Self as TryMigrate>::TryFrom>>::Error>
        + From<<<Self as TryMigrate>::TryFrom as TryMigrate>::Error>
        + Display
        + Debug;

    #[must_use]
    fn try_from_str_migrations(input: &str) -> Option<Result<Self, <Self as TryMigrate>::Error>> {
        if let Ok(instance) = Self::deserialize(Self::deserializer(input)) {
            Some(Ok(instance))
        } else if TypeId::of::<Self>() == TypeId::of::<Self::TryFrom>() {
            return None;
        } else {
            <Self::TryFrom as TryMigrate>::try_from_str_migrations(input).map(|inner| {
                inner
                    .map_err(Into::into)
                    .and_then(|before: <Self as TryMigrate>::TryFrom| {
                        Self::try_from(before).map_err(Into::into)
                    })
            })
        }
    }
}

/// Implement [`TryMigrate`] for all structs that infailably
/// can [`Migrate`].
impl<T> TryMigrate for T
where
    T: Migrate,
{
    type TryFrom = <Self as Migrate>::From;

    fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
        <Self as Migrate>::deserializer(input)
    }

    type Error = std::convert::Infallible;
}

/// Macro for linking structs together in an infallible [`Migrate`] migration chain
/// without defining the first migration in the chain
/// or the deserializer.
///
/// i.e. it Links A => B, B => C etc. without linking A => A
///
/// Relies on A => A to define the type of deserializer.
/// That means this can be reused for any deserializer you want.
///
/// For a fallible migration chain see [`try_migrate_link`].
///
/// This macro is used by higher level macros like:
///
/// - [`migrate_toml_chain!`] for TOML migrations
/// - [`migrate_deserializer_chain!`] for migrations with a custom deserializer
#[macro_export]
macro_rules! migrate_link {
    // Base case, defines the trait
    // Links a single pair i.e. A => B
    ($a:ident, $b:ident) => (
        impl Migrate for $b {
            type From = $a;

            fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
                <Self as Migrate>::From::deserializer(input)
            }
        }
    );
    ($a:ident, $b:ident, $($rest:ident),+) => (
        // Call the base case to link A => B
        $crate::migrate_link!($a, $b);

        // Link B => C, and the rest
        $crate::migrate_link!($b, $($rest),*);
    );
}

/// Links each struct passed in to each other to build a [`Migrate`] link chain.
/// Including creating the first "self" migration which defines the deserializer
/// to be TOML.
///
/// To BYO deserializer use [`migrate_deserializer_chain!`]. For a failible migration
/// use [`try_migrate_toml_chain!`].
///
/// ## Example
///
/// See [`crate`] module docs for a full example use case
///
/// ```no_run
/// use magic_migrate::{Migrate, migrate_toml_chain};
///
/// # use serde::{Deserialize, Serialize, de::Deserializer};
/// # #[derive(Deserialize, Serialize, Debug)]
/// # struct UserV1;
/// #
/// # #[derive(Deserialize, Serialize, Debug)]
/// # struct UserV2;
/// # impl From<UserV1> for UserV2 {
/// #   fn from(value: UserV1) -> Self {
/// #     unimplemented!();
/// #   }
/// # }
/// #
/// # #[derive(Deserialize, Serialize, Debug)]
/// # struct UserV3;
/// # impl From<UserV2> for UserV3 {
/// #   fn from(value: UserV2) -> Self {
/// #     unimplemented!();
/// #   }
/// # }
///
/// // ...
///
/// // - Link UserV1 => UserV1 and set the toml deserializer
/// // - Link UserV1 => UserV2
/// // - Link UserV2 => UserV3
/// migrate_toml_chain!(UserV1, UserV2, UserV3);
/// ```
///
#[macro_export(local_inner_macros)]
macro_rules! migrate_toml_chain {
    // Base case
    // Start of the migration chain
    // In A => B => C, we must define the A => A case first.
    ($a:ident) => (
        $crate::migrate_deserializer_chain!(
            deserializer: toml::Deserializer::new,
            chain: [$a]
        );
    );
    ($a:ident, $($rest:ident),+) => (
        $crate::migrate_deserializer_chain!(
            deserializer: toml::Deserializer::new,
            chain: [$a, $($rest),+]
        );
    );
}

/// Macro for linking structs together in an infallible [`TryMigrate`] migration chain
/// without defining the first migration in the chain
/// or the deserializer.
///
/// i.e. it Links A => B, B => C etc. without linking A => A
///
/// Relies on A => A to define the type of deserializer.
/// That means this can be reused for any deserializer you want.
///
/// For a infallible migration chain see [`migrate_link`].
///
/// This macro is used by higher level macros like:
///
/// - [`try_migrate_toml_chain!`] for TOML migrations
/// - [`try_migrate_deserializer_chain!`] for migrations with a custom deserializer
#[macro_export]
macro_rules! try_migrate_link {
    // Base case, defines the trait
    // Links a single pair i.e. A => B
    ($a:ident, $b:ident) => (
        impl TryMigrate for $b {
            type TryFrom = $a;
            type Error = <<Self as TryMigrate>::TryFrom as TryMigrate>::Error;

            fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
                <Self as TryMigrate>::TryFrom::deserializer(input)
            }
        }
    );
    ($a:ident, $b:ident, $($rest:ident),+) => (
        // Call the base case to link A => B
        $crate::try_migrate_link!($a, $b);

        // Link B => C, and the rest
        $crate::try_migrate_link!($b, $($rest),*);
    );
}

/// A macro to help define [`TryMigrate`] based migrations
///
/// To use a different deserializer use [`try_migrate_deserializer_chain!`].
/// To define infallible migrations use [`migrate_toml_chain!`].
///
/// # Example
///
/// ```rust
/// use magic_migrate::{TryMigrate, try_migrate_toml_chain};
///
/// use serde::{Deserialize, Serialize, de::Deserializer};
/// use chrono::{DateTime, Utc};
/// use std::convert::Infallible;
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV1 {
///     name: String
/// }
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV2 {
///     name: String,
///     updated_at: DateTime<Utc>
/// }
///
/// // First define how to map from one struct to another
/// impl TryFrom<PersonV1> for PersonV2 {
///     type Error = NotRichard;
///
///     fn try_from(value: PersonV1) -> Result<Self, NotRichard> {
///         if &value.name == "Schneems" {
///             Ok(PersonV2 {
///                     name: value.name.clone(),
///                     updated_at: Utc::now()
///                })
///         } else {
///             Err(NotRichard { name: value.name.clone() })
///         }
///     }
/// }
///
/// #[derive(Debug, Eq, PartialEq)]
/// struct NotRichard {
///   name: String
/// }
///
/// impl From<NotRichard> for PersonMigrationError {
///     fn from(value: NotRichard) -> Self {
///         PersonMigrationError::NotRichard(value)
///     }
/// }
///
/// #[derive(thiserror::Error, Debug, Eq, PartialEq)]
/// enum PersonMigrationError {
///     #[error("Not Richard {0:?}")]
///     NotRichard(NotRichard),
/// }
///
/// try_migrate_toml_chain!(
///     error: PersonMigrationError,
///     chain: [PersonV1, PersonV2],
/// );
///
/// // Now, given a serialized struct
/// let toml_string = toml::to_string(&PersonV1 { name: "Schneems".to_string() }).unwrap();
///
/// // Cannot deserialize PersonV1 toml into PersonV2
/// let result = toml::from_str::<PersonV2>(&toml_string);
///  assert!(result.is_err());
///
/// // Can deserialize to PersonV1 then migrate to PersonV2
/// let person: PersonV2 = PersonV2::try_from_str_migrations(&toml_string).unwrap().unwrap();
/// assert_eq!(person.name, "Schneems".to_string());
///
/// // Conversion can fail
/// let result = PersonV2::try_from_str_migrations(&"name = 'Cinco'").unwrap();
/// assert!(result.is_err());
/// ```
#[macro_export]
macro_rules! try_migrate_toml_chain {
    // Base case
    (error: $err:ident, chain: [$a:ident] $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: toml::Deserializer::new, chain: [$a]);
    };
    // Position variant
    (chain: [$a:ident], error: $err:ident $(,)?) => {
        $crate::try_migrate_toml_chain!(error: $err, chain: [$a]);
    };
    // Rest case
    (error: $err:ident, chain: [$a:ident, $($rest:ident),+] $(,)?) => (
        // Call the base case to link A => A
        $crate::try_migrate_toml_chain!(error: $err, chain: [$a]);

        // Link the rest i.e. A => B, B => C, etc.
        $crate::try_migrate_link!($a, $($rest),+);
    );
    // Position variant
    (chain: [$a:ident, $($rest:ident),+], error: $err:ident $(,)?) => (
        $crate::try_migrate_toml_chain!(error: $err, chain: [$a, $($rest),+]);
    );
}

/// A macro to help define infallible [`Migrate`] based migrations with an arbitrary deserializer.
///
/// The argument passed to `deserializer:` in the macro should be a function that returns an `impl Deserializer`.
///
/// For a fallible migration chain see [`try_migrate_deserializer_chain!`].
///
/// # Example
///
/// ```rust
/// use magic_migrate::{Migrate, migrate_deserializer_chain};
///
/// use serde::{Deserialize, Serialize, de::Deserializer};
/// use chrono::{DateTime, Utc};
/// use std::convert::Infallible;
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV1 {
///     name: String
/// }
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV2 {
///     name: String,
///     updated_at: DateTime<Utc>
/// }
///
/// // First define how to map from one struct to another
/// impl From<PersonV1> for PersonV2 {
///     fn from(value: PersonV1) -> Self {
///         PersonV2 {
///             name: value.name.clone(),
///             updated_at: Utc::now()
///         }
///     }
/// }
///
/// migrate_deserializer_chain!(
///     deserializer: toml::Deserializer::new,
///     chain: [PersonV1, PersonV2],
/// );
///
/// // Now, given a serialized struct
/// let toml_string = toml::to_string(&PersonV1 { name: "Schneems".to_string() }).unwrap();
///
/// // Cannot deserialize PersonV1 toml into PersonV2
/// let result = toml::from_str::<PersonV2>(&toml_string);
///  assert!(result.is_err());
///
/// // Can deserialize to PersonV1 then migrate to PersonV2
/// let person: PersonV2 = PersonV2::from_str_migrations(&toml_string).unwrap();
/// assert_eq!(person.name, "Schneems".to_string());
/// ```
#[macro_export]
macro_rules! migrate_deserializer_chain {
    // Base case
    (deserializer: $deser:path, chain: [$a:ident] $(,)?) => {
        impl Migrate for $a {
            type From = Self;

            fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
                $deser(input)
            }
        }
    };
    // Rest case
    (deserializer: $deser:path, chain: [$a:ident, $($rest:ident),+] $(,)?) => (
        // Call the base case to link A => A
        $crate::migrate_deserializer_chain!(deserializer: $deser, chain: [$a]);

        // Link the rest i.e. A => B, B => C, etc.
        $crate::migrate_link!($a, $($rest),+);
    );

    // Base case variants
    (chain: [$a:ident], deserializer: $deser:path $(,)?) => {
        $crate::migrate_deserializer_chain!(deserializer: $deser, chain: [$a]);
    };
    // Rest case variants
    (chain: [$a:ident, $($rest:ident),+] , deserializer: $deser:path $(,)?) => {
        $crate::migrate_deserializer_chain!(deserializer: $deser, chain: [$a, $($rest),+]);
    };
}

/// A macro to help define [`TryMigrate`] based migrations with an arbitrary deserializer.
///
/// The argument passed to `deserializer:` in the macro should be a function that returns an `impl Deserializer`.
///
/// The argument passed to `error:` in the macro should be the error type that every error from the [`TryFrom`] implementations
/// can be coherced [`Into`].
///
/// For an infallible migration chain see [`migrate_deserializer_chain!`].
///
/// ## Example
///
/// ```rust
/// use magic_migrate::{TryMigrate, try_migrate_deserializer_chain};
///
/// use serde::{Deserialize, Serialize, de::Deserializer};
/// use chrono::{DateTime, Utc};
/// use std::convert::Infallible;
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV1 {
///     name: String
/// }
///
/// #[derive(Deserialize, Serialize, Debug)]
/// #[serde(deny_unknown_fields)]
/// struct PersonV2 {
///     name: String,
///     updated_at: DateTime<Utc>
/// }
///
/// // First define how to map from one struct to another
/// impl TryFrom<PersonV1> for PersonV2 {
///     type Error = NotRichard;
///
///     fn try_from(value: PersonV1) -> Result<Self, NotRichard> {
///         if &value.name == "Schneems" {
///             Ok(PersonV2 {
///                     name: value.name.clone(),
///                     updated_at: Utc::now()
///                })
///         } else {
///             Err(NotRichard { name: value.name.clone() })
///         }
///     }
/// }
///
/// #[derive(Debug, Eq, PartialEq)]
/// struct NotRichard {
///   name: String
/// }
///
/// impl From<NotRichard> for PersonMigrationError {
///     fn from(value: NotRichard) -> Self {
///         PersonMigrationError::NotRichard(value)
///     }
/// }
///
/// #[derive(thiserror::Error, Debug, Eq, PartialEq)]
/// enum PersonMigrationError {
///     #[error("Not Richard {0:?}")]
///     NotRichard(NotRichard),
/// }
///
/// try_migrate_deserializer_chain!(
///     deserializer: toml::Deserializer::new,
///     error: PersonMigrationError,
///     chain: [PersonV1, PersonV2],
/// );
///
/// // Now, given a serialized struct
/// let toml_string = toml::to_string(&PersonV1 { name: "Schneems".to_string() }).unwrap();
///
/// // Cannot deserialize PersonV1 toml into PersonV2
/// let result = toml::from_str::<PersonV2>(&toml_string);
///  assert!(result.is_err());
///
/// // Can deserialize to PersonV1 then migrate to PersonV2
/// let person: PersonV2 = PersonV2::try_from_str_migrations(&toml_string).unwrap().unwrap();
/// assert_eq!(person.name, "Schneems".to_string());
///
/// // Conversion can fail
/// let result = PersonV2::try_from_str_migrations(&"name = 'Cinco'").unwrap();
/// assert!(result.is_err());
/// ```
#[macro_export]
macro_rules! try_migrate_deserializer_chain {
    // Base case
    (error: $err:ident, deserializer: $deser:path, chain: [$a:ident] $(,)?) => {
        impl TryMigrate for $a {
            type TryFrom = Self;
            type Error = $err;

            fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
                $deser(input)
            }
        }
        impl From<Infallible> for $err {
            fn from(_value: Infallible) -> Self {
                unreachable!();
            }
        }
    };
    // Rest case
    (error: $err:ident, deserializer: $deser:path, chain: [$a:ident, $($rest:ident),+] $(,)?) => (
        // Call the base case to link A => A
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a]);

        // Link the rest i.e. A => B, B => C, etc.
        $crate::try_migrate_link!($a, $($rest),+);
    );

    // Base case variants
    (error: $err:ident, chain: [$a:ident], deserializer: $deser:path $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a]);
    };
    (chain: [$a:ident], deserializer: $deser:path, error: $err:ident $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a]);
    };
    (chain: [$a:ident], error: $err:ident, deserializer: $deser:path $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a]);
    };
    (deserializer: $deser:path, chain: [$a:ident], error: $err:ident $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a]);
    };
    (deserializer: $deser:path, error: $err:ident, chain: [$a:ident], $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a]);
    };
    // Rest case variants
    (error: $err:ident, chain: [$a:ident, $($rest:ident),+] , deserializer: $deser:path $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a, $($rest),+]);
    };
    (chain: [$a:ident, $($rest:ident),+], deserializer: $deser:path, error: $err:ident $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a, $($rest),+]);
    };
    (chain: [$a:ident, $($rest:ident),+], error: $err:ident, deserializer: $deser:path $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a, $($rest),+]);
    };
    (deserializer: $deser:path, chain: [$a:ident, $($rest:ident),+], error: $err:ident $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a, $($rest),+]);
    };
    (deserializer: $deser:path, error: $err:ident, chain: [$a:ident, $($rest:ident),+] $(,)?) => {
        $crate::try_migrate_deserializer_chain!(error: $err, deserializer: $deser, chain: [$a, $($rest),+]);
    };
}
