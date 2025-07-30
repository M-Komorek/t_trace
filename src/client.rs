use crate::protocol::Request;
use crate::socket::get_socket_path;
use crate::state::CommandStats;
use anyhow::{Context, Result};
use comfy_table::{ContentArrangement, Table};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
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
    let mut buf_reader = tokio::io::BufReader::new(reader);

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

async fn request_with_response(request: Request) -> Result<String> {
    let socket_path = get_socket_path()?;
    let stream = UnixStream::connect(&socket_path)
        .await
        .with_context(|| "Failed to connect to daemon. Is it running?")?;
    let (reader, mut writer) = stream.into_split();

    writer
        .write_all(format!("{}\n", request).as_bytes())
        .await?;

    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut response = String::new();
    buf_reader.read_to_string(&mut response).await?;
    Ok(response)
}

pub async fn run_stats_display() -> Result<()> {
    let json_response = request_with_response(Request::GetStats).await?;
    let stats: HashMap<String, CommandStats> = serde_json::from_str(&json_response)?;

    let mut table = Table::new();
    table
        .set_header(vec![
            "Command",
            "Count",
            "Mean Time",
            "Total Time",
            "Last Time",
        ])
        .set_content_arrangement(ContentArrangement::Dynamic);

    let mut sorted_stats: Vec<_> = stats.into_iter().collect();
    sorted_stats.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    for (command, data) in sorted_stats {
        let mean_duration = if data.count > 0 {
            data.total_duration.as_nanos() / data.count as u128
        } else {
            0
        };

        table.add_row(vec![
            command,
            data.count.to_string(),
            format!("{:.3?}", Duration::from_nanos(mean_duration as u64)),
            format!("{:.3?}", data.total_duration),
            format!("{:.3?}", data.last_run_duration),
        ]);
    }

    println!("{table}");
    Ok(())
}
