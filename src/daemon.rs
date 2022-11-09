use crate::api::{
    ClientId as ApiClientId, IdError, Request as ApiRequest, Response as ApiResponse,
    Server as ApiServer, UpdateError,
};
use crate::behaviour::{
    self, Behaviour, GitPatchRequest, GitPatchResponse, GitPatchResponseChannel,
    PatchResponseUpdateError,
};
use crate::config::Config;
use crate::database::{self, Database};
use crate::git::{Commit, EagerRepository, Repository};

use libp2p::{
    identity::Keypair,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    // kad::{record::store::MemoryStore, {GetClosestPeersError, Kademlia, KademliaConfig, KademliaEvent, QueryResult}},
    request_response::{RequestId, RequestResponseEvent, RequestResponseMessage},
    Multiaddr,
    PeerId,
    Swarm,
};

use futures::channel::mpsc;
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
    api_response_queue: Vec<(ApiClientId, ApiResponse)>,
    pending_update_requests: HashMap<RequestId, ApiClientId>,
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
            api_response_queue: Vec::new(),
            pending_update_requests: HashMap::new(),
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn error::Error>> {
        self.swarm.listen_on(self.swarm_listen.clone())?;
        loop {
            use behaviour::Event::*;
            use libp2p::core::ConnectedPoint;
            use libp2p::swarm::SwarmEvent::*;
            futures::select! {
                (client, cmd) = self.api_server.select_next_some() => {
                    use ApiRequest::*;
                    log::debug!("cmd: {cmd:?}");
                    match cmd {
                        Update { peer } => self.update(peer, client),
                        Patch { peer, commit_id } => self.patch(peer, commit_id),
                        Id { nickname } => self.id(nickname, client),
                        Shutdown => break,
                    }
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
                                    GitPatchResponse::Update(result) => self.gitpatch_update_response(peer, request_id, result).await,
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
        Ok(())
    }

    // ---------------- Update Handlers ----------------
    fn update(&mut self, peer: PeerId, mut client: ApiClientId) {
        if !self.database.contains(peer) {
            self.api_response_queue
                .push((client, ApiResponse::Update(Err(UpdateError::UnknownPeerId))));
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
        peer: PeerId,
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

    async fn gitpatch_update_response(
        &mut self,
        peer: PeerId,
        request_id: RequestId,
        error: behaviour::UpdateResult,
    ) {
        if let Some(mut client) = self.pending_update_requests.remove(&request_id) {
            self.api_response_queue
                .push((client, ApiResponse::Update(Ok(()))))
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

    fn id(&mut self, nickname: Option<String>, client: ApiClientId) {
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
