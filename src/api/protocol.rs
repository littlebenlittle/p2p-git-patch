use libp2p::PeerId;

pub enum Request {
    /// Sync metadata with peer
    Update { peer: PeerId },
    /// Request patch from peer
    Patch { peer: PeerId, commit_id: git2::Oid },
    /// Show peer id
    /// If no peer nickname provided, show own peer id
    Id { nickname: Option<String> },
    /// Shut down the server
    Shutdown,
}

#[derive(Debug)]
pub enum Response {
    Update(Result<(), UpdateError>),
    Patch(Result<(), PatchError>),
    Id(Result<PeerId, IdError>),
    Shutdown(Result<(), ShutdownError>),
}

#[derive(Debug)]
pub enum UpdateError {
    UnknownPeerId
}
#[derive(Debug)]
pub enum PatchError {}

#[derive(Debug)]
pub enum ShutdownError {}

#[derive(Debug)]
pub enum IdError {
    UnknownNickname
}
