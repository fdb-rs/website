+++
title = "fdb crate v0.3.1 released"
description = ""
date = 2022-04-09T11:00:00+00:00
updated = 2022-04-09T11:00:00+00:00
template = "blog/page.html"
draft = false

[taxonomies]
authors = ["rajivr"]

[extra]
lead = ''
images = []
+++

I am happy to announce fdb crate version
[v0.3.1](https://docs.rs/fdb/0.3.1/fdb/index.html) has been
released. This is the first release in the v0.3.x series and provides
support for FoundationDB client API version 710 using cargo feature
flag `fdb-7_1`.

Following are the API changes and new features in this release.

1. The
   [`Range`](https://docs.rs/fdb/0.3.1/fdb/range/struct.Range.html)
   type now has a
   [`into_parts`](https://docs.rs/fdb/0.3.1/fdb/range/struct.Range.html#method.into_parts)
   method that lets you easily de-structure a `Range` value. When you
   have an owned `Range` value, and you would like owned values of its
   parts, then you can use this method and avoid unnecessary
   [`clone()`](https://doc.rust-lang.org/std/clone/trait.Clone.html#tymethod.clone). The
   `into_parts` idiom is also available on
   [`KeyValue`](https://docs.rs/fdb/0.3.1/fdb/struct.KeyValue.html#method.into_parts)
   and
   [`MappedKeyValue`](https://docs.rs/fdb/0.3.1/fdb/struct.MappedKeyValue.html#method.into_parts)
   types.

2. The
   [`ReadTransaction`](https://docs.rs/fdb/0.3.1/fdb/transaction/trait.ReadTransaction.html)
   trait has support for
   [`get_range_split_points`](https://docs.rs/fdb/0.3.1/fdb/transaction/trait.ReadTransaction.html#tymethod.get_range_split_points)
   which is an API version 710 feature.
   
3. The
   [`ReadTransaction`](https://docs.rs/fdb/0.3.1/fdb/transaction/trait.ReadTransaction.html)
   trait also has support for the
   [`get_mapped_range`](https://docs.rs/fdb/0.3.1/fdb/transaction/trait.ReadTransaction.html#tymethod.get_mapped_range)
   feature, which is an experimental feature in FoundationDB 7.1. This
   feature is automatically enabled when you use the cargo feature
   flag `fdb-7_1`.
   
   The `Range` type now includes a
   [`into_mapped_stream`](https://docs.rs/fdb/0.3.1/fdb/range/struct.Range.html#method.into_mapped_stream)
   method that returns a stream (asynchronous iterator) of
   `MappedKeyValue`.
   
   You can find more information about the GetMappedRange feature
   [here](https://github.com/apple/foundationdb/wiki/Everything-about-GetMappedRange). [`get_mapped_range.rs`](https://github.com/fdb-rs/fdb/blob/fdb-0.3.1/fdb/examples/get_mapped_range.rs)
   provides an example of how this feature can be used.
   
4. FoundationDB
   [Tenant](https://apple.github.io/foundationdb/tenants.html) support
   is another experimental feature whose support is included in this
   release. Tenant support is also automatically enabled when you use
   the cargo feature flag `fdb-7_1`.

   In the bindings Tenant support is implemented using types
   [`Tenant`](https://docs.rs/fdb/0.3.1/fdb/struct.Tenant.html),
   [`FdbTenant`](https://docs.rs/fdb/0.3.1/fdb/tenant/struct.FdbTenant.html)
   and
   [`TenantManagement`](https://docs.rs/fdb/0.3.1/fdb/tenant/struct.TenantManagement.html). The
   [`open_tenant`](https://docs.rs/fdb/0.3.1/fdb/database/struct.FdbDatabase.html#method.open_tenant)
   method on `FdbDatabase` provides a way to create a value of
   `FdbTenant` type and work with tenants in the cluster.

With the release of v0.3.x series, support for cargo feature flag
`fdb-6_3` is deprecated. We will continue to support the cargo feature
flag `fdb-6_3` in subsequent v0.3.x releases. It would be removed in
v0.4.x release, when support for API version 720 and cargo feature
flag `fdb-7_2` would be introduced. Please plan your migrations
accordingly.

I would like to say a big thank you to the FoundationDB community for
their support and answering my questions in the forums. This release
would not have been possible without your help!

Please [contact us](/docs/help/contact-us/) if you have any
questions or feedback.
