use anyhow::Result;
use clap::Parser;
use daemonize::Daemonize;
use t_trace::cli::{
    Cli, ClientArgs, ClientCommands, Commands, DaemonArgs, DaemonCommands, InitArgs,
};
use t_trace::{client, daemon, init};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let pid_file = "/tmp/t_trace.pid";

    if let Commands::Init(InitArgs { shell }) = cli.command {
        init::print_script(shell);
        return Ok(());
    }

    // special command does not require tokio runtime
    if let Commands::Daemon(DaemonArgs {
        command: DaemonCommands::Start,
    }) = cli.command
    {
        println!("Starting t_trace daemon in the background...");
        let daemonize = Daemonize::new().pid_file(pid_file).working_directory("/");

        match daemonize.start() {
            Ok(_) => {
                let _guard =
                    t_trace::logging::setup_daemon_logging().expect("Daemon logging setup failed");
                let daemon_rt = tokio::runtime::Runtime::new()
                    .expect("Failed to create Tokio runtime for daemon");
                if let Err(e) = daemon_rt.block_on(daemon::run()) {
                    tracing::error!("Daemon failed: {}", e);
                }
            }
            Err(e) => eprintln!("Error, could not start daemon: {}", e),
        }
        return Ok(());
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        match cli.command {
            Commands::Daemon(DaemonArgs { command }) => match command {
                DaemonCommands::Stop => {
                    client::run_daemon_stop().await?;
                }
                DaemonCommands::Status => client::run_status_check().await?,
                DaemonCommands::Start => unreachable!(),
            },
            Commands::Client(ClientArgs { command }) => match command {
                ClientCommands::Start { pid, command } => {
                    client::run_client_start(pid, command).await?
                }
                ClientCommands::End { pid, exit_code } => {
                    client::run_client_end(pid, exit_code).await?
                }
            },
            Commands::Stats => client::run_stats_display().await?,
            Commands::Init(_) => unreachable!(),
        }

        Ok::<(), anyhow::Error>(())
    })
}
