use bytes::{Buf, BufMut, Bytes, BytesMut};

use fdb::database::{DatabaseOption, FdbDatabase};
use fdb::error::{FdbError, FdbResult};
use fdb::range::{Range, RangeOptions};
use fdb::transaction::{FdbTransaction, ReadTransaction, Transaction};
use fdb::tuple::Tuple;
use fdb::{Key, KeyValue, Value};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use tracing::{debug, debug_span, Instrument};

use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Sender};

use tokio_stream::StreamExt;

use std::convert::{TryFrom, TryInto};
use std::env;
use std::error::Error;

#[derive(Clone, Debug, PartialEq)]
struct Class(String);

#[derive(Clone, Debug, PartialEq)]
struct Student(String);

// ("class", class_name)
#[derive(Clone, Debug)]
struct ClassKey {
    class_name: Class,
}

impl ClassKey {
    fn new(class_name: Class) -> ClassKey {
        ClassKey { class_name }
    }
}

impl From<ClassKey> for Key {
    fn from(c: ClassKey) -> Key {
        let key_tup: (&'static str, Class) = ("class", c.class_name);

        let key_bytes = {
            let mut tup = Tuple::new();

            tup.add_string((key_tup.0).to_string());

            let Class(class_inner) = key_tup.1;
            tup.add_string(class_inner);

            tup
        }
        .pack();

        key_bytes.into()
    }
}

impl From<ClassKey> for Class {
    fn from(c: ClassKey) -> Class {
        c.class_name
    }
}

// Exists for documentation purposes.
#[allow(dead_code)]
const VALUE_CONVERTION_ERROR: i32 = 998;

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

struct ClassValue {
    val: u8,
}

impl ClassValue {
    fn new(val: u8) -> ClassValue {
        ClassValue { val }
    }

    fn get_val(&self) -> u8 {
        self.val
    }
}

impl From<ClassValue> for Value {
    fn from(c: ClassValue) -> Value {
        let val_bytes = Bytes::from({
            let mut b = BytesMut::new();
            b.put_u8(c.val);
            b
        });

        val_bytes.into()
    }
}

impl From<Value> for ClassValue {
    fn from(v: Value) -> ClassValue {
        let val = Bytes::from(v).get_u8();

        ClassValue::new(val)
    }
}

// ("attends", student, class_name)
#[derive(Clone, Debug)]
struct AttendsKey {
    student: Student,
    class_name: Class,
}

impl AttendsKey {
    fn new(student: Student, class_name: Class) -> AttendsKey {
        AttendsKey {
            student,
            class_name,
        }
    }
}

impl From<AttendsKey> for Key {
    fn from(a: AttendsKey) -> Key {
        let key_tup: (&'static str, Student, Class) = ("attends", a.student, a.class_name);

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

        key_bytes.into()
    }
}

impl TryFrom<Key> for AttendsKey {
    type Error = FdbError;

    fn try_from(key: Key) -> FdbResult<AttendsKey> {
        Tuple::from_bytes(key)
            .and_then(|tup| {
                // ("attends", student, class_name)
                if tup.get_string_ref(0)?.as_str() != "attends" {
                    return Err(FdbError::new(KEY_CONVERTION_ERROR));
                }

                let student = Student(tup.get_string_ref(1)?.to_string());

                let class_name = Class(tup.get_string_ref(2)?.to_string());

                Ok(AttendsKey::new(student, class_name))
            })
            .map_err(|_| FdbError::new(KEY_CONVERTION_ERROR))
    }
}

struct AttendsValue;

impl AttendsValue {
    fn new() -> AttendsValue {
        AttendsValue
    }
}

impl From<AttendsValue> for Value {
    fn from(_: AttendsValue) -> Value {
        let val_bytes = Bytes::new();

        val_bytes.into()
    }
}

// ("class")
struct ClassPrefix;

impl ClassPrefix {
    fn new() -> ClassPrefix {
        ClassPrefix
    }

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
}

// ("attends")
struct AttendsPrefix;

impl AttendsPrefix {
    fn new() -> AttendsPrefix {
        AttendsPrefix
    }

