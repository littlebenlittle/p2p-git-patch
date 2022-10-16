use libp2p::PeerId;

pub enum Request {
    /// Sync metadata with peer
    Update { peer: PeerId },
    /// Request patch from peer
    Patch { peer: PeerId, commit_id: git2::Oid },
    /// Show peer id
    /// If no peer nickname provided, show own peer id
    Id { nickname: Option<String> },
}

pub enum Response {
    Update(Result<(), UpdateError>),
    Patch(Result<(), PatchError>),
    Id(Result<PeerId, IdError>)
}

// TODO
pub enum UpdateError {
    UnknownPeerId
}
pub enum PatchError {}
pub enum IdError {
    UnknownNickname
}
