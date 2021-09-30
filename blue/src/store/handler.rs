use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use serde_json::json;
use tokio::net::TcpStream;

use super::super::ipc::message;
use super::super::ipc::message::request::Command;
use super::super::ipc::receiver::{async_read_message, read_message};
use super::super::ipc::sender::{async_send_message, send_message};
use super::cluster::{Cluster, Node, NodeRole};
use super::serialize::persist_store;
use super::wal::WriteAheadLog;

pub async fn handle_stream<'a>(
    mut stream: TcpStream,
    store: Arc<Mutex<message::Store>>,
    store_path: Arc<&Path>,
    wal: Arc<Mutex<WriteAheadLog<'a>>>,
    cluster: Arc<Mutex<Cluster>>,
) -> io::Result<()> {
    loop {
        println!("Handling");
        let input = async_read_message::<message::Request>(&mut stream).await?;
        println!("{:?}", input);
        let mut store = store.lock().unwrap();
        let mut wal = wal.lock().unwrap();
        let mut cluster = cluster.lock().unwrap();

        match input.command {
            Some(Command::FollowRequest(c)) => {
                // follow_request_handler(c, &mut cluster).await?
                follow_request_handler(c, &mut cluster, &mut stream).await?
            }
            Some(Command::FollowResponse(c)) => {
                let cluster = follow_response_handler(c)?;
            }
            Some(Command::InitiateSession(c)) => initiate_session_handler(&mut stream, c).await?,
            Some(Command::Get(c)) => get_handler(&mut stream, c, &mut store).await?,
            Some(Command::Set(c)) => {
                set_handler(&mut stream, &c, &mut store).await?;
                persist_store(&mut store, &store_path)?;
                println!("Appending sequence #{} to WAL", wal.next_sequence);
                wal.append_message(c)?;
            }
            // Some(Command::InitiateBackup(c)) => initiate_backup_handler(c, &mut store)?,
            // Some(Command::ExecuteBackup(c)) => execute_backup_handler(c, &store_path)?,
            None => println!("Figure this out"),
        }
    }
}

async fn follow_request_handler(
    follow_request: message::FollowRequest,
    cluster: &mut Cluster,
    stream: &mut TcpStream,
) -> io::Result<()> {
    println!("Follow request handler");
    let follower_addr = SocketAddr::from_str(follow_request.addr.as_str()).unwrap();
    println!("Follower addr: {:?}", follower_addr);
    // cluster.add_follower(follower_addr).await?;
    cluster.add_follower(follower_addr, stream).await?;

    Ok(())
}

fn follow_response_handler(follow_response: message::FollowResponse) -> io::Result<Cluster> {
    println!("Follow response handler");
    println!("{:?}", follow_response);
    match follow_response.replication {
        // Synchronous
        0 => {
            // let node = Node {
            //     addr,
            //     role: NodeRole::Follower,
            //     replication: message::Replication::Sync,
            // };
            let leader = Node {
                addr: SocketAddr::from_str(&follow_response.leader)
                    .expect("Failed to parse leader addr"),
                role: NodeRole::Leader,
                replication: message::Replication::Sync,
            };
            // TODO: Add cluster sync and async followers to followers
            return Ok(Cluster {
                leader,
                sync_follower: None,
                async_followers: None,
            });
        }
        // Asynchronous
        1 => {
            // let node = Node {
            //     addr,
            //     role: NodeRole::Follower,
            //     replication: message::Replication::Sync,
            // };
            let leader = Node {
                addr: SocketAddr::from_str(&follow_response.leader)
                    .expect("Failed to parse leader addr"),
                role: NodeRole::Leader,
                replication: message::Replication::Async,
            };
            // TODO: Add cluster sync and async followers to followers
            return Ok(Cluster {
                leader,
                sync_follower: None,
                async_followers: None,
            });
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid Cluster config",
        )),
    }
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

async fn get_handler(
    stream: &mut TcpStream,
    get: message::Get,
    store: &mut message::Store,
) -> io::Result<()> {
    // TODO: Get from store or file
    println!("Getting key={}", get.key);
    let value = store.records.get(&get.key);
    let msg = match value {
        Some(v) => message::Response {
            success: true,
            message: v.clone(),
        },
        None => message::Response {
            success: true,
            message: json!(store.records).to_string(),
        },
    };
    async_send_message(msg, stream).await
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
