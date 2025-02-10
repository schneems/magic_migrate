use magic_migrate::TryMigrate;
use serde::Deserialize;

#[derive(TryMigrate, Debug, Deserialize)]
#[try_migrate(from = None, error = std::convert::Infallible)]
#[serde(deny_unknown_fields)]
struct MetadataV1 {
    name: String,
}

#[derive(TryMigrate, Debug, Deserialize)]
#[try_migrate(from = MetadataV1)]
#[serde(deny_unknown_fields)]
struct MetadataV2 {
    full_name: String,
}
impl std::convert::From<MetadataV1> for MetadataV2 {
    fn from(value: MetadataV1) -> Self {
        MetadataV2 {
            full_name: value.name,
        }
    }
}

fn main() {
    let v2: Result<MetadataV2, std::convert::Infallible> =
        MetadataV2::try_from_str_migrations("name = ''").unwrap();
    assert!(matches!(v2, Ok(MetadataV2 { .. })));
}
