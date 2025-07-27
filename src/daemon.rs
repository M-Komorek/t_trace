use crate::socket::get_socket_path;
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{debug, error, info, warn};

pub async fn run() -> Result<()> {
    let socket_path = get_socket_path()?;

    if socket_path.exists() {
        warn!("Removing existing socket at {:?}", &socket_path);
        std::fs::remove_file(&socket_path)
            .with_context(|| format!("Failed to remove existing socket at {:?}", &socket_path))?;
    }

    info!("Daemon listening on {:?}", &socket_path);
    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("Failed to bind to socket at {:?}", &socket_path))?;

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                tokio::spawn(async move {
                    debug!("Daemon received new connection");
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();

                    match reader.read_line(&mut line).await {
                        Ok(0) => {
                            debug!("Client disconnected");
                        }
                        Ok(_) => {
                            if line.trim() == "PING" {
                                debug!("Received PING, sending PONG");
                                if let Err(e) = stream.write_all(b"PONG\n").await {
                                    error!("Failed to write PONG to socket: {}", e);
                                }
                            } else {
                                warn!("Received unknown command: {}", line.trim());
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from socket: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}
