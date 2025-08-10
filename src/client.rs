use crate::dto::CommandStats;
use crate::protocol::Request;
use crate::socket;

use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

pub struct Client {
    stream: UnixStream,
}

impl Client {
    pub async fn connect() -> Result<Self> {
        let socket_path = socket::get_socket_path()?;
        let stream = UnixStream::connect(&socket_path)
            .await
            .with_context(|| "Failed to connect to daemon. Is it running?")?;
        Ok(Self { stream })
    }

    pub async fn send_command_begin(&mut self, pid: u32, command: String) -> Result<()> {
        let request = Request::CommandBegin { pid, command };
        self.send_fire_and_forget(request).await
    }

    pub async fn send_end_command(&mut self, pid: u32, exit_code: i32) -> Result<()> {
        let request = Request::CommandEnd { pid, exit_code };
        self.send_fire_and_forget(request).await
    }

    pub async fn send_stop(&mut self) -> Result<()> {
        self.send_fire_and_forget(Request::Stop).await
    }

    pub async fn send_get_stats(&mut self) -> Result<HashMap<String, CommandStats>> {
        let json_response = self.send_request_for_response(Request::GetStats).await?;
        let stats: HashMap<String, CommandStats> = serde_json::from_str(&json_response)?;
        Ok(stats)
    }

    pub async fn send_health_check(&mut self) -> Result<String> {
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

// TODO add UTs
