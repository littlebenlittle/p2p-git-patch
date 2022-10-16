mod cli;
mod daemon;
mod database;
mod config;
mod behaviour;

use cli::Cli;
use daemon::Client;
use config::Config;

use clap::Parser;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        cli::Command::Init(cmd) => {
            let config = Config::new(cmd.repo, cmd.db_path, cmd.listen.parse()?);
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
                    let mut client: Client = Client::new(config.keypair).await?;
                    client.start(config.listen).await
                })?;
            }
        }
    }
    Ok(())
}
