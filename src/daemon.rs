use crate::protocol::Request;
use crate::socket::get_socket_path;
use crate::state::DaemonState;
use crate::storage;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

type SharedDaemonState = Arc<Mutex<DaemonState>>;

pub async fn run() -> Result<()> {
    let initial_stats = storage::load_state()?;
    let shared_daemon_state: SharedDaemonState = Arc::new(Mutex::new(DaemonState {
        in_flight: Default::default(),
        aggregated_stats: initial_stats, // <-- Initialize with loaded data
    }));

    let socket_path = get_socket_path()?;
    if socket_path.exists() {
        warn!("Removing existing socket at {:?}", &socket_path);
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("Failed to bind to socket at {:?}", &socket_path))?;

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    info!("Daemon listening on {:?}", &socket_path);

    loop {
        tokio::select! {
            Ok((stream, _addr)) = listener.accept() => {
                let state_clone = Arc::clone(&shared_daemon_state);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, state_clone).await {
                        error!("Error handling connection: {}", e);
                    }
                });
            },
            _ = sigint.recv() => { info!("SIGINT received, shutting down."); break; },
            _ = sigterm.recv() => { info!("SIGTERM received, shutting down."); break; },
        }
    }

    info!("Saving final state before exiting...");
    let final_state = shared_daemon_state.lock().await;
    if let Err(e) = storage::save_state(&final_state) {
        error!("Failed to save state during shutdown: {}", e);
    } else {
        info!("State saved successfully.");
    }

    let _ = std::fs::remove_file(&socket_path);
    info!("Daemon has shut down.");
    Ok(())
}

async fn handle_connection(mut stream: UnixStream, state: SharedDaemonState) -> Result<()> {
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();

    let bytes_read = reader
        .read_line(&mut line)
        .await
        .with_context(|| "Failed to read from stream")?;

    if bytes_read == 0 {
        debug!("Client disconnected without sending data.");
        return Ok(());
    }

    let response = process_request(&line, state).await;

    if let Some(resp_str) = response {
        stream
            .write_all(resp_str.as_bytes())
            .await
            .with_context(|| "Failed to write response to stream")?;
    }

    Ok(())
}

async fn process_request(line: &str, state: SharedDaemonState) -> Option<String> {
    match Request::from_str(line) {
        Ok(Request::Start { pid, command }) => {
            debug!("Received START for pid {}: {}", pid, &command);
            let mut state_guard = state.lock().await;
            state_guard.handle_start(pid, command);
            None
        }
        Ok(Request::End { pid, exit_code }) => {
            debug!("Received END for pid {} with code {}", pid, exit_code);
            let mut state_guard = state.lock().await;
            state_guard.handle_end(pid, exit_code);
            None
        }
        Ok(Request::GetStats) => {
            debug!("Received GET_STATS");
            let state_guard = state.lock().await;

            if let Err(e) = storage::save_state(&state_guard) {
                error!("Failed to save state on GET_STATS request: {}", e);
            }

            match serde_json::to_string(&state_guard.aggregated_stats) {
                Ok(json) => Some(json),
                Err(e) => {
                    error!("Failed to serialize stats to JSON: {}", e);
                    None
                }
            }
        }
        Ok(Request::Stop) => {
            info!("STOP command received. Shutting down now.");
            let state_guard = state.lock().await;
            if let Err(e) = storage::save_state(&state_guard) {
                error!("Failed to save state during shutdown: {}", e);
            }
            if let Ok(path) = get_socket_path() {
                let _ = std::fs::remove_file(path);
            }
            std::process::exit(0);
        }
        Err(_) => {
            if line.trim() == "PING" {
                debug!("Received PING, sending PONG");
                Some("PONG\n".to_string())
            } else {
                warn!("Failed to parse request: '{}'", line.trim());
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::DaemonState;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_process_request_ping() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let response = process_request("PING\n", state).await;

        assert_eq!(response, Some("PONG\n".to_string()));
    }

    #[tokio::test]
    async fn test_process_request_start() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let request_line = "START 1234 ls -l";
        let response = process_request(request_line, Arc::clone(&state)).await;

        assert_eq!(response, None);

        let state_guard = state.lock().await;
        assert_eq!(state_guard.in_flight.len(), 1);
        assert!(state_guard.in_flight.contains_key(&1234));
    }

    #[tokio::test]
    async fn test_process_request_end() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        {
            let mut state_guard = state.lock().await;
            state_guard.handle_start(1234, "ls -l".to_string());
        }

        let request_line = "END 1234 0";
        let response = process_request(request_line, Arc::clone(&state)).await;

        assert_eq!(response, None);

        let state_guard = state.lock().await;
        assert!(state_guard.in_flight.is_empty());
        assert_eq!(state_guard.aggregated_stats.len(), 1);
        assert!(state_guard.aggregated_stats.contains_key("ls -l"));
    }

    #[tokio::test]
    async fn test_process_request_invalid() {
        let state = Arc::new(Mutex::new(DaemonState::default()));
        let request_line = "GARBAGE a b c";
        let response = process_request(request_line, Arc::clone(&state)).await;

        assert_eq!(response, None);

        let state_guard = state.lock().await;
        assert!(state_guard.in_flight.is_empty());
        assert!(state_guard.aggregated_stats.is_empty());
    }
}
