use clap::{Parser, Subcommand, ValueEnum};

/// A high-performance command-line statistics tracker.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Prints the shell function used to integrate t_trace with shell.
    Init(InitArgs),
    /// Manage the t_trace background daemon process.
    Daemon(DaemonArgs),
    /// Display aggregated command statistics.
    Stats,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    /// The target shell for which to generate the initialization script.
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
    /// Start the daemon process in the background.
    Run,
    /// Stop the daemon process gracefully.
    Stop,
    /// Check if the daemon process is running and responsive.
    HealthCheck,
    /// Notify the daemon process that a command is beginning.
    CommandBegin {
        #[arg()]
        pid: u32,
        #[arg()]
        command: String,
    },
    /// Notify the daemon process that a command has ended.
    CommandEnd {
        #[arg()]
        pid: u32,
        #[arg()]
        exit_code: i32,
    },
}
