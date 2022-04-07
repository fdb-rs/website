+++
title = "Introducing FoundationDB Client API for Tokio"
description = ""
date = 2022-04-07T11:00:00+00:00
updated = 2022-04-07T11:00:00+00:00
template = "blog/page.html"
draft = false

[taxonomies]
authors = ["rajivr"]

[extra]
lead = ''
images = []
+++

I am happy to announce the release of the
[fdb](https://docs.rs/fdb/0.2.1/fdb/) crate, which provides
[FoundationDB](https://www.foundationdb.org/) client API for
[Tokio](https://tokio.rs/). You can find guide-level documentation for
the fdb crate [here](@/docs/crate-fdb/prerequisites.md).

FoundationDB is a distributed key-value store with [strict
serializable](https://jepsen.io/consistency) ACID
transactions. FoundationDB is known for its reliability and
scalability.

Tokio and FoundationDB are complementary technologies for building
cloud-scale applications. Tokio runtime and its ecosystem of crates
such as Tower, Hyper, Tonic, gives us a great way to develop
applications whose state can be managed using FoundationDB.

In the [fdb-rs](https://github.com/fdb-rs/) GitHub organization,
together with the Tokio and FoundationDB community, I hope to develop
crates related to FoundationDB that work well with the Tokio ecosystem
crates.

Our first crate is the fdb crate. It is also a foundational crate that
interacts with the C library. The fdb crate provides idiomatic Tokio
and Rust APIs that we can use directly in our applications and also to
build other crates. In FoundationDB terminology the latter crates
would be referred to as
[layers](https://apple.github.io/foundationdb/layer-concept.html).

No conversation around FoundationDB would be complete without a word
about correctness and testing.

Under the hood, most of the fdb crate is written in a functional
programming style. FoundationDB has a [tuple
layer](https://apple.github.io/foundationdb/api-python.html#api-python-tuple-layer)
which provides a standardized mechanism of converting key-value byte
strings into a
[s-expression](https://en.wikipedia.org/wiki/S-expression) like data
construct. Parsing code for the tuple layer is written using the
excellent [nom](https://github.com/Geal/nom) parser-combinator
library.

Within the fdb crate when doing asynchronous range reads of key-value
pairs and converting them into a Tokio stream, there is a need to run
a state machine. For correctness and understandability, this [state
machine](https://github.com/fdb-rs/fdb/blob/fdb-0.2.1/fdb/sismic/range_result_state_machine.yaml)
has been modeled using
[sismic](https://sismic.readthedocs.io/en/latest/).

Besides regular unit and integration tests, the fdb crate continuously
runs
[thousands](https://github.com/fdb-rs/fdb/actions/workflows/schedule-6_3_23.yml)
of simulation tests which checks the bindings using a mechanism called
[binding
tester](https://github.com/apple/foundationdb/blob/6.3.23/bindings/bindingtester/spec/bindingApiTester.md). Official
FoundationDB bindings for other languages are tested this way, and we
do the same for our Tokio Rust bindings.

At this point the fdb crate is reasonably stable, and I do not expect
any major API changes to it. Most of the future work will be focused
on adding upcoming FoundationDB 7.x support to the fdb crate and
exploring how we can create a minimal
[record-layer](https://foundationdb.github.io/fdb-record-layer/) like
crate for Tokio.

Tokio and FoundationDB community members have been extremely helpful
in making the fdb crate happen.

From the Tokio community, I would like to thank [Alice
Ryhl](https://ryhl.io/) who answered many Tokio related questions on
Discord.

From the FoundationDB community, I would like to thank [Alec
Grieser](https://forums.foundationdb.org/u/alloc), [Andrew
Noyes](https://forums.foundationdb.org/u/andrew.noyes),
[A.J. Beamon](https://forums.foundationdb.org/u/ajbeamon), [Jingyu
Zhou](https://forums.foundationdb.org/u/jzhou), [Alex
Miller](https://forums.foundationdb.org/u/alexmiller) and [Pierre
Zemb](https://forums.foundationdb.org/u/PierreZ) for providing
detailed replies to my questions on the FoundationDB forum.

The fdb crate would not have been possible without your help!
