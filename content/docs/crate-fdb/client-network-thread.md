+++
title = "Client Network Thread"
description = "Client Network Thread"
date = 2021-05-01T18:10:00+00:00
updated = 2021-05-01T18:10:00+00:00
draft = false
weight = 30
sort_by = "weight"
template = "docs/page.html"

[extra]
toc = true
top = false
+++

_Full working code for concepts described in this section is
[here](https://github.com/fdb-rs/website/tree/main/code/crate-fdb/client-network-thread)_.

FoundationDB clients are **required** to start a [network
thread](https://apple.github.io/foundationdb/api-general.html#client-network-thread)
before they can communicate with the cluster.

Each language binding provides its own mechanism for starting client
network thread. An important consideration is that while the client
network thread can be started and stopped, once it has been stopped,
it cannot be started again.

For our Tokio binding, we recommend that you start the client network
thread and open the database in the main thread before starting the
Tokio runtime. Once the Tokio runtime completes, you can safely close
the database and stop the network thread.

In the next sections we describe how this can be accomplished in more
detail.

## Identifying the cluster file

Begin by identifying the location of the FoundationDB [cluster
file](https://apple.github.io/foundationdb/administration.html#cluster-files). It
is common practice to specify this file using the environment variable
`FDB_CLUSTER_FILE`.

```rust
let fdb_cluster_file = env::var("FDB_CLUSTER_FILE")
    .expect("FDB_CLUSTER_FILE not defined!");
```

## Select API version

Before starting the client network you need to specify [API
version](https://apple.github.io/foundationdb/api-general.html#api-versions)
to use. This can be done as follows.

```rust
unsafe {
    fdb::select_api_version(630);
    // ...
}
```

Here we are selecting API version `630`. In Rust `unsafe` keyword is
used to indicate to the user of API that there invariants that cannot
be checked by the compiler.

For our bindings, we have marked initialization APIs as `unsafe`
primarily to help you understand the interaction that between Tokio
runtime and FoundationDB client network thread.

## Start Client Network Thread

Once the API version is selected you can start the client network
thread as follows.

```rust
unsafe {
    fdb::select_api_version(630);
    fdb::start_network();
}
```

## Open Database

After the client network thread as started, you can open the
FoundationDB database specified by the cluster file.

```rust
let fdb_database = fdb::open_database(fdb_cluster_file)?;
```

[`open_database`](https://docs.rs/fdb/0.3.1/fdb/fn.open_database.html)
function returns a value of
[`FdbResult`](https://docs.rs/fdb/0.3.1/fdb/error/type.FdbResult.html)
type.

`FdbResult` type provides a common error handling abstraction for all
APIs in the crate. In addition it also signals
[errors](https://apple.github.io/foundationdb/api-error-codes.html)
that might have occurred in the C API.

In the above code `fdb_database` is a value of
[`FdbDatabase`](https://docs.rs/fdb/0.3.1/fdb/database/struct.FdbDatabase.html)
type and points to the FoundationDB database specified by the cluster
file. 

## Start Tokio Runtime

After you have obtained a value of `FdbDatabase` type, you can start
the Tokio Runtime, its the main `async` task and move a cloned value
of `FdbDatabase` into the main Tokio task.

```rust
let rt = Runtime::new()?;

let cloned_fdb_database = fdb_database.clone();

rt.block_on(async {
    let fdb_database = cloned_fdb_database;

    // your main async app here
    
    Result::<(), Box<dyn Error>>::Ok(())
})?;
```

At this point, `rt.block_on` will block the main thread. In your
application you will have multiple running threads. They would include
threads launched by Tokio runtime and FoundationDB client network
thread.

## Stop Client Network Thread

Once `rt.block_on` returns, you can gracefully stop the client network
thread by doing the following.

```rust
drop(fdb_database);

unsafe {
    fdb::stop_network();
}
```

This should allow you to cleanly exit your application.
