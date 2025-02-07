// use magic_migrate::TryMigrate;
// use std::convert::TryFrom;

// #[derive(TryMigrate, serde::Deserialize, Debug)]
// #[try_migrate(prior = None)]
// struct MetadataV1 {
//     name: String,
// }

// #[derive(TryMigrate, serde::Deserialize, Debug)]
// #[try_migrate(prior = MetadataV1)]
// struct MetadataV2 {
//     full_name: String,
// }

// #[derive(TryMigrate, serde::Deserialize, Debug)]
// #[try_migrate(prior = MetadataV2)]
// struct MetadataV3 {
//     full_name_two: String,
// }

// #[derive(Debug, thiserror::Error)]
// enum MigrateError {}

// impl std::convert::From<std::convert::Infallible> for MigrateError {
//     fn from(_value: std::convert::Infallible) -> Self {
//         unreachable!()
//     }
// }

// We can: Implement the Infallible conversion with TryFrom::Error reference, but only once:
//
// impl std::convert::From<std::convert::Infallible> for <MetadataV2 as TryFrom<MetadataV1>>::Error {
//     fn from(_value: std::convert::Infallible) -> Self {
//         unreachable!()
//     }
// }
//
// If we try to add the V1 as TryFrom<V1> it fails because we didn't write that implementation:
//
// impl std::convert::From<std::convert::Infallible> for <MetadataV1 as TryFrom<MetadataV1>>::Error {
//     fn from(_value: std::convert::Infallible) -> Self {
//         unreachable!()
//     }
// }
//
// If we try to add V3 as TryFrom<V2> then it fails because we've defined the implementation twice.
//
// #[allow(private_interfaces)]
// impl std::convert::From<std::convert::Infallible> for <MetadataV3 as TryFrom<MetadataV2>>::Error {
//     fn from(_value: std::convert::Infallible) -> Self {
//         unreachable!()
//     }
// }
//
// We could theoretically (i'm assuming) store global state about what's been defined and what hasn't.
//
// Once the first error is known, the rest aren't needed, which is how we do it with the derive macro:
//
// impl TryMigrate for $b {
//     type TryFrom = $a;
//     type Error = <<Self as TryMigrate>::TryFrom as TryMigrate>::Error;
//
//     fn deserializer<'de>(input: &str) -> impl serde::de::Deserializer<'de> {
//         <Self as TryMigrate>::TryFrom::deserializer(input)
//     }
// }
//
// The downside is that this forces you to define an error and implement display on it, before you need it
//
// If we're okay creating an error for people we could change the API to be more inline with what we've got today:
//
// #[try_migrate(prior = [MetadataV1, MetadataV2, MetadataV3])]
// struct MetadataV4 {}
//
// Then create an enum MigrationError:
//
// enum MigrationError {
//    MetadataV2(<MetadataV2 as TryFrom<MetadataV1>>::Error)
//    MetadataV3(<MetadataV2 as TryFrom<MetadataV1>>::Error)
// }
//
// And impl Infallible conversion for that one enum. That's cool, let's do it.

// #[allow(private_interfaces)]
// impl std::convert::From<std::convert::Infallible> for <MetadataV2 as TryFrom<MetadataV1>>::Error {
//     fn from(_value: std::convert::Infallible) -> Self {
//         unreachable!()
//     }
// }

// #[allow(private_interfaces)]
// impl std::convert::From<std::convert::Infallible> for <MetadataV3 as TryFrom<MetadataV2>>::Error {
//     fn from(_value: std::convert::Infallible) -> Self {
//         unreachable!()
//     }
// }

// impl TryFrom<MetadataV1> for MetadataV2 {
//     type Error = MigrateError;

//     fn try_from(value: MetadataV1) -> Result<Self, Self::Error> {
//         Ok(MetadataV2 {
//             full_name: value.name,
//         })
//     }
// }

// impl TryFrom<MetadataV2> for MetadataV3 {
//     type Error = MigrateError;

//     fn try_from(value: MetadataV2) -> Result<Self, Self::Error> {
//         Ok(MetadataV3 {
//             full_name_two: value.full_name,
//         })
//     }
// }

// fn main() {
//     let _v2 = MetadataV2::try_from_str_migrations("name = 'richard'").unwrap();
// }

fn main() {}
