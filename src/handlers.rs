use crate::client::Client;
use crate::dto::CommandStats;

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

pub async fn handle_stats(filter: Option<String>) -> Result<()> {
    let all_stats = Client::connect().await?.send_get_stats().await?;

    if all_stats.is_empty() {
        println!("No commands tracked yet. Run a few commands and try again!");
        return Ok(());
    }

    let mut filtered_stats = filter_stats(all_stats.into_iter().collect(), &filter);
    if filtered_stats.is_empty() {
        if let Some(filter) = filter {
            println!("No commands found matching the filter: \"{}\"", filter);
        }
        return Ok(());
    }

    filtered_stats.sort_by(|first, second| first.1.total_duration.cmp(&second.1.total_duration));

    println!("{}", build_stats_table(filtered_stats));

    Ok(())
}

fn filter_stats(
    stats: Vec<(String, CommandStats)>,
    filter: &Option<String>,
) -> Vec<(String, CommandStats)> {
    let Some(filter) = filter else {
        return stats;
    };

    let filter_lowercase = filter.to_lowercase();
    stats
        .into_iter()
        .filter(|(command, _stats)| command.to_lowercase().contains(&filter_lowercase))
        .collect()
}

fn build_stats_table(stats_to_display: Vec<(String, CommandStats)>) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table
        .set_header(
            vec![
                "Command",
                "Success Count",
                "Fail Count",
                "Total Time",
                "Mean Time",
                "Last Time",
            ]
            .into_iter()
            .map(|h| Cell::new(h).add_attribute(Attribute::Bold)),
        )
        .set_content_arrangement(ContentArrangement::Dynamic);

    for (command, data) in stats_to_display {
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

    table.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn dummy_command_stats(count: u64) -> CommandStats {
        CommandStats {
            total_duration: Duration::from_secs(1),
            last_run_duration: Duration::from_secs(1),
            success_count: count,
            fail_count: count,
        }
    }

    #[test]
    fn filter_stats_with_no_filter() {
        let stats = vec![
            ("git status".to_string(), dummy_command_stats(10)),
            ("ls -l".to_string(), dummy_command_stats(5)),
        ];
        let filtered = filter_stats(stats, &None);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn filter_stats_with_matching_filter() {
        let stats = vec![
            ("git status".to_string(), dummy_command_stats(10)),
            ("git push".to_string(), dummy_command_stats(8)),
            ("ls -l".to_string(), dummy_command_stats(5)),
        ];
        let filtered = filter_stats(stats, &Some("git".to_string()));
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|(cmd, _)| cmd.starts_with("git")));
    }

    #[test]
    fn filter_stats_is_case_insensitive() {
        let stats = vec![
            ("git status".to_string(), dummy_command_stats(10)),
            ("ls -l".to_string(), dummy_command_stats(5)),
        ];
        let filtered = filter_stats(stats, &Some("GIT".to_string()));
        assert_eq!(filtered.len(), 1);
        assert!(filtered.iter().all(|(cmd, _)| cmd.starts_with("git")));
    }

    #[test]
    fn filter_stats_with_no_matches() {
        let stats = vec![
            ("git status".to_string(), dummy_command_stats(10)),
            ("ls -l".to_string(), dummy_command_stats(5)),
        ];
        let filtered = filter_stats(stats, &Some("cargo".to_string()));
        assert!(filtered.is_empty());
    }
}
