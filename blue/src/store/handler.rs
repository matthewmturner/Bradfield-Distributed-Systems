use std::io;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use log::{debug, error, info};
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream as asyncTcpStream;
use tokio::sync::Mutex;

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
    store_path: Arc<PathBuf>,
    wal: Arc<Mutex<WriteAheadLog>>,
    cluster: Arc<Mutex<Cluster>>,
    role: Arc<NodeRole>,
) -> io::Result<()> {
    loop {
        info!("Handling stream: {:?}", stream);
        let input = async_read_message::<message::Request>(&mut stream).await;
        match input {
            Ok(r) => {
                let mut store = store.lock().await;
                let mut wal = wal.lock().await;
                let mut cluster = cluster.lock().await;

                match r.command {
                    Some(Command::FollowRequest(follow)) => {
                        follow_request_handler(follow, &mut cluster, &mut stream).await?;
                        debug!("New cluster: {:?}", cluster);
                    }
                    Some(Command::InitiateSession(initiate_session)) => {
                        initiate_session_handler(&mut stream, initiate_session).await?
                    }
                    Some(Command::SynchronizeRequest(synchronize_request)) => {
                        synchronize_request_handler(&mut stream, synchronize_request, &wal).await?
                    }
                    Some(Command::Get(get)) => get_handler(&mut stream, get, &mut store).await?,
                    Some(Command::Set(set)) => match *role {
                        NodeRole::Leader => {
                            async_set_handler(&mut stream, &set, &mut store).await?;
                            persist_store(&mut store, &store_path)?;
                            let sequence = wal.next_sequence;
                            debug!("Appending sequence #{} to WAL", sequence);
                            wal.append_message(&set)?;
                            let r = message::Request {
                                command: Some(Command::ReplicateSet(message::ReplicateSet {
                                    leader_addr: cluster.leader.addr.to_string(),
                                    set: Some(set),
                                    sequence,
                                })),
                            };
                            Cluster::replicate(r, &cluster.sync_follower, &cluster.async_followers)
                                .await?;
                            info!("Replicated set command");
                        }
                        NodeRole::Follower => {
                            let response = message::Response {
                                success: false,
                                message: "Only the leader accepts writes".to_string(),
                            };
                            async_send_message(response, &mut stream).await?;
                            error!("Only the leader accepts writes");
                        }
                    },
                    Some(Command::ReplicateSet(replicate_set)) => {
                        let peer = stream.peer_addr()?;
                        info!("Replication request from {}", peer);
                        if peer == cluster.leader.addr {
                            error!("Leader does not accept replication requests");
                        } else {
                            replicate_set_handler(&replicate_set, &mut store)?;
                            persist_store(&mut store, &store_path)?;
                            wal.append_message(&replicate_set)?;
                        }
                    }
                    Some(Command::ReplicateResponse(replicate_response)) => {
                        println!("{:?}", replicate_response)
                    }
                    None => error!("Figure this out"),
                }
            }
            Err(_) => {
                error!("Unknown command");
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
    info!("Adding follower: {:?}", follower_addr);
    cluster.add_follower(follower_addr, stream).await?;
    Ok(())
}

async fn initiate_session_handler(
    stream: &mut asyncTcpStream,
    initiate_session: message::InitiateSession,
) -> io::Result<()> {
    info!("Initiating session with {}", initiate_session.name);
    let welcome = message::Welcome {
        message: "Welcome to Blue!\n".to_string(),
    };
    async_send_message(welcome, stream).await
}

async fn synchronize_request_handler<'a>(
    stream: &mut asyncTcpStream,
    request_synchronize: message::SynchronizeRequest,
    wal: &WriteAheadLog,
) -> io::Result<()> {
    info!(
        "Synchronization requested: {:?}\nFrom: {:?}",
        request_synchronize, stream
    );
    let seq_start = request_synchronize.next_sequence;

    let synchronize_response = message::SynchronizeResponse {
        latest_sequence: wal.next_sequence - 1,
    };
    async_send_message(synchronize_response, stream).await?;
    if seq_start == wal.next_sequence {
        info!("Synchronization not required. Follower already up to date");
        return Ok(());
    }
    let messages = wal.clone().messages()?;
    debug!("WAL Messages: {:?}", messages);
    for item in messages {
        if item.0 >= seq_start {
            let seq_bytes = item.0.to_le_bytes();
            stream.write_all(&seq_bytes).await?;
            debug!("Sending sequence: {}", item.0);
            async_send_message(item.1.clone(), stream).await?
        };
        info!("Synchronization complete");
    }
    Ok(())
}

async fn get_handler(
    stream: &mut asyncTcpStream,
    get: message::Get,
    store: &mut message::Store,
) -> io::Result<()> {
    info!("Getting key={}", get.key);
    let m = if get.key.is_empty() {
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
    info!("Storing {}={}", set.key, set.value);
    store.records.insert(set.key.clone(), set.value.clone());
    let msg = message::Response {
        success: true,
        message: "Succesfully wrote key to in memory store".to_string(),
    };
    async_send_message(msg, stream).await?;

    Ok(())
}

pub fn set_handler(
    stream: &mut TcpStream,
    set: &message::Set,
    store: &mut message::Store,
) -> io::Result<()> {
    info!("Storing {}={}", set.key, set.value);
    store.records.insert(set.key.clone(), set.value.clone());
    let msg = message::Response {
        success: true,
        message: "Succesfully wrote key to in memory store".to_string(),
    };
    send_message(msg, stream)?;

    Ok(())
}

pub fn replicate_set_handler(
    replicate_set: &message::ReplicateSet,
    store: &mut message::Store,
) -> io::Result<()> {
    if let Some(set) = replicate_set.clone().set {
        info!("Storing {}={}", set.key, set.value);
        store.records.insert(set.key, set.value);
    }

    let mut stream = TcpStream::connect(&replicate_set.leader_addr)?;
    let msg = message::Request {
        command: Some(Command::ReplicateResponse(message::ReplicateResponse {
            success: true,
            sequence: replicate_set.sequence,
        })),
    };
    send_message(msg, &mut stream)?;

    Ok(())
}

pub fn synchronize_handler(
    // stream: &mut asyncTcpStream,
    set: &message::Set,
    store: &mut message::Store,
) -> io::Result<()> {
    // TODO: Send message back to leader to remove from WAL
    store.records.insert(set.key.clone(), set.value.clone());
    info!("Synchronized {}={} from leader", set.key, set.value);
    Ok(())
}
