use crate::client::Client;

use anyhow::Result;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use std::time::Duration;

pub async fn handle_daemon_health_check() -> Result<()> {
    let mut client = Client::connect().await?;
    let response = client.send_health_check().await?;
    if response == "Daemon alive" {
        println!("Daemon is responsive.");
        Ok(())
    } else {
        anyhow::bail!("Daemon responded with an unexpected message: {}", response);
    }
}

pub async fn handle_daemon_command_begin(pid: u32, command: String) -> Result<()> {
    Client::connect()
        .await?
        .send_command_begin(pid, command)
        .await
}

pub async fn handle_daemon_command_end(pid: u32, exit_code: i32) -> Result<()> {
    Client::connect()
        .await?
        .send_end_command(pid, exit_code)
        .await
}

pub async fn handle_daemon_stop() -> Result<()> {
    Client::connect().await?.send_stop().await?;
    Ok(())
}

pub async fn handle_stats() -> Result<()> {
    let stats = Client::connect().await?.send_get_stats().await?;

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table
        .set_header(
            vec![
                "Command",
                "Success Count",
                "Fail Count",
                "Mean Time",
                "Total Time",
                "Last Time",
            ]
            .into_iter()
            .map(|h| Cell::new(h).add_attribute(Attribute::Bold)),
        )
        .set_content_arrangement(ContentArrangement::Dynamic);

    let mut sorted_stats: Vec<_> = stats.into_iter().collect();
    sorted_stats.sort_by(|a, b| b.1.total_duration.cmp(&a.1.total_duration));

    for (command, data) in sorted_stats {
        let mean_duration = if data.success_count > 0 || data.fail_count > 0 {
            data.total_duration.as_nanos() / (data.success_count as u128 + data.fail_count as u128)
        } else {
            0
        };

        table.add_row(vec![
            Cell::new(command).fg(Color::Yellow),
            Cell::new(data.success_count.to_string()).fg(Color::Green),
            Cell::new(data.fail_count.to_string()).fg(Color::Red),
            Cell::new(format!("{:.3?}", data.total_duration)),
            Cell::new(format!(
                "{:.3?}",
                Duration::from_nanos(mean_duration as u64)
            )),
            Cell::new(format!("{:.3?}", data.last_run_duration)),
        ]);
    }

    println!("{table}");
    Ok(())
}
