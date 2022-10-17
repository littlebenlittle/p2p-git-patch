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
    use super::*;
    use async_std::task::JoinHandle;
    use futures::channel;
    use libp2p::identity::Keypair;
    use std::path::PathBuf;

    /// daemon can be started
    #[test]
    fn can_run_daemon() -> Result<(), Box<dyn Error>> {
        let (shutdown_tx, shutdown_rx) = channel::oneshot::channel();
        let api_server = api::TestServer::new(shutdown_rx)?;
        let db_path = PathBuf::from("/tmp/p2p-gitpatch-test/db.yaml");
        let repo_dir = PathBuf::from("/tmp/p2p-gitpatch-test/repo");
        let swarm_addr = "/ip4/127.0.0.1/udp/1234".parse()?;
        let database = Box::<dyn Database>::try_from(db_path)?;
        let repository = EagerRepository::try_from(repo_dir)?;
        let jh = async_std::task::spawn(async move {
            let mut service = Service::new(
                swarm_addr,
                Keypair::generate_ed25519(),
                api_server,
                database,
                repository,
            )
            .await
            .or_else(|e| Err(format!("failed to create service: {e:?}")))?;
            service
                .start()
                .await
                .or_else(|e| Err(format!("failed to create service: {e:?}")))
        });
        async_std::task::block_on(async move {
            match shutdown_tx.send(()) {
                Ok(()) => {}
                Err(_) => {
                    return Err("failed to send shutdown to test server");
                }
            }
            match jh.await {
                Ok(()) => {}
                Err(s) => {
                    return Err(format!("failed to join threads with server: {s}"));
                }
            };
            Ok(())
        })?;
        Ok(())
    }

    /// two daemons can find each other on loopback device via mdns
    #[test]
    fn daemons_can_connect_via_mdns() {
        unimplemented!();
    }

    /// daemon returns own peer id when id is queried
    /// without partameters
    #[test]
    fn get_own_peer_id() {
        unimplemented!();
    }

    /// daemon returns peer id of known peer
    /// when nickname is provided to id
    #[test]
    fn get_peer_id_by_nickname() {
        unimplemented!();
    }

    /// daemon returns error when unknown nickname
    /// is provided to id
    #[test]
    fn get_peer_id_by_nickname_with_unknown_nickname_fails() {
        unimplemented!();
    }
}
