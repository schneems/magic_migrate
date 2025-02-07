use magic_migrate::TryMigrate;

#[derive(TryMigrate)]
#[try_migrate(prior = None)]
#[try_migrate(prior = None)]
struct MetadataV1 {}

#[derive(TryMigrate)]
#[try_migrate(error = None)]
#[try_migrate(error = None)]
struct MetadataV2 {}

fn main() {}
