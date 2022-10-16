mod api;
mod behaviour;
mod cli;
mod config;
mod daemon;
mod database;
mod git;

use api::UnixSocketServer;
use cli::Cli;
use config::Config;
use daemon::Service;
use database::YamlDatabase;
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
                async_std::task::block_on(async move {
                    let cmd_stream = UnixSocketServer::try_from(config.api_listen)?;
                    let database = YamlDatabase::new(config.database_path)?;
                    let repository = EagerRepository::new(config.repo_dir)?;
                    let mut service = Service::new(
                        config.swarm_listen,
                        config.keypair,
                        cmd_stream,
                        database,
                        repository,
                    )
                    .await?;
                    service.start().await
                });
            }
        }
    }
    Ok(())
}
