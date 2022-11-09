mod api;
mod behaviour;
mod cli;
mod config;
mod daemon;
mod database;
mod git;

use api::Server;
use cli::Cli;
use config::Config;
use daemon::Service;
use database::Database;
use git::EagerRepository;

use clap::Parser;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        cli::Command::Init(cmd) => {
            let config = Config::new(
                &cmd.repo,
                &cmd.db_path,
                cmd.swarm_listen.parse()?,
                cmd.api_listen.parse()?,
            );
            if std::path::PathBuf::from(&cli.config).exists() {
                println!("config path already exists")
            } else {
                std::fs::write(cli.config, config.to_yaml()?)?;
            }
        }
        cli::Command::Sync(cmd) => {
            unimplemented!()
        }
        cli::Command::Patch(cmd) => {
            unimplemented!()
        }
        cli::Command::Id(cmd) => {
            unimplemented!()
        }
        cli::Command::Daemon(cmd) => {
            let config = Config::from_path(cli.config)?;
            if cmd.foreground {
                async_std::task::block_on(async move {
                    match run_daemon(config).await {
                        Ok(_) => {}
                        Err(e) => log::error!("{e:?}"),
                    }
                })
            } else {
                unimplemented!("background daemon unimplemented")
            }
        }
    }
    Ok(())
}

async fn run_daemon(config: Config) -> Result<(), Box<dyn Error>> {
    let api_server = Box::<dyn Server>::try_from(config.api_listen)?;
    let database = Box::<dyn Database>::try_from(config.database_path)?;
    let repository = EagerRepository::try_from(config.repo_dir)?;
    let mut service = Service::new(
        config.swarm_listen,
        config.keypair,
        api_server,
        database,
        repository,
    )
    .await?;
    service.start().await
}

#[cfg(test)]
mod test {
    use async_std::task::JoinHandle;
    use libp2p::{
        identity::{self, Keypair},
        PeerId,
    };
    use std::path::PathBuf;
    use temp_dir::TempDir;

    type Result = std::result::Result<(), Box<dyn Error>>;
    use std::error::Error;

    use super::{
        api::{self, Client},
        Database, EagerRepository, Service,
    };

    fn spawn_service(
        db_path: PathBuf,
        repo_dir: PathBuf,
        swarm_addr: &str,
        keypair: identity::Keypair,
    ) -> std::result::Result<
        (api::TestClient, JoinHandle<std::result::Result<(), String>>),
        Box<dyn Error>,
    > {
        let mut api_server = api::TestServer::new();
        let api = api_server.client()?;
        let swarm_addr = swarm_addr.parse()?;
        let database = Box::<dyn Database>::try_from(db_path)?;
        let repository = EagerRepository::try_from(repo_dir)?;
        let jh = async_std::task::spawn(async move {
            let mut service = Service::new(swarm_addr, keypair, api_server, database, repository)
                .await
                .or_else(|e| Err(format!("failed to create service: {e:?}")))?;
            service
                .start()
                .await
                .or_else(|e| Err(format!("failed to create service: {e:?}")))
        });
        Ok((api, jh))
    }

    /// daemon can be started
    #[test]
    fn can_run_daemon() -> Result {
        env_logger::init();
        let tmp = TempDir::new()?;
        log::debug!("starting service...");
        let (mut api, jh) = spawn_service(
            tmp.path().join("db.yaml"),
            tmp.path().join("repo"),
            "/ip4/127.0.0.1/udp/0",
            Keypair::generate_ed25519(),
        )?;
        log::debug!("sending shutdown...");
        api.shutdown()?;
        async_std::task::block_on(async move { jh.await })?;
        Ok(())
    }

    /// two daemons can find each other on loopback device via mdns
    #[test]
    fn daemons_can_connect_via_mdns() -> Result {
        let tmp = TempDir::new()?;
        let (mut api_a, jh_a) = spawn_service(
            tmp.path().join("A/db.yaml"),
            tmp.path().join("A/repo"),
            "/ip4/127.0.0.1/udp/0",
            Keypair::generate_ed25519(),
        )?;
        let (mut api_b, jh_b) = spawn_service(
            tmp.path().join("db.yaml"),
            tmp.path().join("repo"),
            "/ip4/127.0.0.1/udp/0",
            Keypair::generate_ed25519(),
        )?;
        let id_a = api_a.get_id()?;
        let id_b = api_b.get_id()?;
        api_a.add_peer(id_b, "A")?;
        api_b.add_peer(id_a, "B")?;
        assert_eq!(api_a.get_peer("A")?, id_a);
        assert_eq!(api_a.get_peer("B")?, id_b);
        api_a.shutdown()?;
        api_a.shutdown()?;
        async_std::task::block_on(async move {
            jh_a.await?;
            jh_b.await?;
            std::result::Result::<(), String>::Ok(())
        })?;
        Ok(())
    }

    /// daemon returns own peer id when id is queried
    /// without partameters
    #[test]
    fn get_own_peer_id() -> Result {
        let tmp = TempDir::new()?;
        let keypair: Keypair = Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id();
        let (mut api, jh) = spawn_service(
            tmp.path().join("db.yaml"),
            tmp.path().join("repo"),
            "/ip4/127.0.0.1/udp/0",
            keypair,
        )?;
        assert_eq!(api.get_id()?, peer_id);
        api.shutdown()?;
        async_std::task::block_on(async move { jh.await })?;
        Ok(())
    }

    /// daemon returns peer id of known peer
    /// when nickname is provided to id
    #[test]
    fn get_peer_id_by_nickname() -> Result {
        let tmp = TempDir::new()?;
        let (mut api, jh) = spawn_service(
            tmp.path().join("db.yaml"),
            tmp.path().join("repo"),
            "/ip4/127.0.0.1/udp/0",
            Keypair::generate_ed25519(),
        )?;
        let peer_id: PeerId = Keypair::generate_ed25519().public().to_peer_id();
        api.add_peer(peer_id, "A")?;
        assert_eq!(api.get_peer("A")?, peer_id);
        api.shutdown()?;
        async_std::task::block_on(async move { jh.await })?;
        Ok(())
    }

    /// daemon returns error when unknown nickname
    /// is provided to id
    #[test]
    fn get_peer_id_by_nickname_with_unknown_nickname_fails() -> Result {
        let tmp = TempDir::new()?;
        let (mut api, jh) = spawn_service(
            tmp.path().join("db.yaml"),
            tmp.path().join("repo"),
            "/ip4/127.0.0.1/udp/0",
            Keypair::generate_ed25519(),
        )?;
        use super::api::protocol;
        match api.get_peer("A") {
            Err(api::ClientError::IdError(protocol::IdError::UnknownNickname)) => {}
            Err(e) => panic!(
                "expected {:?}; got {e}",
                api::ClientError::IdError(protocol::IdError::UnknownNickname)
            ),
            Ok(id) => panic!("expected error; got {id}"),
        }
        api.shutdown()?;
        async_std::task::block_on(async move { jh.await })?;
        Ok(())
    }
}
