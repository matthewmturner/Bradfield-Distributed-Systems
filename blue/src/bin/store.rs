use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};

use structopt::StructOpt;
use tokio::net::TcpListener;

extern crate blue;

use blue::store::args;
use blue::store::deserialize::deserialize_store;
use blue::store::handler::handle_stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = args::Opt::from_args();
    let addr = format!("{}:{}", opt.host, opt.port);
    let listener = TcpListener::bind(addr).await?;

    let store_path = Path::new("data.pb");
    let store = deserialize_store(store_path)?;
    let store_path = Arc::new(store_path);
    let store = Arc::new(Mutex::new(store));
    println!("Blue launched. Waiting for incoming connection");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Incoming request from {}", addr);
        let store = Arc::clone(&store);
        let store_path = Arc::clone(&store_path);
        handle_stream(stream, store, store_path).await?;
    }
}
