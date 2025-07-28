use crate::protocol::Request;
use crate::socket::get_socket_path;
use crate::state::DaemonState;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

type SharedDaemonState = Arc<Mutex<DaemonState>>;

pub async fn run() -> Result<()> {
    let shared_daemon_state: SharedDaemonState = Arc::new(Mutex::new(DaemonState::default()));

    let socket_path = get_socket_path()?;
    if socket_path.exists() {
        warn!("Removing existing socket at {:?}", &socket_path);
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("Failed to bind to socket at {:?}", &socket_path))?;

    info!("Daemon listening on {:?}", &socket_path);

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let shared_daemon_state_clone = Arc::clone(&shared_daemon_state);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, shared_daemon_state_clone).await {
                        error!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
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