    fn get_range(&self) -> Range {
        // ("attends")
        let attends_tup: (&'static str,) = ("attends",);

        let attends_range = {
            let mut tup = Tuple::new();

            tup.add_string((attends_tup.0).to_string());

            tup
        }
        .range(Bytes::new());

        attends_range
    }
}

// ("attends", student)
struct AttendsStudentPrefix {
    student: Student,
}

impl AttendsStudentPrefix {
    fn new(student: Student) -> AttendsStudentPrefix {
        AttendsStudentPrefix { student }
    }

    fn get_range(&self) -> Range {
        // ("attends", student)
        let attends_student_tup: (&'static str, Student) = ("attends", self.student.clone());

        let attends_student_range = {
            let mut tup = Tuple::new();

            tup.add_string((attends_student_tup.0).to_string());

            let Student(student_inner) = attends_student_tup.1;
            tup.add_string(student_inner);

            tup
        }
        .range(Bytes::new());

        attends_student_range
    }
}

fn add_class(tr: &FdbTransaction, class_name: Class) {
    // ("class", class_name)
    let class_key = ClassKey::new(class_name);

    let class_value = ClassValue::new(100);

    tr.set(class_key, class_value);
}

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
    "chem", "bio", "cs", "geometry", "calc", "alg", "film", "music", "art", "dance",
];

const TIMES: [&str; 18] = [
    "2:00", "3:00", "4:00", "5:00", "6:00", "7:00", "8:00", "9:00", "10:00", "11:00", "12:00",
    "13:00", "14:00", "15:00", "16:00", "17:00", "18:00", "19:00",
];

fn init_class_names() -> Vec<Class> {
    let mut class_names = Vec::new();

    for level in LEVELS {
        // we can't use type here as that is a keyword in Rust.
        for typ in TYPES {
            for time in TIMES {
                class_names.push(Class(format!("{} {} {}", time, typ, level).to_string()));
            }
        }
    }

    class_names
}

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

// async fn available_classes(tr: &FdbTransaction) -> FdbResult<Vec<Class>> {
//     // ("class", ...)
//     let mut class_range_stream = ClassPrefix::new()
//         .get_range()
//         .into_stream(tr, RangeOptions::default());

//     let mut class_names = Vec::new();

//     while let Some(x) = class_range_stream.next().await {
//         let key = x?.into_key();

//         let class_key = TryInto::<ClassKey>::try_into(key)?;

//         class_names.push(class_key.into());
//     }

//     Ok(class_names)
// }

async fn available_classes(tr: &FdbTransaction) -> FdbResult<Vec<Class>> {
    // ("class", ...)
    let mut class_range_stream = ClassPrefix::new()
        .get_range()
        .into_stream(tr, RangeOptions::default());

    let mut class_names = Vec::new();

    while let Some(x) = class_range_stream.next().await {
        let (key, value) = x?.into_parts();

        let class_key = TryInto::<ClassKey>::try_into(key)?;

        let seats_available = ClassValue::from(value).get_val();

        if seats_available > 0 {
            class_names.push(class_key.into());
        }
    }

    Ok(class_names)
}

// fn signup(tr: &FdbTransaction, student: Student, class_name: Class) {
//     // ("attends", student, class_name)
//     let attends_key = AttendsKey::new(student, class_name);

//     // ""
//     let attends_value = AttendsValue::new();

//     tr.set(attends_key, attends_value);
// }

const NO_REMAINING_SEATS: i32 = 996;
const ALREADY_SIGNED_UP: i32 = 997;

// async fn signup(tr: &FdbTransaction, student: Student, class_name: Class) -> FdbResult<()> {
//     // ("attends", student, class_name)
//     let attends_key = AttendsKey::new(student, class_name.clone());

//     // ""
//     let attends_value = AttendsValue::new();

//     if tr.get(attends_key.clone()).await?.is_some() {
//         Err(FdbError::new(ALREADY_SIGNED_UP))
//     } else {
//         // ("class", class_name)
//         let class_key = ClassKey::new(class_name);

//         // Safety: It is safe to `unwrap()` here because in our data
//         // model assume that key `("class", class_name)` will *always*
//         // have seats left value.
//         let class_value = ClassValue::from(tr.get(class_key.clone()).await?.unwrap());

//         let seats_left = class_value.get_val();

//         if seats_left == 0 {
//             Err(FdbError::new(NO_REMAINING_SEATS))
//         } else {
//             let updated_class_value = ClassValue::new(seats_left - 1);

//             tr.set(class_key, updated_class_value);

//             tr.set(attends_key, attends_value);

//             Ok(())
//         }
//     }
// }

async fn get_attends_student_keyvalue(
    tr: &FdbTransaction,
    student: Student,
) -> FdbResult<Vec<KeyValue>> {
    // ("attends", student, ...)
    let mut range_stream = AttendsStudentPrefix::new(student)
        .get_range()
        .into_stream(tr, RangeOptions::default());

    let mut kvs = Vec::new();

    while let Some(x) = range_stream.next().await {
        let kv = x?;

        kvs.push(kv);
    }

    Ok(kvs)
}

const TOO_MANY_CLASSES: i32 = 995;

async fn signup(tr: &FdbTransaction, student: Student, class_name: Class) -> FdbResult<()> {
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
        let class_value = ClassValue::from(tr.get(class_key.clone()).await?.unwrap());

        let seats_left = class_value.get_val();

        if seats_left == 0 {
            Err(FdbError::new(NO_REMAINING_SEATS))
        } else {
            let attends_student_kvs = get_attends_student_keyvalue(tr, student).await?;

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

// // Unlike other bindings, we cannot name this function as `drop`,
// // because `drop` is already used in Rust.
// fn dropout(tr: &FdbTransaction, student: Student, class_name: Class) {
//     // ("attends", student, class_name)
//     let attends_key = AttendsKey::new(student, class_name);

//     tr.clear(attends_key);
// }

// Unlike other bindings, we cannot name this function as `drop`,
// because `drop` is already used in Rust.
async fn dropout(tr: &FdbTransaction, student: Student, class_name: Class) -> FdbResult<()> {
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
        let class_value = ClassValue::from(tr.get(class_key.clone()).await?.unwrap());

        let seats_left = class_value.get_val();

        let updated_class_value = ClassValue::new(seats_left + 1);

        tr.set(class_key, updated_class_value);

        tr.clear(attends_key);

        Ok(())
    }
}

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

    if tr.get(old_attends_key).await?.is_some() && tr.get(new_attends_key).await?.is_some() {
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

#[derive(Copy, Clone, Debug)]
enum Mood {
    Add,
    Dropout,
    Switch,
}

async fn indecisive_student(task_finished: Sender<()>, db: FdbDatabase, id: usize, ops: usize) {
    let student_id = format!("s{}", id);

    debug!(%student_id, "starting");

    let mut all_classes = init_class_names();

    let mut my_classes: Vec<Class> = Vec::new();

    let mut rng = StdRng::from_entropy();

    for _ in 0..ops {
        let class_count = my_classes.len();

        let mut moods = Vec::new();

        if class_count > 0 {
            moods.push(Mood::Dropout);
            moods.push(Mood::Switch);
        }

        if class_count < 5 {
            moods.push(Mood::Add);
        }

        // Safety: Fail in case we are unable to select a random mood.
        let mood = *moods.choose(&mut rng).unwrap();

        if all_classes.is_empty() {
            // all_classes empty, populating from db.
            all_classes = db
                .run(|tr| async move { available_classes(&tr).await })
                .await
                .unwrap_or_else(|err| panic!("Error occurred during `run`: {:?}", err));
        }

        match mood {
            Mood::Add => {
                // Safety: Fail in case we are unable to select a
                // random class from `all_classes`.
                let c = all_classes.choose(&mut rng).unwrap();

                let student_id_ref = &student_id;

                match db
                    .run(|tr| async move {
                        signup(&tr, Student(student_id_ref.clone()), c.clone()).await
                    })
                    .await
                {
                    Ok(()) => my_classes.push(c.clone()),
                    Err(err) => {
                        if err.code() == NO_REMAINING_SEATS {
                            // Populate available classes in the next iteration
                            all_classes.clear();
                        } else if err.code() == ALREADY_SIGNED_UP {
                            // Ignore `Mood::Add` if we have already
                            // signed up.
                        } else if err.code() == TOO_MANY_CLASSES {
                            debug!(err = "TOO_MANY_CLASSES");
                            panic!("TOO_MANY_CLASSES");
                        } else {
                            debug!(?err);
                            panic!("Error occurred during `run`: {:?}", err);
                        }
                    }
                }
            }
            Mood::Dropout => {
                // Safety: Fail in case we are unable to select a
                // random class from `my_classes`.
                let c = my_classes.choose(&mut rng).unwrap().clone();

                let student_id_ref = &student_id;
                let c_ref = &c;

                match db
                    .run(|tr| async move {
                        dropout(&tr, Student(student_id_ref.clone()), c_ref.clone()).await
                    })
                    .await
                {
                    Ok(()) => my_classes.retain(|x| *x != c),
                    Err(err) => {
                        // `dropout` should not fail.
                        debug!(?err);
                        panic!("Error occurred during `run`: {:?}", err);
                    }
                }
            }
            Mood::Switch => {
                // Safety: Fail in case we are unable to select a
                // random class from `my_classes`.
                let old_c = OldClass(my_classes.choose(&mut rng).unwrap().clone());

                // Safety: Fail in case we are unable to select a
                // random class from `all_classes`.
                let new_c = NewClass(all_classes.choose(&mut rng).unwrap().clone());

                let student_id_ref = &student_id;
                let old_c_ref = &old_c;
                let new_c_ref = &new_c;

                match db
                    .run(|tr| async move {
                        switch_classes(
                            &tr,
                            Student(student_id_ref.clone()),
                            old_c_ref.clone(),
                            new_c_ref.clone(),
                        )
                        .await
                    })
                    .await
                {
                    Ok(()) => {
                        // Remove `old_c` and add `new_c` to
                        // `my_classes` upon successful swtich.
                        let OldClass(old_class_name) = old_c;

                        my_classes.retain(|x| *x != old_class_name);

                        my_classes.push({
                            let NewClass(class_name) = new_c;
                            class_name
                        });
                    }
                    Err(err) => {
                        // Error handling for `switch_classes` is
                        // similar to `signup`, but we should not be
                        // seeing `TOO_MANY_CLASSES` and
                        // `ALREADY_SIGNED_UP` errors.
                        if err.code() == NO_REMAINING_SEATS {
                            // Populate available classes in the next iteration
                            all_classes.clear();
                        } else {
                            debug!(?err);
                            panic!("Error occurred during `run`: {:?}", err);
                        }
                    }
                }
            }
        }
    }

    drop(task_finished);

    debug!(%student_id, "finished");
}

async fn run_sim(db: FdbDatabase, students: usize, ops_per_student: usize) {
    let (task_finished, mut task_finished_recv) = mpsc::channel::<()>(1);

    for i in 0..students {
        let cloned_task_finished = task_finished.clone();
        let cloned_db = db.clone();

        tokio::spawn(
            async move {
                indecisive_student(cloned_task_finished, cloned_db, i, ops_per_student).await;
            }
            .instrument(debug_span!("indecisive_student", %i)),
        );
    }

    drop(task_finished);

    let _ = task_finished_recv.recv().await;

    debug!(
        total_transactions = students * ops_per_student,
        "transactions run"
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let fdb_cluster_file = env::var("FDB_CLUSTER_FILE").expect(
        "FDB_CLUSTER_FILE not defined!
",
    );

    unsafe {
        fdb::select_api_version(fdb::FDB_API_VERSION as i32);
        fdb::start_network();
    }

    let fdb_database = fdb::open_database(fdb_cluster_file)?;

    // 60,000 ms = 1 minute
    fdb_database.set_option(DatabaseOption::TransactionTimeout(60000))?;
    fdb_database.set_option(DatabaseOption::TransactionRetryLimit(100))?;

    let rt = Runtime::new()?;

    let cloned_fdb_database = fdb_database.clone();

    rt.block_on(async {
        let fdb_database = cloned_fdb_database;

        init(&fdb_database).await?;

        run_sim(fdb_database, 10, 10).await;

        Result::<(), Box<dyn Error>>::Ok(())
    })?;

    drop(fdb_database);

    unsafe {
        fdb::stop_network();
    }

    Ok(())
}
