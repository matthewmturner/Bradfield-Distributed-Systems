use std::io::{self, ErrorKind};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;

use prost::Message;

use crate::ipc::receiver::read_message;

use super::super::ipc::message;
use super::super::ipc::message::request::Command;
use super::super::ipc::message::{FollowRequest, FollowResponse, Replication};
use super::super::ipc::sender::send_message;

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
    leader: Node,
    sync_follower: Option<Node>,
    async_followers: Option<Vec<Node>>,
}

impl Cluster {
    pub fn new(addr: SocketAddr, role: NodeRole, leader: SocketAddr) -> io::Result<Cluster> {
        println!("Creating new cluster");
        println!("{}{:?}{}", addr, role, leader);
        match role {
            NodeRole::Leader => {
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
                let follow_request = message::Request {
                    command: Some(Command::FollowRequest(FollowRequest {
                        addr: addr.to_string(),
                    })),
                };
                let mut stream = TcpStream::connect(leader)?;
                println!("Follow Request: {:?}", follow_request);
                send_message(follow_request, &mut stream)?;
                println!("Follow request sent");
                let follow_response = read_message::<FollowResponse>(&mut stream)?;
                println!("Follow response received");
                match follow_response.replication {
                    // Synchronous
                    0 => {
                        let node = Node {
                            addr,
                            role: NodeRole::Follower,
                            replication: Replication::Sync,
                        };
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
                        let node = Node {
                            addr,
                            role: NodeRole::Follower,
                            replication: Replication::Sync,
                        };
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
    pub fn add_follower(&mut self, addr: SocketAddr) -> io::Result<()> {
        println!("Adding follower");
        match self.sync_follower {
            Some(_) => {
                println!("Adding async follower");
                let node = Node {
                    addr,
                    role: NodeRole::Follower,
                    replication: Replication::Async,
                };
                let followers = self.async_followers.as_mut();
                followers.unwrap().push(node);
                let mut stream = TcpStream::connect(self.leader.addr)?;
                let response = message::Request {
                    command: Some(Command::FollowResponse(FollowResponse { replication: 1 })),
                };
                send_message(response, &mut stream)?;
            }
            None => {
                println!("Adding sync follower");
                let node = Node {
                    addr,
                    role: NodeRole::Follower,
                    replication: Replication::Sync,
                };
                println!("{:?}", node);
                self.sync_follower = Some(node);
                let mut stream = TcpStream::connect(addr)?;
                let response = message::Request {
                    command: Some(Command::FollowResponse(FollowResponse { replication: 0 })),
                };
                send_message(response, &mut stream)?;
            }
        }
        Ok(())
    }
}
