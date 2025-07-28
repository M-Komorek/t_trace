use crate::protocol::Request; // <-- Import our protocol
use crate::socket::get_socket_path;
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

async fn send_request(request: Request) -> Result<()> {
    let socket_path = get_socket_path()?;
    let mut stream = UnixStream::connect(&socket_path)
        .await
        .with_context(|| "Failed to connect to daemon. Is it running?")?;

    stream
        .write_all(format!("{}\n", request).as_bytes())
        .await?;
    Ok(())
}

pub async fn run_client_start(pid: u32, command: String) -> Result<()> {
    let request = Request::Start { pid, command };
    send_request(request).await
}

pub async fn run_client_end(pid: u32, exit_code: i32) -> Result<()> {
    let request = Request::End { pid, exit_code };
    send_request(request).await
}

pub async fn run_status_check() -> Result<()> {
    let socket_path = get_socket_path()?;
    let stream = UnixStream::connect(&socket_path)
        .await
        .with_context(|| "Daemon is not running or socket is inaccessible.")?;

    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    writer.write_all(b"PING\n").await?;

    let mut response = String::new();
    buf_reader.read_line(&mut response).await?;

    if response.trim() == "PONG" {
        println!("Daemon is responsive.");
        Ok(())
    } else {
        anyhow::bail!(
            "Daemon responded with an unexpected message: {}",
            response.trim()
        );
    }
}
