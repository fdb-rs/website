+++
title = "Class Scheduling Tutorial"
description = "Class Scheduling Tutorial"
date = 2021-05-01T18:10:00+00:00
updated = 2021-05-01T18:10:00+00:00
draft = false
weight = 40
sort_by = "weight"
template = "docs/page.html"

[extra]
toc = true
top = false
+++

_Full working code for concepts described in this section is
[here](https://github.com/fdb-rs/website/tree/main/code/crate-fdb/class-scheduling-tutorial)_.

This tutorial provides a walk through of designing and building a
simple application in Tokio and Rust using FoundationDB. In this
tutorial, we use a few simple data modeling techniques. For a more
in-depth discussion on data modeling in FoundationDB, see [Data
Modeling](https://apple.github.io/foundationdb/data-modeling.html).

The concepts in this tutorial are applicable to all the
[languages](https://apple.github.io/foundationdb/api-reference.html)
supported by FoundationDB. If you prefer, you can see a version of
this tutorial in
[Java](https://apple.github.io/foundationdb/class-scheduling-java.html),
[Go](https://apple.github.io/foundationdb/class-scheduling-go.html),
[Python](https://apple.github.io/foundationdb/class-scheduling.html)
or
[Ruby](https://apple.github.io/foundationdb/class-scheduling-ruby.html).

## First steps

Let's begin with "Hello World."

If you have not yet installed FoundationDB, see [Getting Started on
macOS](https://apple.github.io/foundationdb/getting-started-mac.html)
or [Getting Started on
Linux](https://apple.github.io/foundationdb/getting-started-linux.html).

We will start by importing the paths that we need from Rust standard
library, FDB and Tokio crates.

```rust
use fdb::transaction::{ReadTransaction, Transaction};
use fdb::tuple::Tuple;

use tokio::runtime::Runtime;

use std::env;
use std::error::Error;
```

Before using the API, we need to specify the API version. This allows
programs to maintain compatibility even if the API is modified in
future versions. Next we open a FoundationDB database. The API will
connect to the FoundationDB cluster using the cluster file specified
by `FDB_CLUSTER_FILE`. If you specify an empty path (`""`) to
[`fdb::open_database`](https://docs.rs/fdb/0.2.2/fdb/fn.open_database.html)
then the client will connect to the cluster as indicated by the
[default cluster
file](https://apple.github.io/foundationdb/administration.html#default-cluster-file).

```rust
unsafe {
    fdb::select_api_version(630);
    fdb::start_network();
}

let fdb_cluster_file = env::var("FDB_CLUSTER_FILE").expect(
    "FDB_CLUSTER_FILE not defined!",
);

let fdb_database = fdb::open_database(fdb_cluster_file)?;
```

We are ready to use the database. First, let us write a key-value
pair. We do this by executing a transaction with
[`run()`](https://docs.rs/fdb/0.2.2/fdb/database/struct.FdbDatabase.html#method.run)
method. We will also use methods on type
[`Tuple`](https://docs.rs/fdb/0.2.2/fdb/tuple/struct.Tuple.html) to
[`pack()`](https://docs.rs/fdb/0.2.2/fdb/tuple/struct.Tuple.html#method.pack)
data for storage in the database.

```rust
fdb_database
    .run(|tr| async move {
        tr.set(
            {
                let key_tup: (&'static str,) = ("hello",);
                let mut tup = Tuple::new();
                tup.add_string((key_tup.0).to_string());
                tup
            }
            .pack(),
            {
                let val_tup: (&'static str,) = ("world",);
                let mut tup = Tuple::new();
                tup.add_string((val_tup.0).to_string());
                tup
            }
            .pack(),
        );
        Ok(())
    })
    .await?;
```

When `run()` returns without an error, the modification is durably
stored in FoundationDB! This method creates a transaction with a
single modification. We will see later how to do multiple operations
in a single transaction. For now let us read back the data. We will
use `Tuple` again to unpack the `result` as a `String`.

```rust
let hello = fdb_database
    .run(|tr| async move {
        let result = tr
            .get(
                {
                    let key_tup: (&'static str,) = ("hello",);
                    let mut tup = Tuple::new();
                    tup.add_string((key_tup.0).to_string());
                    tup
                }
                .pack(),
            )
            .await?
            .unwrap();

        Ok(Tuple::from_bytes(result)?.get_string_ref(0)?.to_string())
    })
    .await?;

println!("Hello {}", hello);
```

If this is all working, it looks like we are ready to start building
real application. For reference, the full code for "hello world" is
[here](https://github.com/fdb-rs/website/tree/main/code/crate-fdb/class-scheduling-tutorial/hello-world).

## Class scheduling application

Let us say we have been asked to build a class scheduling system for
students and administrators. We will walk through the design and
implementation of this application. Instead of typing everything in as
you follow along, look at
[`class-scheduling/src/main.rs`](https://github.com/fdb-rs/website/tree/main/code/crate-fdb/class-scheduling-tutorial/class-scheduling/src/main.rs)
for a finished version of the program. You may want to refer to this
code as we walk through the tutorial.

### Requirements

We will need to let users list available classes and track which
students have signed up for which classes. Here is a first cut of the
functions we will need to implement.

```rust
available_classes()          // returns a vector of classes
signup(student, class_name)  // signs up a student for a class
dropout(student, class_name) // drops a student from a class
```

### Data model

First, we need to design a [data
model](https://apple.github.io/foundationdb/data-modeling.html). A
data model is just a method for storing our application data using
keys and values in FoundationDB. We seem to have two main types of
data.

1. A list of classes (and)

2. A record of which student will attend which classes

Let us keep attending data like this:

```
// ("attends", student, class_name) = ""
```

We will just store the key with a blank value to indicate that a
student is signed up for a particular class. For this application, we
are going to think about a key-value pair's key as a
[tuple](https://apple.github.io/foundationdb/data-modeling.html#data-modeling-tuples). Encoding
a tuple of data elements into a key is a very common pattern for an
ordered key-value store.

We will keep data about classes like this:

```
// ("class", class_name") = seats_available
```

Similarly, each such key will represent an available class. We will
use `seats_available` to record the number of seats available.

### Leverage Rust type system

In FoundationDB keys and values are simple byte strings. The database
does not interpret the contents of the keys and values. The tuple
layer provides some type safety, but that is still about
_representation_ and not about the _semantics_ of data. For example in
our data model, `student` and `class_name` are tuple layer strings,
but semantically one refers to a student while the other refers to a
class name.

If we are not careful when writing our application, we could make the
mistake of using `student`, when we intended to use `class_name`. As
our data model becomes complex, it is easy to make mistakes.

Luckily for us, we can leverage the Rust type system to avoid many
such errors.

By using the
[newtype](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
idiom, we can get compile time guarantee that we would not be allowed
to accidentally interchange a `class_name` value with a `student`
value.

```rust
#[derive(Clone, Debug, PartialEq)]
struct Class(String);

#[derive(Clone, Debug, PartialEq)]
struct Student(String);
```

When constructing and deconstructing FoundationDB tuples, we could
potentially re-order the tuples. For example instead of

```
// ("attends", student, class_name) = ""
```

we might mistakenly construct the following FoundationDB tuple, which
would break our data model.

```
// (student, class_name, "attends") = ""
```

We can avoid this problems by creating custom types for keys and
values. For an example see `AttendsKey` type in
[`class-scheduling/src/main.rs`](https://github.com/fdb-rs/website/tree/main/code/crate-fdb/class-scheduling-tutorial/class-scheduling/src/main.rs). Since
Rust [tuple](https://doc.rust-lang.org/std/primitive.tuple.html) and
FoundationDB tuple are isomorphic, within `AttendsKey` type, we can
first construct a Rust tuple and then convert it to a FoundationDB
tuple.

```rust
let key_tup: (&'static str, Student, Class) = \
    ("attends", a.student, a.class_name);

let key_bytes = {
    let mut tup = Tuple::new();

    tup.add_string((key_tup.0).to_string());

    let Student(student_inner) = key_tup.1;
    tup.add_string(student_inner);

    let Class(class_inner) = key_tup.2;
    tup.add_string(class_inner);

    tup
}
.pack();
```

Here we are asserting the type of `key_tup` to our data model and then
immediately creating the FoundationDB tuple in `key_bytes`. By keeping
`key_tup` and `key_bytes` visually next to each other, the scope for
re-ordering errors is minimized.

Lastly, when we look at
[`Transaction`](https://docs.rs/fdb/0.2.2/fdb/transaction/trait.Transaction.html)
and
[`ReadTransaction`](https://docs.rs/fdb/0.2.2/fdb/transaction/trait.ReadTransaction.html)
traits, we will find that many of the methods accepts values of type
`impl Into<Key>` and `impl Into<Value>`. We can take advantage of this
design by implementing `From` trait on our types. Once we have the
appropriate `From` traits implemented, these APIs will work with
values of our type and there is no need to work with raw bytes.

[`class-scheduling/src/main.rs`](https://github.com/fdb-rs/website/tree/main/code/crate-fdb/class-scheduling-tutorial/class-scheduling/src/main.rs)
has additional examples of above mentioned techniques.

### Transactions

We are going to rely on the powerful guarantees of transactions to
help keep of all our modifications straight, so let us look at how the
FoundationDB Tokio API lets you write a transaction function. We use
`run()` method to execute a code block transactionally. Let us write
the simple `add_class` function we will use to populate the database's
class list.

```rust
fn add_class(tr: &FdbTransaction, class_name: Class) {
    // ("class", class_name)
    let class_key = ClassKey::new(class_name);

    let class_value = ClassValue::new(100);

    tr.set(class_key, class_value);
}

db.run(|tr| async move {
    // Assuming we have initialized `class_name` with a value of type
    // `Class`
    add_class(&tr, class_name);

    Ok(())
})
.await
```

The `run()` method _automatically creates a transaction and implements
a retry loop_ to ensure that the transaction eventually commits.

This is equivalent to something like:

```rust,hl_lines=5-11 16-20
let t = self.create_transaction()?;

loop {
    let ret_val = (async move {
        // [...]

        // Assuming we have initialized `class_name` with a value of
        // type `Class`
        add_class(&t, class_name);

        Ok(())
    }).await;

    // Received an error
    if let Err(e) = ret_val {
        if FdbError::layer_error(e.code()) {
            // Check if it is a layer error. If so, just
            // return it.
            return Err(e);
        } else if let Err(e1) = unsafe { t.on_error(e) }.await {
            // Check if `on_error` returned an error. This
            // means we have a non-retryable error.
            return Err(e1);
        } else {
            continue;
        }
    }

    // No error. Attempt to commit the transaction.
    if let Err(e) = unsafe { t.commit() }.await {
        // Commit returned an error
        if let Err(e1) = unsafe { t.on_error(e) }.await {
            // Check if `on_error` returned an error. This
            // means we have a non-retryable error.
            return Err(e1);
        } else {
            continue;
        }
    }

    // Commit successful, return `Ok(())`
    return ret_val;
}

```

You can abort a transaction by creating value `Err(FdbError::new(err)`
where `err` is in the range `100.=999`. This provides an unified
approach to error handling in FoundationDB Tokio APIs. See
[`error`](https://docs.rs/fdb/0.2.2/fdb/error/index.html) module and
[`FdbError`](https://docs.rs/fdb/0.2.2/fdb/error/struct.FdbError.html)
type documentation for details.

Note that by default, the operation will be retried an infinite number
of times and the transaction will never time out. It is therefore
recommended that the client choose a default transaction retry limit
or timeout value that is suitable for their application. This can be
set either at the transaction level by passing
[`TransactionOption::RetryLimit`](https://docs.rs/fdb/0.2.2/fdb/transaction/enum.TransactionOption.html#variant.RetryLimit)
and
[`Transaction::Timeout`](https://docs.rs/fdb/0.2.2/fdb/transaction/enum.TransactionOption.html#variant.Timeout)
to
[`ReadTransaction::set_option`](https://docs.rs/fdb/0.2.2/fdb/transaction/trait.ReadTransaction.html#tymethod.set_option)
method or at the database level by passing
[`DatabaseOption::TransactionRetryLimit`](https://docs.rs/fdb/0.2.2/fdb/database/enum.DatabaseOption.html#variant.TransactionRetryLimit)
and
[`DatabaseOption::TransactionRetryLimit`](https://docs.rs/fdb/0.2.2/fdb/database/enum.DatabaseOption.html#variant.TransactionRetryLimit)
to
[`FdbDatabase::set_option`](https://docs.rs/fdb/0.2.2/fdb/database/struct.FdbDatabase.html#method.set_option)
method. For example, one can set a one minute timeout on each transaction and a default retry limit of 100 by calling:

```rust
// 60,000 ms = 1 minute
fdb_database.set_option(DatabaseOption::TransactionTimeout(60000))?;
fdb_database.set_option(DatabaseOption::TransactionRetryLimit(100))?;
```

### Making some sample classes

Let us make some sample classes and create a function
`init_class_names` that returns a vector of classes. We will make
individual classes from combinations of class types, levels and times:

```rust
const LEVELS: [&str; 9] = [
    "intro",
    "for dummies",
    "remedial",
    "101",
    "201",
    "301",
    "mastery",
    "lab",
    "seminar",
];

const TYPES: [&str; 10] = [
    "chem", "bio", "cs", "geometry", "calc", "alg", "film", "music",
    "art", "dance",
];

const TIMES: [&str; 18] = [
    "2:00", "3:00", "4:00", "5:00", "6:00", "7:00", "8:00", "9:00",
    "10:00", "11:00", "12:00", "13:00", "14:00", "15:00", "16:00",
    "17:00", "18:00", "19:00",
];

fn init_class_names() -> Vec<Class> {
    let mut class_names = Vec::new();

    for level in LEVELS {
        // we can't use type here as that is a keyword in Rust.
        for typ in TYPES {
            for time in TIMES {
                class_names.push(
                    Class(
                        format!("{} {} {}", time, typ, level).to_string()
                    )
                );
            }
        }
    }

    class_names
}
```

### Initializing the database

We initialize the database with our class list:

```rust
async fn init(db: &FdbDatabase) -> FdbResult<()> {
    db.run(|tr| async move {
        // ("attends")
        let attends_prefix_range = AttendsPrefix::new().get_range();
        tr.clear_range(attends_prefix_range);

        // ("class")
        let class_prefix_range = ClassPrefix::new().get_range();
        tr.clear_range(class_prefix_range);

        for class_name in init_class_names() {
            add_class(&tr, class_name);
        }

        Ok(())
    })
    .await
}
```

After `init()` is run, the database will contain all the sample
classes we created above.

### Listing available classes

Before students can do anything else, they need to be able to retrieve
a list of available classes from the database. We do this by
implementing `available_classes` function. Because FoundationDB sorts
its data by key and therefore has efficient range-read capability, we
can retrieve all of the classes in a single database call. We find
this range of keys with
[`get_range()`](https://docs.rs/fdb/0.2.2/fdb/transaction/trait.ReadTransaction.html#tymethod.get_range)
method.

```rust
const KEY_CONVERTION_ERROR: i32 = 999;

impl TryFrom<Key> for ClassKey {
    type Error = FdbError;

    fn try_from(key: Key) -> FdbResult<ClassKey> {
        Tuple::from_bytes(key)
            .and_then(|tup| {
                // ("class", class_name)
                if tup.get_string_ref(0)?.as_str() != "class" {
                    return Err(FdbError::new(KEY_CONVERTION_ERROR));
                }

                let class_name = Class(tup.get_string_ref(1)?.to_string());

                Ok(ClassKey::new(class_name))
            })
            .map_err(|_| FdbError::new(KEY_CONVERTION_ERROR))
    }
}

impl ClassPrefix {
    fn get_range(&self) -> Range {
        // ("class")
        let class_tup: (&'static str,) = ("class",);

        let class_range = {
            let mut tup = Tuple::new();
            tup.add_string((class_tup.0).to_string());
            tup
        }
        .range(Bytes::new());

        class_range
    }

    fn get_range_selector(&self) -> (KeySelector, KeySelector) {
        let key_range = self.get_range();

        let begin_key_selector = KeySelector::first_greater_or_equal(
            key_range.begin().clone()
        );
        let end_key_selector = KeySelector::first_greater_or_equal(
            key_range.end().clone()
        );

        (begin_key_selector, end_key_selector)
    }
}

async fn available_classes(tr: &FdbTransaction) -> FdbResult<Vec<Class>> {
    // ("class", ...)
    let (begin_key_selector, end_key_selector) =
        ClassPrefix::new().get_range_selector();

    let mut range_stream = tr.get_range(
        begin_key_selector,
        end_key_selector,
        RangeOptions::default(),
    );

    let mut class_names = Vec::new();

    while let Some(x) = range_stream.next().await {
        let kv = x?;
        let class_key = TryInto::<ClassKey>::try_into(
            kv.get_key().clone()
        )?;
        class_names.push(class_key.into());
    }

    Ok(class_names)
}
```

In general, the
[`Tuple::range()`](https://docs.rs/fdb/0.2.2/fdb/tuple/struct.Tuple.html#method.range)
method returns a
[`Range`](https://docs.rs/fdb/0.2.2/fdb/range/struct.Range.html)
representing all the key-value pairs starting with the specified
tuple. In this case we want all classes, so we call `Tuple::range()`
with `("class",)`. Once we have the value of `Range` type, we get the
[`KeySelector`](https://docs.rs/fdb/0.2.2/fdb/struct.KeySelector.html)
associated with the `Range`. The `KeySelector` can then be used to
call `get_range` method which returns a Tokio
[Stream](https://docs.rs/tokio-stream/0.1.8/tokio_stream/trait.StreamExt.html)
of the key-values specified by `KeySelector`. To extract the class
name, we unpack the key using
[`Tuple::fromBytes()`](https://docs.rs/fdb/0.2.2/fdb/tuple/struct.Tuple.html#method.from_bytes)
and take its second part. (The first part is the prefix `"class"`).

### Signing up for a class

We finally get to the crucial function. A student has decided on a
class (by name) and wants to sign up. The `signup` function will take
a `student` and a `class_name`.

```rust
fn signup(tr: &FdbTransaction, student: Student, class_name: Class) {
    // ("attends", student, class_name)
    let attends_key = AttendsKey::new(student, class_name);

    // ""
    let attends_value = AttendsValue::new();

    tr.set(attends_key, attends_value);
}
```

We simply insert the appropriate tuple key (with a blank value).

### Dropping a class

Dropping a class is similar to signing up:

```rust
// Unlike other bindings, we cannot name this function as `drop`,
// because `drop` is already used in Rust.
fn dropout(tr: &FdbTransaction, student: Student, class_name: Class) {
    // ("attends", student, class_name)
    let attends_key = AttendsKey::new(student, class_name);

    tr.clear(attends_key);
}
```

Of course, to actually drop the student from the class, we need to be
able to delete a record from the database. We do this with the
[`clear()`](https://docs.rs/fdb/0.2.2/fdb/transaction/trait.Transaction.html#tymethod.clear)
method.

### Done?

We report back to the project leader that our application is done ---
students can sign up for, drop, and list classes. Unfortunately, we
learn that a new problem has been discovered: popular classes are
being over-subscribed. Our application now needs to enforce the class
size constraint as students add and drop classes.

### Seats are limited!

Let us go back to the data model. Remember that we stored the number
of seats in the class in the value of the key value entry in the class
list.

```
// ("class", class_name") = seats_available
```

Let us refine that a bit to track the _remaining_ number of seats in
the class. The initialization can work the same way (in our example,
all classes initially have 100 seats), but the `available_classes`,
`signup`, and `dropout` functions are going to have to change. Let us
start with `available_casses`.

```rust,hl_lines=21-23 25
async fn available_classes(tr: &FdbTransaction) -> FdbResult<Vec<Class>> {
    // ("class", ...)
    let (begin_key_selector, end_key_selector) =
        ClassPrefix::new().get_range_selector();

    let mut range_stream = tr.get_range(
        begin_key_selector,
        end_key_selector,
        RangeOptions::default(),
    );

    let mut class_names = Vec::new();

    while let Some(x) = range_stream.next().await {
        let kv = x?;

        let class_key = TryInto::<ClassKey>::try_into(
            kv.get_key().clone()
        )?;

        let seats_available = ClassValue::from(
            kv.get_value().clone()
        ).get_val();

        if seats_available > 0 {
            class_names.push(class_key.into());
        }
    }

    Ok(class_names)
}
```

This is easy --- we simply add a condition to check that the value is
non-zero. Let us check out `signup` next.

```rust,hl_lines=15-38
const NO_REMAINING_SEATS: i32 = 996;
const ALREADY_SIGNED_UP: i32 = 997;

async fn signup(
    tr: &FdbTransaction,
    student: Student,
    class_name: Class
) -> FdbResult<()> {
    // ("attends", student, class_name)
    let attends_key = AttendsKey::new(student, class_name.clone());

    // ""
    let attends_value = AttendsValue::new();

    if tr.get(attends_key.clone()).await?.is_some() {
        Err(FdbError::new(ALREADY_SIGNED_UP))
    } else {
        // ("class", class_name)
        let class_key = ClassKey::new(class_name);

        // Safety: It is safe to `unwrap()` here because in our data
        // model assume that key `("class", class_name)` will *always*
        // have seats left value.
        let class_value = ClassValue::from(
            tr.get(class_key.clone()).await?.unwrap()
        );

        let seats_left = class_value.get_val();

        if seats_left == 0 {
            Err(FdbError::new(NO_REMAINING_SEATS))
        } else {
            let updated_class_value = ClassValue::new(seats_left - 1);
            tr.set(class_key, updated_class_value);
            tr.set(attends_key, attends_value);
            Ok(())
        }
    }
}
```

We now have to check that we are not already signed up, since we do
not want to double sign up to decrease the number of seats twice. Then
we look up how many seats are left to make sure there is a seat
remaining so we do not push the counter into the negative. If there
is a seat remaining, we decrement the counter.

### Concurrency and consistency

The `signup` function is starting to get a bit complex; it now reads
and writes a few different key-value pairs in the database. One of the
tricky issues in this situation is what happens as multiple
clients/students read and modify the database at the same time. Could
two students see one remaining seat and sign up at the same time?

These are tricky issues without simple answers --- unless you have
transactions! Because these functions are defined using FoundationDB
transactions, we can have a simple answer. Each transaction behaves as
if it is the only one modifying the database. There is no way for a
transaction to "see" another transaction change the database, and each
transaction ensures that either all of its modifications occur or none
of them do.

Looking deeper, it is, of course, possible for two transactions to
conflict. For example, if two people both see a class with one seat
and sign up at the same time, FoundationDB must allow only one to
succeed. This causes one of the transactions to fail to commit (which
can also be caused by network outages, crashes, etc.). To ensure
correct operation, applications need to handle this situation, usually
via retrying the transaction. In this case, the conflicting
transaction will be retried automatically by the `run()` method and
will eventually lead to the correct result, a `NO_REMAINING_SEATS`
error.

### Idempotence

Occasionally, a transaction might be retried even after it succeeds
(for example, if the client loses contact with the cluster at just the
wrong moment). This can cause problems if transactions are not written
to be idempotent, i.e. to have the same effect if committed twice as if
committed once. There are generic design patterns for [making any
transaction
idempotent](https://apple.github.io/foundationdb/developer-guide.html#developer-guide-unknown-results),
but many transactions are naturally idempotent. For example, all of the
transactions in this tutorial are idempotent.

### Dropping with limited seats

Let us finish up the limited seats feature by modifying the `dropout`
function.

```rust,hl_lines=9-24 27
async fn dropout(
    tr: &FdbTransaction,
    student: Student,
    class_name: Class
) -> FdbResult<()> {
    // ("attends", student, class_name)
    let attends_key = AttendsKey::new(student, class_name.clone());

    if tr.get(attends_key.clone()).await?.is_none() {
        // not taking class
        Ok(())
    } else {
        // ("class", class_name)
        let class_key = ClassKey::new(class_name);

        // Safety: It is safe to `unwrap()` here because in our data
        // model assume that key `("class", class_name)` will *always*
        // have seats left value.
        let class_value = ClassValue::from(
            tr.get(class_key.clone()).await?.unwrap()
        );
        let seats_left = class_value.get_val();
        let updated_class_value = ClassValue::new(seats_left + 1);
        tr.set(class_key, updated_class_value);
        tr.clear(attends_key);
	
        Ok(())
    }
}
```

This case is easier than signup because there are no constraints we
can hit. We just need to make sure the student is in the class and to
"give back" one seat when the student drops.

### More features?!

Of course, as soon as our new version of the system goes live, we hear
of a trick that certain students are using. They are signing up for
all classes immediately, and only later dropping those that they do
not want to take. This as lead to an unusable system, and we have been
asked to fix it. We decide to limit students to five classes:

```rust,hl_lines=32-36
const TOO_MANY_CLASSES: i32 = 995;

async fn signup(
    tr: &FdbTransaction,
    student: Student,
    class_name: Class
) -> FdbResult<()> {
    // ("attends", student, class_name)
    let attends_key = AttendsKey::new(student.clone(), class_name.clone());

    // ""
    let attends_value = AttendsValue::new();

    if tr.get(attends_key.clone()).await?.is_some() {
        Err(FdbError::new(ALREADY_SIGNED_UP))
    } else {
        // ("class", class_name)
        let class_key = ClassKey::new(class_name);

        // Safety: It is safe to `unwrap()` here because in our data
        // model assume that key `("class", class_name)` will *always*
        // have seats left value.
        let class_value = ClassValue::from(
            tr.get(class_key.clone()).await?.unwrap()
        );

        let seats_left = class_value.get_val();

        if seats_left == 0 {
            Err(FdbError::new(NO_REMAINING_SEATS))
        } else {
            let attends_student_kvs =
                get_attends_student_keyvalue(tr, student).await?;

            if attends_student_kvs.len() == 5 {
                Err(FdbError::new(TOO_MANY_CLASSES))
            } else {
                let updated_class_value = ClassValue::new(seats_left - 1);
                tr.set(class_key, updated_class_value);
                tr.set(attends_key, attends_value);
                Ok(())
            }
        }
    }
}
```

Fortunately, we decided on a data model that keeps all of the
attending records for a single student together. With this approach,
we can use a single range read to retrieve all the classes that a
student attends. We return an error if the number of classes has
reached the limit of five.

### Composing transactions

Oh, just one last feature, we are told. We have students that are
trying to switch from one popular class to another. By the time they
drop one class to free up a slot for themselves, the open slot in the
other class is gone. By the time they see this and try to re-add their
old class, that slot is gone too! So, can we make it so that a student
can switch from one class to another without this worry?

Fortunately, we have FoundationDB, and this sounds an awful lot like
the transactional property of atomicity --- the all-or-nothing
behavior that we rely on. All we need to do is _compose_ the `dropout`
and `signup` function into new `switch_classes` function. This make
the `switch_classes` function exceptionally easy:

```rust
#[derive(Clone, Debug)]
struct OldClass(Class);

#[derive(Clone, Debug)]
struct NewClass(Class);

async fn switch_classes(
    tr: &FdbTransaction,
    student: Student,
    old_class: OldClass,
    new_class: NewClass,
) -> FdbResult<()> {
    let old_attends_key = AttendsKey::new(student.clone(), {
        let OldClass(class_name) = old_class.clone();
        class_name
    });
    let new_attends_key = AttendsKey::new(student.clone(), {
        let NewClass(class_name) = new_class.clone();
        class_name
    });
    if tr.get(old_attends_key).await?.is_some() &&
        tr.get(new_attends_key).await?.is_some() {
        // nothing to switch
        Ok(())
    } else {
        // switching classes
        dropout(tr, student.clone(), {
            let OldClass(class_name) = old_class;
            class_name
        })
        .await?;
        signup(tr, student.clone(), {
            let NewClass(class_name) = new_class;
            class_name
        })
        .await?;
        Ok(())
    }
}
```

The simplicity of this implementation belies the sophistication of
what FoundationDB is taking care for us.

By dropping the old class and signing up for the new one inside a
single transaction, we ensure either both steps happen, or that
neither happens. The first notable thing about `switch_classes`
function is that it is transactional, but it also calls the
transactional functions `signup` and `dropout`. Once a transaction is
created and passed in as `tr`, the calls to `dropout` and `signup`
both share the same `tr`. This ensures that they see each other's
modifications to the database, and all of the changes that both of
them make in sequence are made transactionally when the
`switch_classes` function returns. This compositional capability is
very powerful.

Also note that, if an error occurs, for example in `signup`, and the
error is not handled in `switch_classes`, then the error be propagated
to the calling function. Eventually it will reach the `run()` where we
check if the error is a retryable error. If it is not, then
transaction value is dropped, automatically rolling back all database
modifications, leaving the database completely unchanged by the
half-executed function.

### Are we done?

Yep, we're done and ready to deploy. If you want to see this entire
application in one place plus some testing code using Tokio tasks to
simulate concurrency, look at
[`class-scheduling/src/main.rs`](https://github.com/fdb-rs/website/tree/main/code/crate-fdb/class-scheduling-tutorial/class-scheduling/src/main.rs).

### Deploying and scaling

Since we store all state for this application in FoundationDB,
deploying and scaling this solution up is impressively painless. Just
run a web server, the UI, this back end, and point the whole thing at
FoundationDB. We can run as many computers with this setup as we want,
and they can all hit the database at the same time because of the
transactional integrity of FoundationDB. Also, since all of the state
in the system is stored in the database, any of these computers can
fail without any lasting consequences.

### Next steps

* See [Data
  Modeling](https://apple.github.io/foundationdb/data-modeling.html)
  for guidance on using tuples and subspaces to enable effective
  storage and retrieval of data.
* See [Developer
  Guide](https://apple.github.io/foundationdb/developer-guide.html)
  for general guidance on development using FoundationDB.
