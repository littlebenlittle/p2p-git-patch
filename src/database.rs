use libp2p::PeerId;
use crate::git::Commit;

pub trait Database {
    /// Returns true of the database contains the provided peer id
    fn contains(&self, peer: PeerId) -> bool;
    /// Get the peer id associated with this nickname
    fn get_peer_id_from_nickname(&self, nickname: impl ToString) -> Option<PeerId>;
    /// If known, returns most recent common ancestor between the local
    /// repo and peer's repo. If unknown, returns repo root.
    fn get_most_recent_common_ancestor(&self, peer: PeerId) -> Option<Commit>;
}
