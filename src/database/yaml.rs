use super::{Commit, Database as DatabaseTrait};

use libp2p::PeerId;

pub struct Database {}

impl Database {
    pub fn new(path: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {})
    }
}

impl DatabaseTrait for Database {
    fn contains(&self, peer: PeerId) -> bool {
        unimplemented!()
    }
    fn get_peer_id_from_nickname(&self, nickname: impl ToString) -> Option<PeerId> {
        unimplemented!()
    }
    fn get_most_recent_common_ancestor(&self, peer: PeerId) -> Option<Commit> {
        unimplemented!()
    }
}
