use serde::de::DeserializeOwned;
use serde::Deserializer;
use std::any::{Any, TypeId};
use std::fmt::Debug;

/// Magic Migrate: Automagically load and migrate deserialized structs to the latest version
///
/// Problem: Let's say that you made a struct that serializes to disk somehow,
/// perhaps it uses toml. Now, let's say that you want to add a new field to that
/// struct, but you don't want to lose older persisted data. What ever should you do?
///
/// You can define how to convert from one struct to another using either `From` or
/// `TryFrom` then tell Rust how to migrate from one to the next via `Migrate` or `TryMigrate`
/// traits. Now, when you try to load data into the current struct it will follow a chain
/// of structs in reverse order to find the first one that successfully serializes. When
/// that happens, it will convert that struct to the latest version for you. It's magic!
/// (It's actually mostly clever use of trait boundries, but whatever).
///
/// This library was created to handle the case of serialized metadata stored in
/// layers in a https://github.com/heroku/libcnb.rs buildpack. To that end, it
/// includes a helpful macro to define a chain of migrations for you:
///
/// ```rust
/// use magic_migrate::{Migrate, migrate_toml_chain};
///
/// use serde::{Deserialize, Serialize, de::Deserializer};
/// use chrono::{DateTime, Utc};
///
/// #[derive(Deserialize, Serialize, Debug)]
/// struct PersonV1 {
///     name: String
/// }
///
/// #[derive(Deserialize, Serialize, Debug)]
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
/// // Then specify the order of the migrations from left to right
/// migrate_toml_chain!(PersonV1, PersonV2);
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
///
/// See trait definitions `Migrate` and `TryMigrate` for additional docs.

/// Use the `Migrate` trait when structs can be infallibly migrated
/// from one version to the next.
///
/// ```rust
/// use magic_migrate::{Migrate};
///
/// use serde::{Deserialize, Serialize, de::Deserializer};
/// use chrono::{DateTime, Utc};
///
/// #[derive(Deserialize, Serialize, Debug)]
/// struct PersonV1 {
///     name: String
/// }
///
/// #[derive(Deserialize, Serialize, Debug)]
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

/// Use the `TryMigrate` trait when structs CANNOT be infallibly migrated
/// from one version to the next and an error may be returned.
///
/// ```rust
/// use magic_migrate::{TryMigrate};
///
/// use serde::{Deserialize, Serialize, de::Deserializer};
/// use chrono::{DateTime, Utc};
/// use std::convert::Infallible;
///
/// #[derive(Deserialize, Serialize, Debug)]
/// struct PersonV1 {
///     name: String
/// }
///
/// #[derive(Deserialize, Serialize, Debug)]
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
/// // error types. The first struct in the chain always
/// // references itself, so the error type must always
/// // support `From<Infallibly>`
/// #[derive(Debug, Eq, PartialEq)]
/// enum PersonMigrationError {
///     NotRichard(NotRichard),
///     Infallible
/// }
///
/// impl From<Infallible> for PersonMigrationError {
///     fn from(_value: Infallible) -> Self {
///         PersonMigrationError::Infallible
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
/// let person: PersonV2 = PersonV2::try_from_str_migrations(&toml_string).unwrap().unwrap();
/// assert_eq!(person.name, "Schneems".to_string());
///
/// // Conversion can fail
/// let result = PersonV2::try_from_str_migrations(&"name = 'Cinco'").unwrap();
/// assert!(result.is_err());
/// ```
pub trait TryMigrate: TryFrom<Self::TryFrom> + Any + DeserializeOwned + Debug {
    type TryFrom: TryMigrate;

    /// Tell magic migrate how you want to deserialize your strings
    /// into structs
    fn deserializer<'de>(input: &str) -> impl Deserializer<'de>;

    type Error: From<<Self as TryFrom<<Self as TryMigrate>::TryFrom>>::Error>
        + From<<<Self as TryMigrate>::TryFrom as TryMigrate>::Error>;

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

/// Implement `TryMigrate` for all structs that infailably
/// can `Migrate`.
///
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

/// Macro for linking structs together in a migration chain
/// without defining the first Self migration in the chain
/// or the deserializer.
///
/// i.e. it Links A => B, B => C etc. without linking A => A
///
/// Relies on A => A to define the type of deserializer.
/// That means this can be reused for any deserializer you want.
///
/// See `migrate_toml_chain` for an example of how to build a macro
/// for your own deserializer
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
        $crate::migrate_link!($b, $($rest),*)
    )
}

/// See module docs for an example use case
#[macro_export(local_inner_macros)]
macro_rules! migrate_toml_chain {
    // Base case
    // Start of the migration chain
    // In A => B => C, we must define the A => A case first.
    ($a:ident) => (
        impl Migrate for $a {
            type From = Self;

            fn deserializer<'de>(input: &str) -> impl Deserializer<'de> {
                toml::Deserializer::new(input)
            }
        }
    );
    ($a:ident, $($rest:ident),+) => (
        // Call the base case to link A => A
        migrate_toml_chain!($a);
        // Link the rest i.e. A => B, B => C, etc.
        migrate_link!($a, $($rest),+);
    )
}

/// A macro to help define `TryMigrate` based migrations
///
/// ```rust
/// use magic_migrate::{TryMigrate, try_migrate_link};
///
/// use serde::{Deserialize, Serialize, de::Deserializer};
/// use chrono::{DateTime, Utc};
/// use std::convert::Infallible;
///
/// #[derive(Deserialize, Serialize, Debug)]
/// struct PersonV1 {
///     name: String
/// }
///
/// #[derive(Deserialize, Serialize, Debug)]
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
/// // error types. The first struct in the chain always
/// // references itself, so the error type must always
/// // support `From<Infailable>`
/// #[derive(Debug, Eq, PartialEq)]
/// enum PersonMigrationError {
///     NotRichard(NotRichard),
///     Infallible
/// }
///
/// impl From<Infallible> for PersonMigrationError {
///     fn from(_value: Infallible) -> Self {
///         PersonMigrationError::Infallible
///     }
/// }
///
/// impl From<NotRichard> for PersonMigrationError {
///     fn from(value: NotRichard) -> Self {
///         PersonMigrationError::NotRichard(value)
///     }
/// }
///
/// // Unlike `try_toml_migrate_chain!` this macro does not specify
/// // the first "self" migration in the chain. We have to do that
/// // manually. Use this to define the associated error and
/// // specify a deserializer
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
/// // All future migrations can be defined using the macro
/// try_migrate_link!(PersonV1, PersonV2);
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
        $crate::try_migrate_link!($b, $($rest),*)
    )
}
