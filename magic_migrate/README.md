<!--
    This readme is created with https://crates.io/crates/cargo-rdme

    To update: edit `magic_migrate/src/lib.rs` then run:

    ```
    $ cargo rdme -w magic_migrate
    ```

    Note: All intra-doc links need a certain type of formattting for rdme to expand
    them to the correct docs.rs links. More info found here:

        https://github.com/orium/cargo-rdme/blob/964a939c8c86a2e6aa3f6a8f89cf75b64ab92f6a/README.md#intralinks
-->

# magic_migrate

<!-- cargo-rdme start -->

<!-- cargo-rdme -->

<!-- cargo-rdme end -->

## Releasing

Releases can be performed via `cargo release`:

```
$ cargo install cargo-release
```

Release readiness for all crates can be checked by running:

```
$ cargo release --workspace --exclude usage --dry-run
```

When satisfied, contributors with permissions can release by running:

```
$ cargo release --workspace --exclude usage --execute
```
