## Magic Migrate

Automagically load and migrate deserialized structs to the latest version.

> 🎵 If you believe in magic, come along with me
>
> We'll dance until morning 'til there's just you and me 🎵
>

These docs are [intended to be read on docs.rs](https://docs.rs/magic_migrate/latest/magic_migrate/).

## What

Let's say that you made a struct that serializes to disk somehow; perhaps it uses toml. Now, let's say you want to add a new field to that struct but want to keep older persisted data. Whatever should you do?

You can define how to convert from one struct to another using either [`From`] or [`TryFrom`], then tell Rust how to migrate from one to the next via [`Migrate`] or [`TryMigrate`] traits. Now, when you try to load data into the current struct, it will follow a chain of structs in reverse order to find the first one that successfully serializes. When that happens, it will convert that struct to the latest version for you. It's magic! (Actually, it's mostly clever use of trait boundaries, but whatever).

## Docs

For additional docs, see:

- Traits
    - [`Migrate`] trait
    - [`TryMigrate`] trait

- Link macros
    - [`migrate_link`] macro
    - [`try_migrate_link`] macro

- Chain macros
    - [`migrate_toml_chain`] macro

# Example

This example uses the [`migrate_toml_chain`] macro to build a migration chain:

```rust
use magic_migrate::{Migrate, migrate_toml_chain};

use serde::{Deserialize, Serialize, de::Deserializer};
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
struct PersonV1 {
    name: String
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
struct PersonV2 {
    name: String,
    updated_at: DateTime<Utc>
}

// First, define how to map from one struct to another
impl From<PersonV1> for PersonV2 {
    fn from(value: PersonV1) -> Self {
        PersonV2 {
            name: value.name.clone(),
            updated_at: Utc::now()
        }
    }
}

// Then specify the order of the migrations from left to right
migrate_toml_chain!(PersonV1, PersonV2);

// Now, given a serialized struct
let toml_string = toml::to_string(&PersonV1 { name: "Schneems".to_string() }).unwrap();

// Cannot deserialize PersonV1 toml into PersonV2
let result = toml::from_str::<PersonV2>(&toml_string);
assert!(result.is_err());

// Can deserialize to PersonV1 then migrate to PersonV2
let person: PersonV2 = PersonV2::from_str_migrations(&toml_string).unwrap();
assert_eq!(person.name, "Schneems".to_string());
```

## Why

This library was created to handle the case of serialized metadata stored in layers in a <https://github.com/heroku/libcnb.rs> buildpack as toml.

In this use case, structs are serialized to disk when the Cloud Native Buildpack (CNB) is run. Usually, these values represent the application cache state and are important for cache invalidations.

The buildpack implementer has no control over how often the buildpack is run. That means there's no guarantee the end user will run it with sequential struct versions. One user might run with the latest struct version serialized, and another might use a version from years ago.

This scenario happens in the wild with <https://github.com/heroku/heroku-buildpack-ruby> (a "classic" buildpack i.e. not CNB).

Instead of forcing the programmer to consider all possible cache states at all times, a "migration" approach allows programmers to focus on a single cache state change at a time. Which reduces programmer cognitive overhead and (hopefully) reduces bugs.

## What won't it do? (The ABA problem)

This library cannot ensure that if a `PersonV1` struct was serialized, it cannot be loaded into `PersonV2` without migration. I.e. it does not guarantee that the [`From`] or [`TryFrom`] code was run.

For example, if the `PersonV2` struct introduced an `Option<String>` field, instead of `DateTime<Utc>` then the string `"name = 'Richard'"` could be deserialized to either PersonV1 or PersonV2 without needing to call a migration.

- [Playground demonstration of the ABA problem](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e26033d3c8c3c34414fe594674f6d053)

There are more links in a related discussion in Serde:

- [serde-rs/serde issue trying to use `tag` and `deny_unknown_fields`](https://github.com/serde-rs/serde/issues/2666)

## What can you do to harden your code against this (ABA) issue?

- Use [deny_unknown_fields](https://serde.rs/container-attrs.html) from serde. This setting prevents silently dropping additional struct fields. This strategy would handle the case where V1 has two fields and V2 has only one field [playground example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=75c6f06234e1d64aea7b37c448321abf). However, it will **not** protect the case where we've added an optional field, [playground example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=47dde9f52b0c5114ef28f35bb019969c).
- Add tests that ensure one struct cannot deserialize into a later one in the chain. Writing tests might be difficult if your structs have many optional fields and you want to generate permutations of all of them.
- Add a [version marker field](https://stackoverflow.com/a/77700752/147390). This strategy works, but you must notice and keep the field name updated when creating a new struct (possible programmer error). And it will leak an implementation detail to anyone who might see your serialized data (which may or may not matter) to you.
- Read these docs and understand the underlying reason why this happens.
- If you have another suggestion to harden a codebase, open an issue.

## Other possible "migration" solutions and their differences

- Using Serde's [container attributes from and try_from](https://serde.rs/container-attrs.html). This feature only works if you never want to store and deserialize the latest version in the chain. [playground example showing you when this fails](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b6ea1cd054bab5d7df62a04cbd7c6284).

Compared to using Serde's `from` and `try_from` container attribute features, magic migrate will always try to convert to the target struct first, then migrate using the latest possible struct in the chain, allowing structs to migrate through the entire chain or storing and using the latest value.

- The [Serde version crate](https://docs.rs/serde-version/latest/serde_version/) seems to have overlapping goals. Differences are unclear. If you've tried it, update these docs.
