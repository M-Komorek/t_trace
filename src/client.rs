use crate::protocol::Request;
use crate::socket::get_socket_path;
use crate::state::CommandStats;
use anyhow::{Context, Result};
use comfy_table::{ContentArrangement, Table};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

pub struct Client {
    stream: UnixStream,
}

impl Client {
    pub async fn connect() -> Result<Self> {
        let socket_path = get_socket_path()?;
        let stream = UnixStream::connect(&socket_path)
            .await
            .with_context(|| "Failed to connect to daemon. Is it running?")?;
        Ok(Self { stream })
    }

    pub async fn start_command(&mut self, pid: u32, command: String) -> Result<()> {
        let request = Request::Start { pid, command };
        self.send_fire_and_forget(request).await
    }

    pub async fn end_command(&mut self, pid: u32, exit_code: i32) -> Result<()> {
        let request = Request::End { pid, exit_code };
        self.send_fire_and_forget(request).await
    }

    pub async fn send_stop_signal(&mut self) -> Result<()> {
        self.send_fire_and_forget(Request::Stop).await
    }

    pub async fn get_stats(&mut self) -> Result<HashMap<String, CommandStats>> {
        let json_response = self.send_request_for_response(Request::GetStats).await?;
        let stats: HashMap<String, CommandStats> = serde_json::from_str(&json_response)?;
        Ok(stats)
    }

    pub async fn check_status(&mut self) -> Result<String> {
        self.stream.write_all(b"PING\n").await?;
        let mut reader = BufReader::new(&mut self.stream);
        let mut response = String::new();
        reader.read_line(&mut response).await?;
        Ok(response.trim().to_string())
    }

    async fn send_fire_and_forget(&mut self, request: Request) -> Result<()> {
        self.stream
            .write_all(format!("{}\n", request).as_bytes())
            .await?;
        Ok(())
    }

    async fn send_request_for_response(&mut self, request: Request) -> Result<String> {
        let (reader, mut writer) = self.stream.split();

        writer
            .write_all(format!("{}\n", request).as_bytes())
            .await?;

        writer.flush().await?;

        let mut buf_reader = BufReader::new(reader);
        let mut response = String::new();
        buf_reader.read_to_string(&mut response).await?;
        Ok(response)
    }
}

pub async fn run_client_start(pid: u32, command: String) -> Result<()> {
    Client::connect().await?.start_command(pid, command).await
}

pub async fn run_client_end(pid: u32, exit_code: i32) -> Result<()> {
    Client::connect().await?.end_command(pid, exit_code).await
}

pub async fn run_stats_display() -> Result<()> {
    let stats = Client::connect().await?.get_stats().await?;

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

pub async fn run_status_check() -> Result<()> {
    let mut client = Client::connect().await?;
    let response = client.check_status().await?;
    if response == "PONG" {
        println!("Daemon is responsive.");
        Ok(())
    } else {
        anyhow::bail!("Daemon responded with an unexpected message: {}", response);
    }
}

pub async fn is_daemon_running() -> bool {
    if let Ok(mut client) = Client::connect().await {
        if let Ok(response) = client.check_status().await {
            return response == "PONG";
        }
    }
    false
}

pub async fn run_daemon_stop() -> Result<()> {
    Client::connect().await?.send_stop_signal().await?;
    Ok(())
}
