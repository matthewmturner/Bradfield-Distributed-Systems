use std::io;
use std::net::{SocketAddr, TcpStream};

use crate::ipc::receiver::read_message;

use super::super::ipc::message::{FollowRequest, FollowResponse, Replication};
use super::super::ipc::sender::send_message;

#[derive(Debug)]
enum NodeRole {
    Leader,
    Follower,
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
    async_follower: Option<Vec<Node>>,
}

impl Cluster {
    pub fn new(addr: SocketAddr, role: NodeRole, leader: SocketAddr) -> io::Result<Cluster> {
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
                    async_follower: None,
                });
            }
            NodeRole::Follower => {
                let follow_request = FollowRequest { addr };
                let mut stream = TcpStream::connect(addr)?;
                send_message(follow_request, &mut stream)?;
                let follow_response = read_message::<FollowResponse>(&mut stream)?;
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
                            async_follower: None,
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
                            async_follower: None,
                        });
                    }
                }
            }
        }
    }
}
