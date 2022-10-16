use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug)]
pub struct Cli {
    /// Path to config file
    #[clap(short)]
    pub config: String,
    /// Command to perform
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create an initial config
    Init(InitCmd),
    /// Sync metadata with peer
    Sync(SyncCmd),
    /// Request patch from peer
    Patch(PatchCmd),
    /// Show peer id
    /// If no peer nickname provided, show own peer id
    Id(IdCmd),
    /// Run the daemon
    Daemon(DaemonCmd),
}

#[derive(Debug, Args)]
pub struct InitCmd {
    pub repo: String,
    pub db_path: String,
    pub swarm_listen: String,
    pub api_listen: String,
}

#[derive(Debug, Args)]
pub struct SyncCmd {}

#[derive(Debug, Args)]
pub struct PatchCmd {}

#[derive(Debug, Args)]
pub struct IdCmd {
    nickname: Option<String>,
}

#[derive(Debug, Args)]
pub struct DaemonCmd {
    #[clap(short, default_value = "false")]
    pub foreground: bool
}
