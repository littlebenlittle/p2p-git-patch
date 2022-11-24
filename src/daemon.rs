use crate::api::{
    ClientId as ApiClientId, IdError, Request as ApiRequest, Response as ApiResponse,
    Server as ApiServer, UpdateError,
};
use crate::behaviour::{
    self, Behaviour, Event, GitPatchRequest, GitPatchResponse, GitPatchResponseChannel,
    PatchResponseUpdateError,
};
use crate::config::Config;
use crate::database::{self, Database};
use crate::git::{Commit, EagerRepository, Repository};

use libp2p::core::either::EitherError;
use libp2p::{
    identity::Keypair,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    // kad::{record::store::MemoryStore, {GetClosestPeersError, Kademlia, KademliaConfig, KademliaEvent, QueryResult}},
    request_response::{RequestId, RequestResponseEvent, RequestResponseMessage},
    swarm::SwarmEvent,
    Multiaddr,
    PeerId,
    Swarm,
};

use futures::channel::mpsc::{self, Receiver, Sender};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{error, fmt, io};

pub struct Service<R: Repository> {
    swarm_listen: Multiaddr,
    swarm: Swarm<Behaviour>,
    api_server: Box<dyn ApiServer>,
    database: Box<dyn Database>,
    repository: R,
    pending_update_requests: HashMap<RequestId, ApiClientId>,
    keep_serving: bool,
}

impl<R: Repository> Service<R> {
    pub async fn new(
        swarm_listen: Multiaddr,
        keypair: Keypair,
        api_server: Box<dyn ApiServer>,
        database: Box<dyn Database>,
        repository: R,
    ) -> Result<Self, io::Error> {
        let peer_id = PeerId::from(keypair.public());
        let transport = libp2p::development_transport(keypair).await?;
        let behaviour = Behaviour {
            mdns: Mdns::new(MdnsConfig::default())?,
            git_patch: behaviour::new(),
        };
        let swarm = Swarm::new(transport, behaviour, peer_id);
        Ok(Self {
            swarm_listen,
            swarm,
            api_server,
            database,
            repository,
            pending_update_requests: HashMap::new(),
            keep_serving: true,
        })
    }

    pub async fn start(mut self) -> Result<(), Box<dyn error::Error>> {
        log::debug!("daemon starting");
        self.swarm.listen_on(self.swarm_listen.clone())?;
        log::debug!("daemon entering main loop");
        while self.keep_serving {
            log::debug!("daemon waiting for next task");
            futures::select! {
                (client, req) = self.api_server.select_next_some() => {
                    use ApiRequest::*;
                    log::debug!("cmd: {req:?}");
                    match req {
                        Update { peer } => self.update(peer, client).await,
                        Patch { peer, commit_id } => self.patch(peer, commit_id),
                        Id { nickname } => self.id(nickname, client).await,
                        Shutdown => self.shutdown(client).await,
                        AddPeer {peer_id, nickname} => self.add_peer(client, peer_id, nickname).await,
                    }
                },
                e = self.swarm.select_next_some() => self.handle_swarm_event(e).await
            }
        }
        Ok(())
    }

    // ---------------- Update Handlers ----------------
    async fn update(&mut self, peer: PeerId, client: ApiClientId) {
        if !self.database.contains(peer) {
            self.api_server
                .send_response(
                    &client,
                    ApiResponse::Update(Err(UpdateError::UnknownPeerId)),
                )
                .await;
            return;
        }
        let mrca = self
            .database
            .get_most_recent_common_ancestor(peer)
            .or_else(|| Some(self.repository.root()))
            .unwrap();
        let mut path: Vec<Commit> = self
            .repository
            .ancestor_iter()
            .take_while(|c| *c != mrca)
            .collect();
        path.push(mrca);
        let req_id = self
            .swarm
            .behaviour_mut()
            .git_patch
            .send_request(&peer, GitPatchRequest::Update { path });
        self.pending_update_requests.insert(req_id, client);
    }

