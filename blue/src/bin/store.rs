use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use structopt::StructOpt;
use tokio::net::TcpListener;

extern crate blue;

use blue::store::args;
use blue::store::deserialize::deserialize_store;
use blue::store::handler::handle_stream;
use blue::store::wal::WriteAheadLog;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = args::Opt::from_args();
    let addr = format!("{}:{}", opt.host, opt.port);
    let listener = TcpListener::bind(addr).await?;

    let store_path = Path::new("data.pb");
    let store = deserialize_store(store_path)?;
    //  TODO: Are Arc / Mutex needed?
    let store_path = Arc::new(store_path);
    let store = Arc::new(Mutex::new(store));

    let wal_path = PathBuf::from("wal0000.log");
    let wal = match wal_path.exists() {
        true => {
            println!("Existing WAL found");
            WriteAheadLog::open(&wal_path)?
        }
        false => {
            println!("Creating WAL");
            WriteAheadLog::new(&wal_path)?
        }
    };
    let wal = Arc::new(Mutex::new(wal));
    println!("Blue launched. Waiting for incoming connection");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Incoming request from {}", addr);
        let store = Arc::clone(&store);
        let store_path = Arc::clone(&store_path);
        let wal = Arc::clone(&wal);
        handle_stream(stream, store, store_path, wal).await?;
    }
}
