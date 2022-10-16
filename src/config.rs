use libp2p::{
    identity::{self, ed25519, Keypair},
    Multiaddr,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    path::{Path, PathBuf},
};

pub struct Config {
    /// Multi-codec encoded keypair
    pub keypair: identity::Keypair,
    /// Path to git repository on filesystem
    pub repo_dir: PathBuf,
    /// Path to file for peer database table
    pub database_path: PathBuf,
    /// listen address for swarm
    pub listen: Multiaddr,
}

#[derive(Serialize, Deserialize)]
struct ConfigSerde {
    /// Multibase encoded serializtion of ed25519 keypair
    keypair: String,
    /// Path to git repository on filesystem
    repo_dir: String,
    /// Path to file for peer database table
    database_path: String,
    /// listen address for swarm
    listen: String,
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
            listen: config.listen.parse()?,
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
            listen: config.listen.to_string(),
        }
    }
}

impl Config {
    pub fn new(repo_dir: String, db_path: String, listen: Multiaddr) -> Self {
        Self {
            keypair: Keypair::generate_ed25519(),
            repo_dir: PathBuf::from(repo_dir),
            database_path: PathBuf::from(db_path),
            listen,
        }
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&ConfigSerde::from(self))
    }
}
