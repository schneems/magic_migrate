#[deprecated(
    since = "1.1.0",
    note = "Please use the `#[derive(TryMigrate, error = std::convert::Infallible)]` macro instead"
)]
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

#[deprecated(
    since = "1.1.0",
    note = "Please use the `#[derive(TryMigrate, error = std::convert::Infallible)]` macro instead"
)]
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

#[deprecated(
    since = "1.1.0",
    note = "Please use the `#[derive(TryMigrate)]` macro instead"
)]
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

#[deprecated(
    since = "1.1.0",
    note = "Please use the `#[derive(TryMigrate)]` macro instead"
)]
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

#[deprecated(
    since = "1.1.0",
    note = "Please use the `#[derive(TryMigrate, error = std::convert::Infallible)]` macro instead"
)]
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

#[deprecated(
    since = "1.1.0",
    note = "Please use the `#[derive(TryMigrate)]` macro instead"
)]
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
