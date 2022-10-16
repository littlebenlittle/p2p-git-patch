use crate::api::{Client, IdError, Request as ApiRequest, Response as ApiResponse, UpdateError};
use crate::behaviour::{
    self, Behaviour, GitPatchRequest, GitPatchResponse, GitPatchResponseChannel,
    PatchResponseUpdateError,
};
use crate::database::Database;
use crate::git::{Commit, Repository};

use libp2p::{
    identity::Keypair,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    // kad::{record::store::MemoryStore, {GetClosestPeersError, Kademlia, KademliaConfig, KademliaEvent, QueryResult}},
    request_response::{RequestId, RequestResponseEvent, RequestResponseMessage},
    Multiaddr,
    PeerId,
    Swarm,
};

use futures::stream::FusedStream;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::{error::Error, io};

pub struct Service<S, D, R, C>
where
    S: FusedStream<Item = (C, ApiRequest)> + Unpin,
    D: Database,
    R: Repository,
    C: Client,
{
    swarm_listen: Multiaddr,
    swarm: Swarm<Behaviour>,
    cmd_stream: S,
    database: D,
    repository: R,
    api_response_queue: Vec<(C, ApiResponse)>,
    pending_update_requests: HashMap<RequestId, C>,
}

impl<S, D, R, C> Service<S, D, R, C>
where
    S: FusedStream<Item = (C, ApiRequest)> + Unpin,
    D: Database,
    R: Repository,
    C: Client,
{
    pub async fn new(
        swarm_listen: Multiaddr,
        keypair: Keypair,
        cmd_stream: S,
        database: D,
        repository: R,
    ) -> Result<Self, io::Error> {
        let peer_id = PeerId::from(keypair.public());
        println!("Local peer id: {:?}", peer_id);
        let transport = libp2p::development_transport(keypair).await?;
        let behaviour = Behaviour {
            mdns: Mdns::new(MdnsConfig::default())?,
            git_patch: behaviour::new(),
        };
        let swarm = Swarm::new(transport, behaviour, peer_id);
        Ok(Self {
            swarm_listen,
            swarm,
            cmd_stream,
            database,
            repository,
            api_response_queue: Vec::new(),
            pending_update_requests: HashMap::new(),
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on(self.swarm_listen.clone())?;
        loop {
            use behaviour::Event::*;
            use libp2p::core::ConnectedPoint;
            use libp2p::swarm::SwarmEvent::*;
            use ApiRequest::*;
            futures::select! {
                (client, cmd) = self.cmd_stream.select_next_some() => match cmd {
                    Update { peer } => self.update(peer, client),
                    Patch { peer, commit_id } => self.patch(peer, commit_id),
                    Id { nickname } => self.id(nickname, client)
                },
                e = self.swarm.select_next_some() => match e {
                    Behaviour(GitPatch(RequestResponseEvent::Message{ peer, message })) => {
                        if self.database.contains(peer) {
                            match message {
                                RequestResponseMessage::Request { request, channel, .. } => match request {
                                    GitPatchRequest::Update { path } => self.gitpatch_update_request(peer, path, channel),
                                    GitPatchRequest::Patch => self.gitpatch_patch_request(peer, channel),
                                }
                                RequestResponseMessage::Response { response, request_id } => match response {
                                    GitPatchResponse::Update(result) => self.gitpatch_update_response(peer, request_id, result),
                                    GitPatchResponse::Patch => self.gitpatch_patch_response(peer),
                                }
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

    // ---------------- Update Handlers ----------------
    fn update(&self, peer: PeerId, client: C) {
        if !self.database.contains(peer) {
            client.send_response(ApiResponse::Update(Err(UpdateError::UnknownPeerId)))
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
        &self,
        peer: PeerId,
        peer_path: Vec<Commit>,
        channel: GitPatchResponseChannel,
    ) {
        if peer_path.is_empty() {
            self.swarm.behaviour_mut().git_patch.send_response(
                channel,
                GitPatchResponse::Update(Err(PatchResponseUpdateError::EmptyPath)),
            );
        }
        for commit in self.repository.ancestor_iter() {
            if commit.is_in(peer_path) || commit.is_ancestor_of(peer_path.first().unwrap()) {
                self.swarm
                    .behaviour_mut()
                    .git_patch
                    .send_response(channel, GitPatchResponse::Update(Ok(commit)));
                return;
            }
        }
        self.swarm.behaviour_mut().git_patch.send_response(
            channel,
            GitPatchResponse::Update(Err(PatchResponseUpdateError::NoCommonAncestor)),
        );
    }

    fn gitpatch_update_response(
        &self,
        peer: PeerId,
        request_id: RequestId,
        error: behaviour::UpdateResult,
    ) {
        if let Some(client) = self.pending_update_requests.get(&request_id) {
            client.send_response(ApiResponse::Update(Ok(())));
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

    fn id(&self, nickname: Option<String>, client: C) {
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
        self.api_response_queue
            .push((client, ApiResponse::Id(response)));
    }
}
