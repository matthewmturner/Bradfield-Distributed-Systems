use std::error::Error;
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};

extern crate blue;

use blue::store::deserialize::deserialize_store;
use blue::store::executor::ThreadPool;
use blue::store::handler::handle_stream;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let pool = ThreadPool::new(4);

    let store_path = Path::new("data.pb");
    let store = deserialize_store(&store_path)?;
    let store_path = Arc::new(store_path);
    let store = Arc::new(Mutex::new(store));
    println!("Blue launched. Waiting for incoming connection");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let store = Arc::clone(&store);
        let store_path = Arc::clone(&store_path);

        pool.execute(move || {
            handle_stream(stream, store, store_path);
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
