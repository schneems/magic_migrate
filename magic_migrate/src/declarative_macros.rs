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

            fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
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
/// use magic_migrate::Migrate;
#[doc = include_str!("fixtures/personV1_V2.txt")]
///
/// // - Link PersonV1 => PersonV1 and set the toml deserializer
/// // - Link PersonV1 => PersonV2
/// magic_migrate::migrate_toml_chain!(PersonV1, PersonV2);
/// ```
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

            fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
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
/// use magic_migrate::TryMigrate;
#[doc = include_str!("fixtures/try_personV1_V2.txt")]
///
/// magic_migrate::try_migrate_toml_chain!(
///     error: PersonMigrationError,
///     chain: [PersonV1, PersonV2],
/// );
///
/// // Now, given a serialized struct
/// let toml_string = toml::to_string(&PersonV1 {
///     name: "Schneems".to_string(),
///     title: Some("Chief Taco Officer".to_string())
/// })
/// .unwrap();
///
/// // Cannot deserialize PersonV1 toml directly into PersonV2
/// let result = toml::from_str::<PersonV2>(&toml_string);
///  assert!(result.is_err());
///
/// // Can deserialize to PersonV1 then migrate to PersonV2
/// let person: PersonV2 = PersonV2::try_from_str_migrations(&toml_string).unwrap().unwrap();
/// assert_eq!(person.name, "Schneems".to_string());
///
/// // Conversion can fail (missing a Title)
/// let result = PersonV2::try_from_str_migrations(&"name = 'Schneems'").unwrap();
/// assert!(result.is_err());
/// assert!(matches!(result, Err(PersonMigrationError::TitleCannotBeEmpty)));
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
/// use magic_migrate::Migrate;
#[doc = include_str!("fixtures/personV1_V2.txt")]
///
/// magic_migrate::migrate_deserializer_chain!(
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

            fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
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
/// use magic_migrate::TryMigrate;
#[doc = include_str!("fixtures/try_personV1_V2.txt")]
///
/// magic_migrate::try_migrate_deserializer_chain!(
///     deserializer: toml::Deserializer::new,
///     error: PersonMigrationError,
///     chain: [PersonV1, PersonV2],
/// );
///
/// // Now, given a serialized struct
/// let toml_string = toml::to_string(&PersonV1 {
///     name: "Schneems".to_string(),
///     title: Some("Chief Taco Officer".to_string())
/// })
/// .unwrap();
///
/// // Cannot deserialize PersonV1 toml directly into PersonV2
/// let result = toml::from_str::<PersonV2>(&toml_string);
///  assert!(result.is_err());
///
/// // Can deserialize to PersonV1 then migrate to PersonV2
/// let person: PersonV2 = PersonV2::try_from_str_migrations(&toml_string).unwrap().unwrap();
/// assert_eq!(person.name, "Schneems".to_string());
///
/// // Conversion can fail (missing a Title)
/// let result = PersonV2::try_from_str_migrations(&"name = 'Schneems'").unwrap();
/// assert!(result.is_err());
/// assert!(matches!(result, Err(PersonMigrationError::TitleCannotBeEmpty)));
/// ```
#[macro_export]
macro_rules! try_migrate_deserializer_chain {
    // Base case
    (error: $err:ident, deserializer: $deser:path, chain: [$a:ident] $(,)?) => {
        impl TryMigrate for $a {
            type TryFrom = Self;
            type Error = $err;

            fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
                $deser(input)
            }
        }
        impl std::convert::From<std::convert::Infallible> for $err {
            fn from(_value: std::convert::Infallible) -> Self {
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
