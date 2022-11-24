use libp2p::PeerId;

#[derive(Debug, PartialEq)]
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
    /// Add a new peer with nickname
    AddPeer { peer_id: PeerId, nickname: String },
}

#[derive(Debug, PartialEq)]
pub enum Response {
    Update(Result<(), UpdateError>),
    Patch(Result<(), PatchError>),
    Id(Result<PeerId, IdError>),
    Shutdown(Result<(), ShutdownError>),
    AddPeer(Result<(), AddPeerError>),
}

#[derive(Debug, PartialEq)]
pub enum UpdateError {
    UnknownPeerId,
}
#[derive(Debug, PartialEq)]
pub enum PatchError {}

#[derive(Debug, PartialEq)]
pub enum ShutdownError {}

#[derive(Debug, PartialEq)]
pub enum IdError {
    UnknownNickname,
}

#[derive(Debug, PartialEq)]
pub enum AddPeerError {
    NicknameAlreadyExists
}
