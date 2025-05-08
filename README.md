# libsqlite3-hotbundle

A fork of `libsqlite3-sys` that bundles a more recent version of sqlite3.

## Usage

In your `Cargo.toml`, include `libsqlite3-hotbundle` as a dependency alongside
an application that otherwise uses `libsqlite3-sys` in *non-bundled* mode. The
"hotbundle" will include a bundled version of the sqlite3 library that should
then be chosen instead of the system's sqlite3 library or the version of the
sqlite3 library that `libsqlite3-sys` would have bundled.

```toml
libsqlite3-hotbundle = "1.490200"
rusqlite = "0.35"
```

## Versioning

This crate uses the sqlite release bundle versioning, which is a big number that
is the whole sqlite3 library version strung together. A sqlite3 release numbered
"3.45.6" would be numbered 3450600, which would become 1.450600.0 in this crate.
