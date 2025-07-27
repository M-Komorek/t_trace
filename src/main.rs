use anyhow::Result;
use clap::Parser;
use t_trace::cli::{Cli, Commands, DaemonArgs, DaemonCommands};
use t_trace::{client, daemon, logging};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon(DaemonArgs { command }) => match command {
            DaemonCommands::Start => {
                let _guard = logging::setup_daemon_logging()?;
                tracing::info!("Starting daemon...");
                daemon::run().await?;
            }
            DaemonCommands::Status => {
                client::run_status_check().await?;
            }
        },
    }

    Ok(())
}
