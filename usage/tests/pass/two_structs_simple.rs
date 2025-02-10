use magic_migrate::TryMigrate;
use serde::Deserialize;

#[derive(TryMigrate, Deserialize, Debug)]
#[try_migrate(from = None)]
struct MetadataV1 {
    name: String,
}

#[derive(TryMigrate, Deserialize, Debug)]
#[try_migrate(from = MetadataV1)]
struct MetadataV2 {
    full_name: String,
}

impl std::convert::TryFrom<MetadataV1> for MetadataV2 {
    type Error = magic_migrate::MigrateError;

    fn try_from(value: MetadataV1) -> Result<Self, Self::Error> {
        Ok(MetadataV2 {
            full_name: value.name,
        })
    }
}

fn main() {
    let _v2 = MetadataV2::try_from_str_migrations("name = 'richard'").unwrap();
}
