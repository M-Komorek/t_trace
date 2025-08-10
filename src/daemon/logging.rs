use anyhow::Result;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

pub fn setup_daemon_logging() -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let data_dir = dirs::data_local_dir()
        .expect("Could not find local data directory")
        .join("t_trace");
    std::fs::create_dir_all(&data_dir)?;

    let log_file_path = data_dir.join("daemon.log");
    let file_appender = tracing_appender::rolling::never(data_dir, "daemon.log");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(non_blocking_appender)
                .with_ansi(false),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set global default subscriber");

    tracing::info!(
        "Daemon logging configured to write to a single file: {:?}",
        log_file_path
    );

    Ok(guard)
}
