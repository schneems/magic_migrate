use serde::de::DeserializeOwned;
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
/// use magic_migrate::Migrate;
///
#[doc = include_str!("fixtures/personV1_V2.txt")]
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
///     fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
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
///     fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
///         <Self as Migrate>::From::deserializer(input)
///     }
/// }
///
/// // That's it! This is basically the same thing that the [`migrate_toml_chain`]
/// // macro did for you, but using the trait directly allows for any deserializer
/// // you want.
///
/// // Now, given a serialized struct
/// let toml_string = toml::to_string(&PersonV1 {
///     name: "Schneems".to_string(),
/// }).unwrap();
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

    fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de>;

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
/// use magic_migrate::TryMigrate;
#[doc = include_str!("fixtures/try_personV1_V2.txt")]
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
///     fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
///         toml::Deserializer::new(input)
///     }
/// }
///
/// // The first struct references itself, in the chain (it's how we know
/// // to stop iterating). A by-product is that the error in `TryMigrate`
/// // must be able to take `Infallible` even though that error cannot be raised
/// impl From<std::convert::Infallible> for PersonMigrationError {
///     fn from(_value: std::convert::Infallible) -> Self {
///         unreachable!();
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
///     fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
///         <Self as TryMigrate>::TryFrom::deserializer(input)
///     }
/// }
///
/// // That's it! Now, you can use it.
///
/// // Given a serialized struct
/// let toml_string = toml::to_string(&PersonV1 {
///     name: "Schneems".to_string(),
///     title: Some("Señor Developer".to_string())
/// }).unwrap();
///
/// // Cannot deserialize PersonV1 toml into PersonV2
/// let result = toml::from_str::<PersonV2>(&toml_string);
///  assert!(result.is_err());
///
/// // Can deserialize to PersonV1 then migrate to PersonV2
/// let person: PersonV2 = PersonV2::try_from_str_migrations(&toml_string).unwrap().unwrap();
/// assert_eq!(person.name, "Schneems".to_string());
///
/// // Conversion can fail (missing title)
/// let result = PersonV2::try_from_str_migrations(&"name = 'Schneems'").unwrap();
/// assert!(result.is_err());
/// ```
pub trait TryMigrate: TryFrom<Self::TryFrom> + Any + DeserializeOwned + Debug {
    type TryFrom: TryMigrate;

    /// Tell magic migrate how you want to deserialize your strings
    /// into structs
    fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de>;

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

    fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
        <Self as Migrate>::deserializer(input)
    }

    type Error = std::convert::Infallible;
}
