use std::error::Error;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use env_logger;
use log::{debug, info};
use structopt::StructOpt;
use tokio::net::TcpListener;

extern crate blue;

use blue::ipc::message;
use blue::store::args;
use blue::store::cluster::{Cluster, NodeRole};
use blue::store::deserialize::deserialize_store;
use blue::store::handler::handle_stream;
use blue::store::wal::WriteAheadLog;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // TODO: Handle incorrect or already used addr
    let opt = args::Opt::from_args();
    let addr = SocketAddr::from_str(format!("{}:{}", opt.host, opt.port).as_str())?;
    let role = NodeRole::from_str(opt.role.as_str()).unwrap();
    let leader_addr = match role {
        NodeRole::Leader => addr,
        NodeRole::Follower => SocketAddr::from_str(opt.follow.unwrap().as_str())?,
    };

    let wal_name = addr.to_string().replace(".", "").replace(":", "");
    let wal_full_name = format!("wal{}.log", wal_name);
    let wal_path = PathBuf::from(wal_full_name);
    let mut wal = match wal_path.exists() {
        true => {
            info!("Existing WAL found");
            WriteAheadLog::open(&wal_path)?
        }
        false => {
            info!("Creating WAL");
            WriteAheadLog::new(&wal_path)?
        }
    };
    debug!("WAL: {:?}", wal);

    let store_pth = format!("{}.pb", addr);
    let store_path = Path::new(&store_pth);
    let mut store = match store_path.exists() {
        true => deserialize_store(store_path)?,
        false => message::Store::default(),
    };
    // let mut store = deserialize_store(store_path)?;

    let listener = TcpListener::bind(addr).await?;
    let cluster = Cluster::new(addr, &role, leader_addr, &mut wal, &mut store).await?;

    let store_path = Arc::new(store_path);
    let store = Arc::new(Mutex::new(store));

    let wal = Arc::new(Mutex::new(wal));
    let cluster = Arc::new(Mutex::new(cluster));
    info!("Blue launched. Waiting for incoming connection");

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Incoming request from {}", addr);
        let store = Arc::clone(&store);
        let store_path = Arc::clone(&store_path);
        let wal = Arc::clone(&wal);
        let cluster = Arc::clone(&cluster);
        handle_stream(stream, store, store_path, wal, cluster, &role).await?;
    }
}
