use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};

use blue::{handle_stream, ThreadPool};

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let pool = ThreadPool::new(4);

    let store_path = Path::new("data.json");

    let store: HashMap<String, String> = match store_path.exists() {
        true => {
            let existing_store = fs::read_to_string(&store_path)?;
            serde_json::from_str(&existing_store)?
        }
        false => HashMap::new(),
    };
    let store = Arc::new(Mutex::new(store));
    let store_path = Arc::new(store_path);

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
