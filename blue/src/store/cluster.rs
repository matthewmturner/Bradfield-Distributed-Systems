use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::str::FromStr;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::ipc::receiver::async_read_message;

use super::super::ipc::message;
use super::super::ipc::message::request::Command;
use super::super::ipc::message::{FollowRequest, FollowResponse, Replication};
use super::super::ipc::sender::async_send_message;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Node {
    pub addr: SocketAddr,
    pub role: NodeRole,
    pub replication: Replication,
}

#[derive(Debug)]
pub struct Cluster {
    pub leader: Node,
    pub sync_follower: Option<Node>,
    pub async_followers: Option<Vec<Node>>,
}

impl Cluster {
    pub async fn new(addr: SocketAddr, role: NodeRole, leader: SocketAddr) -> io::Result<Cluster> {
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
                let mut stream = TcpStream::connect(leader).await?;
                async_send_message(follow_request, &mut stream).await?;
                let follow_response = async_read_message::<FollowResponse>(&mut stream).await?;
                stream.shutdown().await?;
                match follow_response.replication {
                    // Synchronous
                    0 => {
                        let leader = Node {
                            addr: leader,
                            role: NodeRole::Leader,
                            replication: Replication::Sync,
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
                        let leader = Node {
                            addr: leader,
                            role: NodeRole::Leader,
                            replication: Replication::Async,
                        };
                        // TODO: Add cluster sync and async followers to followers
                        return Ok(Cluster {
                            leader,
                            sync_follower: None,
                            async_followers: None,
                        });
                    }
                    _ => Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "Invalid Cluster config",
                    )),
                }
            }
        }
    }
    pub async fn add_follower(
        &mut self,
        addr: SocketAddr,
        stream: &mut TcpStream,
    ) -> io::Result<()> {
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
                // let response = message::Request {
                //     command: Some(Command::FollowResponse(FollowResponse {
                //         leader: self.leader.addr.to_string(),
                //         replication: 1,
                //     })),
                // };
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
                self.sync_follower = Some(node);
                let response = FollowResponse {
                    leader: self.leader.addr.to_string(),
                    replication: 0,
                };
                async_send_message(response, stream).await?;
                stream.shutdown().await?;
            }
        }
        Ok(())
    }
}
