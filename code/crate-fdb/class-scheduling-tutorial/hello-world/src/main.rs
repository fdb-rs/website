use fdb::transaction::{ReadTransaction, Transaction};
use fdb::tuple::Tuple;

use tokio::runtime::Runtime;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        fdb::select_api_version(630);
        fdb::start_network();
    }

    let fdb_cluster_file = env::var("FDB_CLUSTER_FILE").expect(
	"FDB_CLUSTER_FILE not defined!",
    );

    let fdb_database = fdb::open_database(fdb_cluster_file)?;

    let rt = Runtime::new()?;

    let cloned_fdb_database = fdb_database.clone();

    rt.block_on(async {
        let fdb_database = cloned_fdb_database;

        // Run an operation on the database
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

        // Get the value of 'hello' from the database
        let hello = fdb_database
            .run(|tr| async move {
                // Safety: Safe to unwrap because we just put in a value
                // above.
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

        Result::<(), Box<dyn Error>>::Ok(())
    })?;

    drop(fdb_database);

    unsafe {
        fdb::stop_network();
    }

    Ok(())
}
