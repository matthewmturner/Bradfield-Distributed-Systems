use std::io::{self, ErrorKind, Read};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;

use prost::Message;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream as asyncTcpStream;

use crate::ipc::receiver::{async_read_message, read_message};

use super::super::ipc::message;
use super::super::ipc::message::request::Command;
use super::super::ipc::message::{FollowRequest, FollowResponse, Replication};
use super::super::ipc::sender::{async_send_message, send_message};
use super::super::store::handler::synchronize_handler;
use super::wal::WriteAheadLog;

#[derive(Debug, Clone)]
pub enum NodeRole {
    Leader,
    Follower,
}

impl FromStr for NodeRole {
    type Err = ();

    fn from_str(input: &str) -> Result<NodeRole, Self::Err> {
        match input {
            "leader" | "Leader" => Ok(NodeRole::Leader),
            "follower" | "Follower" => Ok(NodeRole::Follower),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub addr: SocketAddr,
    pub role: NodeRole,
    pub replication: Replication,
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub leader: Node,
    pub sync_follower: Option<Node>,
    pub async_followers: Option<Vec<Node>>,
}

impl Cluster {
    pub async fn new<'a>(
        addr: SocketAddr,
        role: &NodeRole,
        leader: SocketAddr,
        wal: &WriteAheadLog<'a>,
        store: &mut message::Store,
    ) -> io::Result<Cluster> {
        match role {
            NodeRole::Leader => {
                println!("Creating new cluster with role Leader");
                let leader = Node {
                    addr,
                    role: NodeRole::Leader,
                    replication: Replication::Sync,
                };
                return Ok(Cluster {
                    leader,
                    sync_follower: None,
                    async_followers: None,
                });
            }
            NodeRole::Follower => {
                println!("Joining cluster (leader: {:?}) as follower", leader);
                let follow_request = message::Request {
                    command: Some(Command::FollowRequest(FollowRequest {
                        follower_addr: addr.to_string(),
                    })),
                };
                let mut stream = asyncTcpStream::connect(leader).await?;
                async_send_message(follow_request, &mut stream).await?;
                let follow_response = async_read_message::<FollowResponse>(&mut stream).await?;
                stream.shutdown().await?;
                let cluster = match follow_response.replication {
                    // Synchronous
                    0 => {
                        let leader = Node {
                            addr: leader,
                            role: NodeRole::Leader,
                            replication: Replication::Sync,
                        };
                        // TODO: Add cluster sync and async followers to followers
                        Ok(Cluster {
                            leader,
                            sync_follower: None,
                            async_followers: None,
                        })
                    }
                    // Asynchronous
                    1 => {
                        let leader = Node {
                            addr: leader,
                            role: NodeRole::Leader,
                            replication: Replication::Async,
                        };
                        // TODO: Add cluster sync and async followers to followers
                        Ok(Cluster {
                            leader,
                            sync_follower: None,
                            async_followers: None,
                        })
                    }
                    _ => Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "Invalid Cluster config",
                    )),
                };
                Cluster::synchronize(leader, &wal, store)?;
                cluster
            }
        }
    }
    pub async fn add_follower(
        &mut self,
        addr: SocketAddr,
        stream: &mut asyncTcpStream,
    ) -> io::Result<()> {
        // TODO: Handle failure when adding following
        // Sync follower already exists
        match self.sync_follower {
            Some(_) => {
                let node = Node {
                    addr,
                    role: NodeRole::Follower,
                    replication: Replication::Async,
                };
                println!("Adding async follower: {:?}", node);
                let followers = self.async_followers.as_mut();
                match followers {
                    Some(f) => f.push(node),
                    None => self.async_followers = Some(vec![node]),
                }
                let response = FollowResponse {
                    leader: self.leader.addr.to_string(),
                    replication: 1,
                };
                async_send_message(response, stream).await?;
            }
            None => {
                let node = Node {
                    addr,
                    role: NodeRole::Follower,
                    replication: Replication::Sync,
                };
                println!("Adding sync follower: {:?}", node);
                let response = FollowResponse {
                    leader: self.leader.addr.to_string(),
                    replication: 0,
                };
                // async_send_message(response, stream).await?;
                let r = async_send_message(response, stream).await;
                println!("{:?}", r);
                match r {
                    Ok(_) => self.sync_follower = Some(node),
                    Err(_) => println!("Failed to communicate with follower"),
                }
                // self.sync_follower = Some(node);
                stream.shutdown().await?;
            }
        }
        Ok(())
    }

    pub async fn replicate<M>(
        message: M,
        sync_follower: &Option<Node>,
        async_followers: &Option<Vec<Node>>,
    ) -> io::Result<()>
    where
        M: Message + Clone,
    {
        if let Some(node) = sync_follower {
            Cluster::send_to_follower(&node, message.clone())?;
        }
        if let Some(nodes) = async_followers {
            for node in nodes {
                Cluster::async_send_to_followers(node, message.clone()).await?;
            }
        }

        Ok(())
    }

    fn send_to_follower<M: Message>(node: &Node, message: M) -> io::Result<()> {
        println!("Replicating to: {:?}", node);
        let mut stream = TcpStream::connect(node.addr)?;
        send_message(message, &mut stream)?;
        Ok(())
    }

    async fn async_send_to_followers<M: Message>(node: &Node, message: M) -> io::Result<()> {
        println!("Replicating to: {:?}", node);
        let mut stream = asyncTcpStream::connect(node.addr).await?;
        async_send_message(message, &mut stream).await?;
        Ok(())
    }

    fn synchronize(
        leader: SocketAddr,
        wal: &WriteAheadLog,
        store: &mut message::Store,
    ) -> io::Result<()> {
        println!("Synchronizing to leader");
        let mut stream = TcpStream::connect(leader)?;
        let sync_request = message::Request {
            command: Some(Command::SynchronizeRequest(message::SynchronizeRequest {
                next_sequence: wal.next_sequence,
            })),
        };
        send_message(sync_request, &mut stream)?;
        let synchronize_response = read_message::<message::SynchronizeResponse>(&mut stream)?;
        let latest_sequence = synchronize_response.latest_sequence;
        println!("Latest sequence: {}", latest_sequence);
        loop {
            let mut seq_bytes = [0u8; 8];
            stream.read_exact(&mut seq_bytes)?;
            let sequence = u64::from_le_bytes(seq_bytes);
            let set = read_message::<message::Set>(&mut stream)?;
            synchronize_handler(&set, store)?;
            if sequence == latest_sequence {
                break;
            }
        }
        Ok(())
    }
}
