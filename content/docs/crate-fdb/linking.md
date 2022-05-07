+++
title = "Linking with C library"
description = "Linking with C library"
date = 2021-05-01T18:10:00+00:00
updated = 2021-05-01T18:10:00+00:00
draft = false
weight = 20
sort_by = "weight"
template = "docs/page.html"

[extra]
toc = true
top = false
+++

_Full working code for concepts described in this section is
[here](https://github.com/fdb-rs/website/tree/main/code/crate-fdb/linking)._

Begin by adding [fdb](https://crates.io/crates/fdb/) crate as a
dependency in your `Cargo.toml` file.

```toml
[dependencies]
fdb = "0.x"
```

fdb crate supports
[linking](https://apple.github.io/foundationdb/api-c.html#linking)
with different versions of the C library using cargo [feature
flags](https://docs.rs/crate/fdb/latest/features). In order to
correctly build your application, you are required to specify the
version of the C library to link against. This can be done in two
ways. Either by using the command line option `--features fdb/fdb-X_Y` or
by specifying the following in your `Cargo.toml`. Here `X` is the
major version and `Y` is the minor version of FoundationDB.

```toml
[features]
default = ["fdb/fdb-X_Y"]
```

If you have installed `libfdb_c.so` in a non-standard location, you
can use the environment variable
[`RUSTC_LINK_SEARCH_FDB_CLIENT_LIB`](https://github.com/fdb-rs/fdb/blob/fdb-0.3.1/fdb-sys/build.rs#L23-L25)
to specify the location of the C library.

**Note** While it is required for your application to link to a
specific version of `fdb_c` library during build time, FoundationDB
client can also dynamically load newer version during runtime. This
feature is used during cluster upgrades. See this
[link](https://apple.github.io/foundationdb/api-general.html#multi-version-client)
for details.
