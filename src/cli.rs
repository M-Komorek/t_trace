use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(InitArgs),
    Daemon(DaemonArgs),
    Client(ClientArgs),
    Stats,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    #[arg(value_enum)]
    pub shell: Shell,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Shell {
    Bash,
}

#[derive(Parser, Debug)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub command: DaemonCommands,
}

#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    Start,
    Stop,
    Status,
}

#[derive(Parser, Debug)]
pub struct ClientArgs {
    #[command(subcommand)]
    pub command: ClientCommands,
}

#[derive(Subcommand, Debug)]
pub enum ClientCommands {
    Start {
        #[arg()]
        pid: u32,
        #[arg()]
        command: String,
    },
    End {
        #[arg()]
        pid: u32,
        #[arg()]
        exit_code: i32,
    },
}
