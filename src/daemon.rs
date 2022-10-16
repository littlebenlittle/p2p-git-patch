use crate::behaviour::{
    self, Behaviour, GitPatchExchangeCodec, GitPatchExchangeProtocol, GitPatchRequest,
    GitPatchResponse,
};
use crate::database::Database;

use libp2p::{
    identity::Keypair,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    // kad::{record::store::MemoryStore, {GetClosestPeersError, Kademlia, KademliaConfig, KademliaEvent, QueryResult}},
    request_response::{ProtocolSupport, RequestResponse},
    Multiaddr,
    PeerId,
    Swarm,
};

use futures::stream::FusedStream;
use futures::stream::StreamExt;
use std::{error::Error, io};

pub enum Command {
    /// Sync metadata with peer
    Sync { peer: PeerId },
    /// Request patch from peer
    Patch { peer: PeerId, commit_id: git2::Oid },
    /// Show peer id
    /// If no peer nickname provided, show own peer id
    Id { nickname: Option<String> },
}

pub struct Client<S, D>
where
    S: FusedStream<Item = Command> + Unpin,
    D: Database,
{
    swarm: Swarm<Behaviour>,
    cmd_stream: S,
    database: D,
}

impl<S, D> Client<S, D>
where
    S: FusedStream<Item = Command> + Unpin,
    D: Database,
{
    pub async fn new(keypair: Keypair, cmd_stream: S, database: D) -> Result<Self, io::Error> {
        let peer_id = PeerId::from(keypair.public());
        println!("Local peer id: {:?}", peer_id);
        let transport = libp2p::development_transport(keypair).await?;
        let behaviour = Behaviour {
            mdns: Mdns::new(MdnsConfig::default())?,
            git_patch: behaviour::new(),
        };
        let swarm = Swarm::new(transport, behaviour, peer_id);
        Ok(Self {
            swarm,
            cmd_stream,
            database,
        })
    }

    pub async fn start(&mut self, swarm_addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on(swarm_addr)?;
        loop {
            use behaviour::Event::Mdns;
            use libp2p::core::ConnectedPoint;
            use libp2p::swarm::SwarmEvent::*;
            use Command::*;
            futures::select! {
                cmd = self.cmd_stream.select_next_some() => match cmd {
                    Sync { peer } => self.sync(peer),
                    Patch { peer, commit_id } => self.patch(peer, commit_id),
                    Id { nickname } => self.id(nickname)
                },
                e = self.swarm.select_next_some() => match e {
                    Behaviour(Mdns(MdnsEvent::Discovered(addresses))) => {
                        for (peer_id, _) in addresses {
                            // self.swarm.behaviour_mut().floodsub.add_node_to_partial_view(peer_id);
                        }
                    }
                    NewListenAddr{address, ..} => {
                        println!("listening on {address}");
                    }
                    ConnectionEstablished{
                        peer_id,
                        endpoint: ConnectedPoint::Dialer{address, ..},
                        ..
                    } => println!("connected to {} at {address}", peer_id.to_base58()),
                    _ => {}
                }
            }
        }
    }

    fn sync(&self, peer: PeerId) {
        self.swarm
            .behaviour_mut()
            .git_patch
            .send_request(&peer, GitPatchRequest{});
    }

    fn patch(&self, peer: PeerId, commit_id: git2::Oid) {
        unimplemented!()
    }

    fn id(&self, nickname: Option<String>) {
        unimplemented!()
    }
}
