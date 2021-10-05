use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use serde_json::json;
use tokio::net::TcpStream;

use super::super::ipc::message;
use super::super::ipc::message::request::Command;
use super::super::ipc::receiver::async_read_message;
use super::super::ipc::sender::async_send_message;
use super::cluster::{Cluster, NodeRole};
use super::serialize::persist_store;
use super::wal::WriteAheadLog;

pub async fn handle_stream<'a>(
    mut stream: TcpStream,
    store: Arc<Mutex<message::Store>>,
    store_path: Arc<&Path>,
    wal: Arc<Mutex<WriteAheadLog<'a>>>,
    cluster: Arc<Mutex<Cluster>>,
    role: &NodeRole,
) -> io::Result<()> {
    loop {
        println!("Handling stream: {:?}", stream);
        let input = async_read_message::<message::Request>(&mut stream).await;
        println!("Stream input: {:?}", input);
        match input {
            Ok(r) => {
                let mut store = store.lock().unwrap();
                let mut wal = wal.lock().unwrap();
                let mut cluster = cluster.lock().unwrap();

                match r.command {
                    Some(Command::FollowRequest(c)) => {
                        follow_request_handler(c, &mut cluster, &mut stream).await?;
                        println!("New cluster: {:?}", cluster);
                    }
                    Some(Command::InitiateSession(c)) => {
                        initiate_session_handler(&mut stream, c).await?
                    }
                    Some(Command::RequestSynchronize(c)) => {
                        request_synchronize_handler(&mut stream, c).await
                    }
                    Some(Command::Get(c)) => get_handler(&mut stream, c, &mut store).await?,
                    Some(Command::Set(c)) => match role {
                        NodeRole::Leader => {
                            set_handler(&mut stream, &c, &mut store).await?;
                            persist_store(&mut store, &store_path)?;
                            println!("Appending sequence #{} to WAL", wal.next_sequence);
                            wal.append_message(c.clone())?;
                            let r = message::Request {
                                command: Some(Command::Set(c)),
                            };
                            Cluster::replicate(r, &cluster.sync_follower, &cluster.async_followers)
                                .await?;
                            println!("Replicated set command");
                        }
                        NodeRole::Follower => {
                            let response = message::Response {
                                success: false,
                                message: "Only the leader accepts writes".to_string(),
                            };
                            async_send_message(response, &mut stream).await?;
                        }
                    },
                    // Some(Command::InitiateBackup(c)) => initiate_backup_handler(c, &mut store)?,
                    // Some(Command::ExecuteBackup(c)) => execute_backup_handler(c, &store_path)?,
                    None => println!("Figure this out"),
                }
            }
            Err(_) => {
                println!("Unknown command");
                return Ok(());
            }
        }
    }
}

async fn follow_request_handler(
    follow_request: message::FollowRequest,
    cluster: &mut Cluster,
    stream: &mut TcpStream,
) -> io::Result<()> {
    let follower_addr = SocketAddr::from_str(follow_request.follower_addr.as_str()).unwrap();
    println!("Adding follower: {:?}", follower_addr);
    cluster.add_follower(follower_addr, stream).await?;
    Ok(())
}

async fn initiate_session_handler(
    stream: &mut TcpStream,
    initiate_session: message::InitiateSession,
) -> io::Result<()> {
    println!("Initiating session with {}", initiate_session.name);
    let welcome = message::Welcome {
        message: "Welcome to Blue!\n".to_string(),
    };
    async_send_message(welcome, stream).await
}

async fn request_synchronize_handler(
    stream: &mut TcpStream,
    request_synchronize: message::RequestSynchronize,
) {
    println!("Synchronization requested");
}

async fn get_handler(
    stream: &mut TcpStream,
    get: message::Get,
    store: &mut message::Store,
) -> io::Result<()> {
    println!("Getting key={}", get.key);
    let m = if &get.key == "" {
        message::Response {
            success: false,
            message: json!(store.records).to_string(),
        }
    } else {
        let value = store.records.get(&get.key);
        let msg = match value {
            Some(v) => message::Response {
                success: true,
                message: v.clone(),
            },
            None => message::Response {
                success: false,
                message: format!("Unknown key '{}'", &get.key),
            },
        };
        msg
    };
    async_send_message(m, stream).await
}

async fn set_handler(
    stream: &mut TcpStream,
    set: &message::Set,
    store: &mut message::Store,
) -> io::Result<()> {
    println!("Storing {}={}", set.key, set.value);
    store.records.insert(set.key.clone(), set.value.clone());
    let msg = message::Response {
        success: true,
        message: "Succesfully wrote key to in memory story".to_string(),
    };
    async_send_message(msg, stream).await?;

    Ok(())
}

// async fn sync_request_handler(own_sequence: u64, cluster: Cluster) {
//     println!("Requesting sync from sequence #{}", own_sequence);
//     let leader = cluster.leader.addr;
//     let stream = TcpStream::connect(leader).await?;
// }

// async fn sync_response_handler(requested_sequence: u64, cluster: Cluster) {
//     println!("Sending sync from sequence #{}", requested_sequence);
//     let stream = TcpStream::connect(leader).await?;
// }

// fn initiate_backup_handler(
//     backup: message::InitiateBackup,
//     store: &mut message::Store,
// ) -> io::Result<()> {
//     println!("Backing up database to {}", backup.addr);
//     let mut backup_stream = TcpStream::connect(backup.addr)?;
//     // TODO: Can this be sent without cloning??
//     async_send_message(store.clone(), &mut backup_stream)?;
//     Ok(())
// }

// fn execute_backup_handler(backup: message::ExecuteBackup, path: &Path) -> io::Result<()> {
//     println!("Storing backup database");
//     if let Some(mut store) = backup.store {
//         persist_store(&mut store, &path);
//     }
//     Ok(())
// }
