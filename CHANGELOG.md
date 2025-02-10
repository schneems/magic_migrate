## Unreleased

- Add: Introduce `TryMigrate` derive macro, this is preferred over the declarative macros.
- Add: Introduce `magic_migrate::MigrateError` as an available generic error for receiving any `TryFrom::Error`.

## 1.0.1

- Fix: Macro users no longer need to import `std::convert::Infallible` or `serde::de::Deserializer` (https://github.com/schneems/magic_migrate/pull/14)

## 1.0.0 - 2024/12/18

- Change: `TryMigrate::Error` must now be `Display` + `Debug` (https://github.com/schneems/magic_migrate/pull/11)

## 0.2.1 - 2024/12/12

- Fix: Missing semicolons caused compilation errors when using 3 or more values in a chain. This is now fixed (https://github.com/schneems/magic_migrate/pull/7)

## 0.2.0 - 2024/05/12

- Introduce `try_migrate_toml_chain!`, `migrate_deserializer_chain!` and `try_migrate_deserializer_chain!` macros (https://github.com/schneems/magic_migrate/pull/5)

## 0.1.0 - 2024/01/15

- Created
