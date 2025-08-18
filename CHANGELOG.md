## Unreleased

- Changed: The function `fn deserializer` in traits now has a lifetime associated with the argument: `input: &'de str` (https://github.com/schneems/magic_migrate/pull/20)
- Added: A `magic_migrate::de::toml` helper method to return a toml deserializer when `toml` feature is enabled. This is helpful as toml 0.9+ deprecated `toml::Deserializer::new` which returned a deserializer directly with `toml::Deserializer::parse` which returns a `Result<toml::Deserializer, toml::Error>` (https://github.com/schneems/magic_migrate/pull/20)
- Added: A `toml` feature (enabled by default) (https://github.com/schneems/magic_migrate/pull/20)
- Changed: Declarative macros are removed, use the derive macros (proc macros) instead (https://github.com/schneems/magic_migrate/pull/19)
- Fix: Explicit full paths in generated code. (https://github.com/schneems/magic_migrate/pull/18)

## 1.1.0

- Add: Introduce `TryMigrate` derive macro, this is preferred over the declarative macros. (https://github.com/schneems/magic_migrate/pull/17)
- Add: Introduce `magic_migrate::MigrateError` as an available generic error for receiving any `TryFrom::Error`. (https://github.com/schneems/magic_migrate/pull/17)
- Add: Declarative macros (`try_migrate_toml_chain!` etc.) are deprecated in favor of the new derive macro. (https://github.com/schneems/magic_migrate/pull/17)

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
