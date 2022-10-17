use super::{Commit, Database as DatabaseTrait};

use libp2p::PeerId;
use std::path::PathBuf;

pub struct Database {
    path: PathBuf,
}

impl Database {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl DatabaseTrait for Database {
    fn contains(&self, peer: PeerId) -> bool {
        unimplemented!()
    }
    fn get_peer_id_from_nickname(&self, nickname: &str) -> Option<PeerId> {
        unimplemented!()
    }
    fn get_most_recent_common_ancestor(&self, peer: PeerId) -> Option<Commit> {
        unimplemented!()
    }
}
