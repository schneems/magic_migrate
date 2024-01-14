## Magic Migrate

Automagically load and migrate deserialized structs to the latest version

## What?

Let's say that you made a struct that serializes to disk somehow,
perhaps it uses toml. Now, let's say that you want to add a new field to that
struct, but you don't want to lose older persisted data. What ever should you do?

You can define how to convert from one struct to another using either `From` or
`TryFrom` then tell Rust how to migrate from one to the next via `Migrate` or `TryMigrate`
traits. Now, when you try to load data into the current struct it will follow a chain
of structs in reverse order to find the first one that successfully serializes. When
that happens, it will convert that struct to the latest version for you. It's magic!
(It's actually mostly clever use of trait boundries, but whatever).

This library was created to handle the case of serialized metadata stored in
layers in a https://github.com/heroku/libcnb.rs buildpack. To that end, it
includes a helpful macro to define a chain of migrations for you.

Read the [docs]() for more info and plenty of examples.

> 🎵 If you believe in magic, come along with me
>
> We'll dance until morning 'til there's just you and me 🎵
>
