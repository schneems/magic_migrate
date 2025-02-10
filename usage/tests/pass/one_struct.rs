use magic_migrate::TryMigrate;

#[derive(TryMigrate, serde::Deserialize, Debug)]
#[try_migrate(from = None)]
struct MetadataV1 {}

fn main() {
    let _v1 = MetadataV1::try_from_str_migrations("").unwrap();
}
