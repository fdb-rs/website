+++
title = "Prerequisites"
description = "Prerequisites for starting with FoundationDB and Tokio"
date = 2021-05-01T18:10:00+00:00
updated = 2021-05-01T18:10:00+00:00
draft = false
weight = 10
sort_by = "weight"
template = "docs/page.html"

[extra]
toc = true
top = false
+++

Before starting please familiarize yourself with the following.

## Tokio

Tokio project provides an excellent tutorial that you can find
[here](https://tokio.rs/tokio/tutorial). If you are new to Tokio, we
urge to complete the tutorial.

## FoundationDB

FoundationDB comes with extensive documentation. If you are new to
FoundationDB please read through the following sections of
FoundationDB documentation.

* [Why FoundationDB](https://apple.github.io/foundationdb/why-foundationdb.html)

* [Layer Concept](https://apple.github.io/foundationdb/layer-concept.html)

* [Features](https://apple.github.io/foundationdb/features.html)

* [Anti-Features](https://apple.github.io/foundationdb/anti-features.html)

* [Getting Started on Linux](https://apple.github.io/foundationdb/getting-started-linux.html) or [Getting Started on macOS](https://apple.github.io/foundationdb/getting-started-mac.html)

* [Using FoundationDB Clients](https://apple.github.io/foundationdb/api-general.html)

* [Developer Guide](https://apple.github.io/foundationdb/developer-guide.html)

  You can ignore the
  [directories](https://apple.github.io/foundationdb/developer-guide.html#directories)
  section. We do not support directory layer in our API. Instead we
  support FoundationDB 7.1
  [Tenants](https://apple.github.io/foundationdb/tenants.html) that
  natively provides similar feature.
  
  When you read the section on [transaction retry
  loop](https://apple.github.io/foundationdb/developer-guide.html#transaction-retry-loops)
  take note of the presence of two distinct types - A _database_ type
  and a _transaction_ type. Other language bindings allows values of
  these two types to be used interchangeably in their API. In our
  bindings, we keep distinction between a value of database type and
  value of transaction type separate. You will see this come up in the
  class scheduling tutorial.

* [Python Class Scheduling Tutorial](https://apple.github.io/foundationdb/class-scheduling.html)

  Even though you might not use Python in production, this tutorial
  will help you become familiar with the API. You can find our version
  of class scheduling tutorial [here](../class-scheduling-tutorial/).

