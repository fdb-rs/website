// For `fdb_database` inside the async closure.
#![allow(unused_variables)]

use tokio::runtime::Runtime;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let fdb_cluster_file = env::var("FDB_CLUSTER_FILE").expect("FDB_CLUSTER_FILE not defined!");

    unsafe {
        fdb::select_api_version(710);
        fdb::start_network();
    }

    let fdb_database = fdb::open_database(fdb_cluster_file)?;

    let rt = Runtime::new()?;

    let cloned_fdb_database = fdb_database.clone();

    rt.block_on(async {
        let fdb_database = cloned_fdb_database;

        // your main async app here

        Result::<(), Box<dyn Error>>::Ok(())
    })?;

    drop(fdb_database);

    unsafe {
        fdb::stop_network();
    }

    Ok(())
}
