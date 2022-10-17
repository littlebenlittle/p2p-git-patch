mod yaml;

pub use yaml::Database as YamlDatabase;

use crate::git::Commit;

use libp2p::PeerId;

use std::path::PathBuf;

pub trait Database {
    /// Returns true of the database contains the provided peer id
    fn contains(&self, peer: PeerId) -> bool;
    /// Get the peer id associated with this nickname
    fn get_peer_id_from_nickname(&self, nickname: &str) -> Option<PeerId>;
    /// If known, returns most recent common ancestor between the local
    /// repo and peer's repo. If unknown, returns repo root.
    fn get_most_recent_common_ancestor(&self, peer: PeerId) -> Option<Commit>;
}

impl TryFrom<PathBuf> for Box<dyn Database> {
    type Error = Error;
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let db = match path.extension() {
            None => return Err(Error::IsDirectory),
            Some(value) => match value.to_str() {
                None => return Err(Error::InvalidFileExension(path)),
                Some("yaml") => YamlDatabase::new(path.to_path_buf()),
                Some(ext) => return Err(Error::UnhandledExnension(path)),
            },
        };
        Ok(Box::new(db))
    }
}

#[derive(Debug)]
pub enum Error {
    IsDirectory,
    InvalidFileExension(PathBuf),
    UnhandledExnension(PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IsDirectory => f.write_str("is a directory"),
            Self::InvalidFileExension(path) => {
                f.write_str(&format!("could not determine file extension for {path:?}"))
            }
            Self::UnhandledExnension(path) => {
                f.write_str(&format!("unandled file extension for {path:?}"))
            }
        }
    }
}

impl std::error::Error for Error {}
