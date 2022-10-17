use libp2p::{
    identity::{self, ed25519, Keypair},
    Multiaddr,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum MultiaddrUnixSocket {
    Multiaddr(Multiaddr),
    UnixSocket(PathBuf),
}

impl std::str::FromStr for MultiaddrUnixSocket {
    type Err = MultiaddrUnixSocketError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Multiaddr>() {
            Ok(a) => Ok(Self::Multiaddr(a)),
            Err(m_e) => {
                match s
                    .match_indices('/')
                    .nth(1)
                    .map(|(index, _)| s.split_at(index))
                {
                    Some((proto, path)) => {
                        if proto == "/unix" {
                            Ok(Self::UnixSocket(PathBuf::from(path)))
                        } else {
                            Err(MultiaddrUnixSocketError::ParseFailure(m_e))
                        }
                    }
                    None => Err(MultiaddrUnixSocketError::EmptyPath(s.to_owned())),
                }
            }
        }
    }
}

impl fmt::Display for MultiaddrUnixSocket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Multiaddr(addr) => f.write_str(&format!("{addr}")),
            Self::UnixSocket(path) => f.write_str(&format!("/unix/{}", path.to_str().unwrap())),
        }
    }
}

#[derive(Debug)]
pub enum MultiaddrUnixSocketError {
    ParseFailure(libp2p::multiaddr::Error),
    EmptyPath(String),
}

impl fmt::Display for MultiaddrUnixSocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParseFailure(e) => f.write_str(&format!("couldn't parse multiaddr: {e:?}")),
            Self::EmptyPath(s) => {
                f.write_str(&format!("fragment following '/unix/' is empty for {s}"))
            }
        }
    }
}

impl Error for MultiaddrUnixSocketError {}

pub struct Config {
    /// Multi-codec encoded keypair
    pub keypair: identity::Keypair,
    /// Path to git repository on filesystem
    pub repo_dir: PathBuf,
    /// Path to file for peer database table
    pub database_path: PathBuf,
    /// listen address for swarm
    pub swarm_listen: Multiaddr,
    /// listen adderr for api
    pub api_listen: MultiaddrUnixSocket,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct ConfigSerde {
    /// Multibase encoded serializtion of ed25519 keypair
    pub keypair: String,
    /// Path to git repository on filesystem
    pub repo_dir: String,
    /// Path to file for peer database table
    pub database_path: String,
    /// listen address for swarm
    pub swarm_listen: String,
    /// listen address for api
    pub api_listen: String,
}

impl Config {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let file = std::fs::File::open(path.as_ref())?;
        let config: ConfigSerde = serde_yaml::from_reader(file)?;
        config.try_into()
    }
}

impl TryFrom<ConfigSerde> for Config {
    type Error = Box<dyn Error>;
    fn try_from(config: ConfigSerde) -> Result<Self, Self::Error> {
        let (_, mut keypair_bytes) = multibase::decode(config.keypair)?;
        let keypair = ed25519::Keypair::decode(&mut keypair_bytes)?;
        Ok(Self {
            keypair: Keypair::Ed25519(keypair),
            repo_dir: PathBuf::from(config.repo_dir),
            database_path: PathBuf::from(config.database_path),
            swarm_listen: config.swarm_listen.parse()?,
            api_listen: config.api_listen.parse()?,
        })
    }
}

impl From<&Config> for ConfigSerde {
    fn from(config: &Config) -> Self {
        Self {
            keypair: multibase::encode(
                multibase::Base::Base58Btc,
                match &config.keypair {
                    Keypair::Ed25519(k) => ed25519::Keypair::encode(&k),
                },
            ),
            repo_dir: config.repo_dir.to_str().unwrap().to_owned(),
            database_path: config.database_path.to_str().unwrap().to_owned(),
            swarm_listen: config.swarm_listen.to_string(),
            api_listen: config.api_listen.to_string(),
        }
    }
}

impl Config {
    pub fn new(
        repo_dir: String,
        db_path: String,
        swarm_listen: Multiaddr,
        api_listen: MultiaddrUnixSocket,
    ) -> Self {
        Self {
            keypair: Keypair::generate_ed25519(),
            repo_dir: PathBuf::from(repo_dir),
            database_path: PathBuf::from(db_path),
            swarm_listen,
            api_listen,
        }
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&ConfigSerde::from(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_multiaddr() -> Result<(), Box<dyn Error>> {
        let s = "/ip4/127.0.0.1/tcp/8080";
        assert_eq!(
            s.parse::<MultiaddrUnixSocket>()?,
            MultiaddrUnixSocket::Multiaddr(s.parse()?)
        );
        Ok(())
    }

    #[test]
    fn parse_unix_socket() -> Result<(), Box<dyn Error>> {
        let s = "/unix/path/to/socket";
        assert_eq!(
            s.parse::<MultiaddrUnixSocket>()?,
            MultiaddrUnixSocket::Multiaddr("/path/to/socket".parse()?)
        );
        Ok(())
    }
}
