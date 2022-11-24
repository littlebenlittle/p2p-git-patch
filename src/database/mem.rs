use crate::api::protocol::AddPeerError;

use super::{Commit, Database as DatabaseTrait};

use libp2p::PeerId;
use std::{path::PathBuf, collections::HashMap};

pub struct Database {
    peers: HashMap<String, PeerId>,
}

impl Database {
    pub fn new() -> Self {
        Self { peers: HashMap::new() }
    }
}

impl DatabaseTrait for Database {
    fn contains(&self, peer: PeerId) -> bool {
        unimplemented!()
    }
    fn get_peer_id_from_nickname(&self, nickname: &str) -> Option<PeerId> {
        self.peers.get(nickname).map(|id| id.clone())
    }
    fn get_most_recent_common_ancestor(&self, peer: PeerId) -> Option<Commit> {
        unimplemented!()
    }
    fn add_peer(&mut self, peer_id: PeerId, nickname: String) -> Result<(), AddPeerError> {
        log::debug!("adding new peer to database");
        if self.peers.contains_key(&nickname) {
            log::debug!("nickname already exists");
            Err(AddPeerError::NicknameAlreadyExists)
        } else {
            log::debug!("add nick to db");
            self.peers.insert(nickname, peer_id);
            Ok(())
        }
    }
}

