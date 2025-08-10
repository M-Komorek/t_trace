use super::logging;
use super::state::DaemonState;
use super::storage;

use crate::protocol::Request;
use crate::socket::get_socket_path;

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

pub async fn run() -> Result<()> {
    let _guard = logging::setup_daemon_logging().expect("Daemon logging setup failed");

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

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    info!("Daemon loop started. Awaiting connections or signals.");

    loop {
        tokio::select! {
            Ok((stream, _addr)) = listener.accept() => {
                let state_clone = Arc::clone(&shared_state);
                tokio::spawn(handle_connection(stream, state_clone));
            },
            _ = sigint.recv() => { info!("SIGINT received, breaking loop."); break; },
            _ = sigterm.recv() => { info!("SIGTERM received, breaking loop."); break; },
        }
    }

    shutdown_gracefully(shared_state).await;

    Ok(())
}

async fn shutdown_gracefully(state: SharedDaemonState) {
    info!("Shutting down gracefully. Saving final state...");
    let final_state = state.lock().await;

    if let Err(e) = storage::save_state(&final_state) {
        error!("Failed to save state during shutdown: {}", e);
    } else {
        info!("State saved successfully.");
    }

    if let Ok(path) = get_socket_path() {
        let _ = std::fs::remove_file(path);
    }
    info!("Daemon has shut down.");
}

async fn handle_connection(mut stream: UnixStream, state: SharedDaemonState) {
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();

    if let Ok(bytes_read) = reader.read_line(&mut line).await {
        if bytes_read == 0 {
            return;
        }

        match process_request(&line, &state).await {
            HandlerResult::Shutdown => {
                shutdown_gracefully(state).await;
                std::process::exit(0);
            }
            HandlerResult::Response(Some(response)) => {
                if let Err(e) = stream.write_all(response.as_bytes()).await {
                    error!("Failed to write response: {}", e);
                }
            }
            HandlerResult::Response(None) => {}
        }
    }
}

async fn process_request(line: &str, state: &SharedDaemonState) -> HandlerResult {
    match Request::from_str(line) {
        Ok(Request::HealthCheck) => HandlerResult::Response(None),
        Ok(Request::CommandBegin { pid, command }) => {
            state.lock().await.handle_start(pid, command);
            HandlerResult::Response(None)
        }
        Ok(Request::CommandEnd { pid, exit_code }) => {
            state.lock().await.handle_end(pid, exit_code);
            HandlerResult::Response(None)
        }
        Ok(Request::GetStats) => {
            let state_guard = state.lock().await;
            if let Err(e) = storage::save_state(&state_guard) {
                error!("Failed to save state on GET_STATS request: {}", e);
            }
            let response = serde_json::to_string(&state_guard.aggregated_stats).ok();
            HandlerResult::Response(response)
        }
        Ok(Request::Stop) => HandlerResult::Shutdown,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::CommandStats;
    use std::collections::HashMap;
    use std::time::Duration;

    fn setup_test_state() -> SharedDaemonState {
        Arc::new(Mutex::new(DaemonState::default()))
    }

    #[tokio::test]
    async fn process_request_ping_responds_with_pong() {
        let state = setup_test_state();
        let result = process_request("PING\n", &state).await;
        assert_eq!(result, HandlerResult::Response(Some("PONG\n".to_string())));
    }

    #[tokio::test]
    async fn process_request_stop_signals_shutdown() {
        let state = setup_test_state();
        let result = process_request("STOP\n", &state).await;
        assert_eq!(result, HandlerResult::Shutdown);
    }

    #[tokio::test]
    async fn process_request_start_modifies_state() {
        let state = setup_test_state();
        let request_line = "COMMAND_BEGIN 1234 ls -l";
        let result = process_request(request_line, &state).await;

        assert_eq!(result, HandlerResult::Response(None));
        let state_guard = state.lock().await;
        assert_eq!(state_guard.in_flight.len(), 1);
        let cmd = state_guard.in_flight.get(&1234).unwrap();
        assert_eq!(cmd.command_text, "ls -l");
    }

    #[tokio::test]
    async fn process_request_end_moves_command_to_aggregated() {
        let state = setup_test_state();
        let cmd_text = "git status".to_string();
        state.lock().await.handle_start(5678, cmd_text.clone());
        let result = process_request("COMMAND_END 5678 0", &state).await;

        assert_eq!(result, HandlerResult::Response(None));
        let state_guard = state.lock().await;
        assert!(state_guard.in_flight.is_empty());
        assert_eq!(state_guard.aggregated_stats.len(), 1);
        assert!(state_guard.aggregated_stats.contains_key(&cmd_text));
    }

    #[tokio::test]
    async fn process_request_get_stats_returns_json_response() {
        let state = setup_test_state();
        {
            let mut state_guard = state.lock().await;
            state_guard.aggregated_stats.insert(
                "cmd1".to_string(),
                CommandStats {
                    total_duration: Duration::from_secs(10),
                    last_run_duration: Duration::from_secs(2),
                    success_count: 5,
                    fail_count: 0,
                },
            );
        }

        let result = process_request("GET_STATS", &state).await;

        match result {
            HandlerResult::Response(Some(json)) => {
                let stats: HashMap<String, CommandStats> =
                    serde_json::from_str(&json).expect("Response should be valid JSON");
                assert_eq!(stats.len(), 1);
                assert_eq!(stats.get("cmd1").unwrap().success_count, 5);
            }
            _ => panic!("Expected a response with JSON data"),
        }
    }

    #[tokio::test]
    async fn process_request_get_stats_on_empty_state_is_ok() {
        let state = setup_test_state();
        let result = process_request("GET_STATS", &state).await;

        match result {
            HandlerResult::Response(Some(json)) => {
                let stats: HashMap<String, CommandStats> = serde_json::from_str(&json).unwrap();
                assert!(stats.is_empty());
            }
            _ => panic!("Expected a response with empty JSON data"),
        }
    }

    #[tokio::test]
    async fn process_request_invalid_input_is_handled_gracefully() {
        let state = setup_test_state();
        let result = process_request("GARBAGE_COMMAND_DOES_NOT_EXIST", &state).await;

        assert_eq!(result, HandlerResult::Response(None));
        let state_guard = state.lock().await;
        assert!(state_guard.in_flight.is_empty());
        assert!(state_guard.aggregated_stats.is_empty());
    }
}
