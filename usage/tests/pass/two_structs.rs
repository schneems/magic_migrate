use magic_migrate::TryMigrate;
use std::convert::TryFrom;

#[derive(TryMigrate, serde::Deserialize, Debug)]
#[try_migrate(prior = None)]
struct MetadataV1 {
    name: String,
}

#[derive(TryMigrate, serde::Deserialize, Debug)]
#[try_migrate(prior = MetadataV1)]
struct MetadataV2 {
    full_name: String,
}

#[derive(Debug, thiserror::Error)]
enum MigrateError {}

impl TryFrom<MetadataV1> for MetadataV2 {
    type Error = MigrateError;

    fn try_from(value: MetadataV1) -> Result<Self, Self::Error> {
        Ok(MetadataV2 {
            full_name: value.name,
        })
    }
}

fn main() {
    let _v2 = MetadataV2::try_from_str_migrations("name = 'richard'").unwrap();
}
