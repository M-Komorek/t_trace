use crate::protocol::Request;
use crate::socket::get_socket_path;
use crate::state::DaemonState;
use crate::storage;
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

type SharedDaemonState = Arc<Mutex<DaemonState>>;

#[derive(Debug, PartialEq)]
enum HandlerResult {
    Response(Option<String>),
    Shutdown,
}

pub struct Daemon {
    listener: UnixListener,
    shared_state: SharedDaemonState,
}

impl Daemon {
    pub async fn new() -> Result<Self> {
        let initial_stats = storage::load_state()?;
        let shared_state = Arc::new(Mutex::new(DaemonState {
            in_flight: Default::default(),
            aggregated_stats: initial_stats,
        }));

        let socket_path = get_socket_path()?;
        if socket_path.exists() {
            warn!("Removing existing socket at {:?}", &socket_path);
            let _ = std::fs::remove_file(&socket_path);
        }
        let listener = UnixListener::bind(&socket_path)?;

        Ok(Self {
            listener,
            shared_state,
        })
    }

    pub async fn run(self) -> Result<()> {
        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;

        info!("Daemon loop started. Awaiting connections or signals.");

        loop {
            tokio::select! {
                Ok((stream, _addr)) = self.listener.accept() => {
                    let state_clone = Arc::clone(&self.shared_state);
                    tokio::spawn(async move {
                         Self::handle_connection(stream, state_clone).await;
                    });
                },
                _ = sigint.recv() => { info!("SIGINT received, shutting down."); break; },
                _ = sigterm.recv() => { info!("SIGTERM received, shutting down."); break; },
            }
        }

        self.shutdown().await
    }

    async fn shutdown(self) -> Result<()> {
        info!("Saving final state before exiting...");
        let final_state = self.shared_state.lock().await;
        storage::save_state(&final_state)?;
        info!("State saved successfully.");

        if let Ok(path) = get_socket_path() {
            let _ = std::fs::remove_file(path);
        }
        info!("Daemon has shut down.");
        Ok(())
    }

    async fn handle_connection(mut stream: UnixStream, state: SharedDaemonState) {
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        if let Ok(bytes_read) = reader.read_line(&mut line).await {
            if bytes_read > 0 {
                if let HandlerResult::Shutdown = Self::process_request(&line, &state).await {
                    info!("Shutdown requested by client. Exiting now.");
                    let state_guard = state.lock().await;
                    if let Err(e) = storage::save_state(&state_guard) {
                        error!(
                            "Failed to save state during client-initiated shutdown: {}",
                            e
                        );
                    }
                    std::process::exit(0);
                } else if let HandlerResult::Response(Some(resp)) =
                    Self::process_request(&line, &state).await
                {
                    if let Err(e) = stream.write_all(resp.as_bytes()).await {
                        error!("Failed to write response: {}", e);
                    }
                }
            }
        }
    }

    async fn process_request(line: &str, state: &SharedDaemonState) -> HandlerResult {
        match Request::from_str(line) {
            Ok(Request::Start { pid, command }) => {
                state.lock().await.handle_start(pid, command);
                HandlerResult::Response(None)
            }
            Ok(Request::End { pid, exit_code }) => {
                state.lock().await.handle_end(pid, exit_code);
                HandlerResult::Response(None)
            }
            Ok(Request::GetStats) => {
                let state_guard = state.lock().await;
                if let Err(e) = storage::save_state(&state_guard) {
                    error!("Failed to save state on GET_STATS request: {}", e);
                }
                HandlerResult::Response(serde_json::to_string(&state_guard.aggregated_stats).ok())
            }
            Ok(Request::Stop) => {
                // THE CRITICAL CHANGE: Instead of exiting, we signal our intent.
                HandlerResult::Shutdown
            }
            Err(_) => {
                if line.trim() == "PING" {
                    HandlerResult::Response(Some("PONG\n".to_string()))
                } else {
                    warn!("Failed to parse request: '{}'", line.trim());
                    HandlerResult::Response(None)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::CommandStats;
    use std::collections::HashMap;

    fn setup_test_state() -> SharedDaemonState {
        Arc::new(Mutex::new(DaemonState::default()))
    }

    #[tokio::test]
    async fn process_request_ping_responds_with_pong() {
        let state = setup_test_state();
        let result = Daemon::process_request("PING\n", &state).await;
        assert_eq!(result, HandlerResult::Response(Some("PONG\n".to_string())));
    }

    #[tokio::test]
    async fn process_request_stop_signals_shutdown() {
        let state = setup_test_state();
        let result = Daemon::process_request("STOP\n", &state).await;
        assert_eq!(result, HandlerResult::Shutdown);
    }

    #[tokio::test]
    async fn process_request_start_modifies_state() {
        let state = setup_test_state();
        let result = Daemon::process_request("START 1234 ls -l", &state).await;
        assert_eq!(result, HandlerResult::Response(None));

        let state_guard = state.lock().await;
        assert_eq!(state_guard.in_flight.len(), 1);
        assert_eq!(
            state_guard.in_flight.get(&1234).unwrap().command_text,
            "ls -l"
        );
    }

    #[tokio::test]
    async fn process_request_end_modifies_state() {
        let state = setup_test_state();
        state
            .lock()
            .await
            .handle_start(5678, "git status".to_string());

        let result = Daemon::process_request("END 5678 0", &state).await;
        assert_eq!(result, HandlerResult::Response(None));

        let state_guard = state.lock().await;
        assert!(state_guard.in_flight.is_empty());
        assert_eq!(state_guard.aggregated_stats.len(), 1);
    }

    #[tokio::test]
    async fn process_request_get_stats_returns_json_response() {
        let state = setup_test_state();
        state.lock().await.aggregated_stats.insert(
            "cmd1".into(),
            CommandStats {
                count: 1,
                total_duration: std::time::Duration::from_secs(1),
                last_run_duration: std::time::Duration::from_secs(1),
                success_count: 1,
                fail_count: 0,
            },
        );

        let result = Daemon::process_request("GET_STATS", &state).await;

        match result {
            HandlerResult::Response(Some(json)) => {
                let stats: HashMap<String, CommandStats> = serde_json::from_str(&json).unwrap();
                assert_eq!(stats.get("cmd1").unwrap().count, 1);
            }
            _ => panic!("Expected a response with JSON data"),
        }
    }
}
