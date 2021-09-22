use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod ipc;
mod store;

use store::deserialize::deserialize_store;
use store::executor::ThreadPool;
use store::handler::handle_stream;
use store::serialize::serialize_store;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let pool = ThreadPool::new(4);

    // let store_path = Path::new("data.json");
    let p_store_path = Path::new("data.pb");

    let mut pstore = deserialize_store(&p_store_path)?;
    println!("{:?}", pstore);

    // // // let store: HashMap<String, String> = match store_path.exists() {
    // // //     true => {
    // // //         let existing_store = fs::read_to_string(&store_path)?;
    // // //         serde_json::from_str(&existing_store)?
    // // //     }
    // // //     false => HashMap::new(),
    // // // };
    // // pstore.records.extend(store.clone());
    // let b = serialize_store(&pstore);
    // fs::write(&"data.pb", &b)?;

    println!("{:?}", pstore);
    // let store = Arc::new(Mutex::new(store));
    // let store_path = Arc::new(store_path);

    let pstore = Arc::new(Mutex::new(pstore));
    let p_store_path = Arc::new(p_store_path);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let store = Arc::clone(&pstore);
        let store_path = Arc::clone(&p_store_path);

        pool.execute(move || {
            handle_stream(stream, pstore, store_path);
        });
    }
    Ok(())
}

// NOTE: Old implementation pre multi threading
// fn main() -> Result<(), Box<dyn Error>> {
//     let listener = TcpListener::bind("127.0.0.1:7878")?;
//     let pool = ThreadPool::new(4);

//     for stream in listener.incoming() {
//         let stream = stream.unwrap();

//         thread::spawn(|| {
//             handle_connection(stream);
//         });
//     }
//     Ok(())
// }
