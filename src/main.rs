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
                cmd.repo,
                cmd.db_path,
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
                async_std::task::block_on(async move { run_daemon(config).await })?
            } else {
                unimplemented!("background daemon unimplemented")
            }
        }
    }
    Ok(())
}

async fn run_daemon(config: Config) -> Result<(), Box<dyn std::error::Error>> {
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
    use super::config::ConfigSerde;
    use super::*;

    /// create an inital configuration file
    #[test]
    fn can_create_initial_config_file() -> Result<(), Box<dyn Error>> {
        let config = Config::new(
            "/tmp/p2p-gitpatch-test/repo".to_owned(),
            "/tmp/p2p-gitpatch-test/db.yaml".to_owned(),
            "/ip4/127.0.0.1/udp/1234".parse()?,
            "/unix/tmp/p2p-gitpatch-test/socket".parse::<config::MultiaddrUnixSocket>()?,
        );
        let yaml = config.to_yaml()?;
        let parsed = serde_yaml::from_str::<ConfigSerde>(&yaml)?;
        assert_eq!(ConfigSerde::from(&config), parsed);
        Ok(())
    }

    /// daemon can be started
    #[test]
    fn can_run_daemon() {
        let config = ConfigSerde {
            keypair: "".to_owned(),
            repo_dir: "".to_owned(),
            database_path: "".to_owned(),
            swarm_listen: "".to_owned(),
            api_listen: "".to_owned(),
        };
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