    fn gitpatch_update_request(
        &mut self,
        peer_path: Vec<Commit>,
        channel: GitPatchResponseChannel,
    ) {
        let response = if peer_path.is_empty() {
            Err(PatchResponseUpdateError::EmptyPath)
        } else {
            let mut response = Err(PatchResponseUpdateError::NoCommonAncestor);
            for commit in self.repository.ancestor_iter() {
                if commit.is_in(&peer_path) || commit.is_ancestor_of(peer_path.first().unwrap()) {
                    response = Ok(commit)
                }
            }
            response
        };
        match self
            .swarm
            .behaviour_mut()
            .git_patch
            .send_response(channel, GitPatchResponse::Update(response))
        {
            Ok(()) => {}
            Err(resp) => log::info!("failed to send response: {resp:?}"),
        }
    }

    async fn gitpatch_update_response(&mut self, request_id: RequestId) {
        if let Some(client) = self.pending_update_requests.remove(&request_id) {
            self.api_server
                .send_response(&client, ApiResponse::Update(Ok(())))
                .await
        } else {
            log::info!("unknown request_id: {}", request_id);
        }
    }

    // ---------------- Patch Handlers ----------------

    fn patch(&self, peer: PeerId, commit_id: git2::Oid) {
        unimplemented!()
    }

    fn gitpatch_patch_request(&self, peer: PeerId, channel: GitPatchResponseChannel) {
        unimplemented!()
    }

    fn gitpatch_patch_response(&self, peer: PeerId) {
        unimplemented!()
    }

    // ---------------- Id Handler ----------------

    async fn id(&mut self, nickname: Option<String>, client: ApiClientId) {
        let response = if let Some(nickname) = nickname {
            let peer = self.database.get_peer_id_from_nickname(&nickname);
            if let Some(peer) = peer {
                Ok(peer)
            } else {
                Err(IdError::UnknownNickname)
            }
        } else {
            Ok(*self.swarm.local_peer_id())
        };
        self.api_server
            .send_response(&client, ApiResponse::Id(response))
            .await;
    }

    // ---------------- Add Peer Handler ----------------

    async fn add_peer(&mut self, client: ApiClientId, peer_id: PeerId, nickname: String) {
        let res = match self.database.add_peer(peer_id, nickname) {
            Ok(()) => ApiResponse::AddPeer(Ok(())),
            Err(e) => ApiResponse::AddPeer(Err(e)),
        };
        self.api_server.send_response(&client, res).await
    }

    // ---------------- Shutdown Handler ----------------

    async fn shutdown(&mut self, client: ApiClientId) {
        self.keep_serving = false;
        self.api_server
            .send_response(&client, ApiResponse::Shutdown(Ok(())))
            .await;
    }

    // ---------------- Swarm Handler ----------------

    async fn handle_swarm_event<E>(&mut self, e: SwarmEvent<Event, E>) {
        use behaviour::Event::*;
        use libp2p::core::ConnectedPoint;
        use SwarmEvent::*;
        match e {
            Behaviour(GitPatch(RequestResponseEvent::Message { peer, message })) => {
                if self.database.contains(peer) {
                    match message {
                        RequestResponseMessage::Request {
                            request, channel, ..
                        } => match request {
                            GitPatchRequest::Update { path } => {
                                self.gitpatch_update_request(path, channel)
                            }
                            GitPatchRequest::Patch => self.gitpatch_patch_request(peer, channel),
                        },
                        RequestResponseMessage::Response {
                            response,
                            request_id,
                        } => match response {
                            GitPatchResponse::Update(result) => {
                                self.gitpatch_update_response(request_id).await
                            }
                            GitPatchResponse::Patch => self.gitpatch_patch_response(peer),
                        },
                    }
                } else {
                    log::info!("dropping request from unknown peer: {}", peer.to_base58())
                }
            }
            Behaviour(Mdns(MdnsEvent::Discovered(addresses))) => {
                for (peer_id, _) in addresses {
                    // self.swarm.behaviour_mut().floodsub.add_node_to_partial_view(peer_id);
                }
            }
            NewListenAddr { address, .. } => {
                println!("listening on {address}");
            }
            ConnectionEstablished {
                peer_id,
                endpoint: ConnectedPoint::Dialer { address, .. },
                ..
            } => println!("connected to {} at {address}", peer_id.to_base58()),
            _ => {}
        }
    }
}
