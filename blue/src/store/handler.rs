use std::io;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use prost::Message;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream as asyncTcpStream;

use super::super::ipc::message;
use super::super::ipc::message::request::Command;
use super::super::ipc::receiver::async_read_message;
use super::super::ipc::sender::{async_send_message, send_message};
use super::cluster::{Cluster, NodeRole};
use super::serialize::persist_store;
use super::wal::WriteAheadLog;

pub async fn handle_stream<'a>(
    mut stream: asyncTcpStream,
    store: Arc<Mutex<message::Store>>,
    store_path: Arc<&Path>,
    wal: Arc<Mutex<WriteAheadLog<'a>>>,
    cluster: Arc<Mutex<Cluster>>,
    role: &NodeRole,
) -> io::Result<()> {
    loop {
        println!("Handling stream: {:?}", stream);
        let input = async_read_message::<message::Request>(&mut stream).await;
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
                    Some(Command::SynchronizeRequest(c)) => {
                        synchronize_request_handler(&mut stream, c, &wal).await?
                    }
                    Some(Command::Get(c)) => get_handler(&mut stream, c, &mut store).await?,
                    Some(Command::Set(c)) => match role {
                        NodeRole::Leader => {
                            async_set_handler(&mut stream, &c, &mut store).await?;
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
                            let peer = stream.peer_addr()?;
                            println!("Replication request from {}", peer);
                            if peer == cluster.leader.addr {
                                replication_handler(&c, &mut store).await?;
                                persist_store(&mut store, &store_path)?;
                                println!("Appending sequence #{} to WAL", wal.next_sequence);
                                wal.append_message(c.clone())?;
                            } else {
                                let response = message::Response {
                                    success: false,
                                    message: "Only the leader accepts writes".to_string(),
                                };
                                async_send_message(response, &mut stream).await?;
                            }
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
    stream: &mut asyncTcpStream,
) -> io::Result<()> {
    let follower_addr = SocketAddr::from_str(follow_request.follower_addr.as_str()).unwrap();
    println!("Adding follower: {:?}", follower_addr);
    cluster.add_follower(follower_addr, stream).await?;
    Ok(())
}

async fn initiate_session_handler(
    stream: &mut asyncTcpStream,
    initiate_session: message::InitiateSession,
) -> io::Result<()> {
    println!("Initiating session with {}", initiate_session.name);
    let welcome = message::Welcome {
        message: "Welcome to Blue!\n".to_string(),
    };
    async_send_message(welcome, stream).await
}

async fn synchronize_request_handler<'a>(
    stream: &mut asyncTcpStream,
    request_synchronize: message::SynchronizeRequest,
    wal: &WriteAheadLog<'a>,
) -> io::Result<()> {
    println!(
        "Synchronization requested: {:?}\nFrom: {:?}",
        request_synchronize, stream
    );
    let seq_start = request_synchronize.next_sequence;
    let synchronize_response = message::SynchronizeResponse {
        latest_sequence: wal.next_sequence - 1,
    };
    async_send_message(synchronize_response, stream).await?;
    println!("WAL Messages: {:?}", wal.messages());
    for item in wal.messages() {
        println!("Seq: {}", item[0].0);
        if item[0].0 >= seq_start {
            let seq_bytes = item[0].0.to_le_bytes();
            stream.write_all(&seq_bytes).await?;
            async_send_message(item[0].1.clone(), stream).await?
        };
    }
    Ok(())
}

async fn get_handler(
    stream: &mut asyncTcpStream,
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

async fn async_set_handler(
    stream: &mut asyncTcpStream,
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

pub fn set_handler(
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
    send_message(msg, stream)?;

    Ok(())
}

async fn replication_handler(
    // stream: &mut asyncTcpStream,
    set: &message::Set,
    store: &mut message::Store,
) -> io::Result<()> {
    // TODO: Send message back to leader to remove from WAL
    println!("Replication storing {}={}", set.key, set.value);
    store.records.insert(set.key.clone(), set.value.clone());
    Ok(())
}

pub fn synchronize_handler(
    // stream: &mut asyncTcpStream,
    set: &message::Set,
    store: &mut message::Store,
) -> io::Result<()> {
    // TODO: Send message back to leader to remove from WAL
    println!("Synchronize storing {}={}", set.key, set.value);
    store.records.insert(set.key.clone(), set.value.clone());
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
