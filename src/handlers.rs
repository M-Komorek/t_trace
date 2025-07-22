use crate::config::AppConfig;
use crate::model::{CommandRunInfo, StatsDB, update_stats_db};
use crate::utils::format_duration;

use comfy_table::{Cell, Table};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn handle_install() -> io::Result<()> {
    println!("✅ t_trace binary is installed!");
    println!("\nTo complete the setup, add the following line to your shell configuration file.");
    println!("For bash, this is usually ~/.bashrc\n");
    println!("--------------------------------------------------------------------------");
    println!(r#"eval "$(_T_TRACE_HOOK=1 t-trace)" # t-trace hook"#);
    println!("--------------------------------------------------------------------------\n");
    println!("After adding the line, please restart your shell or run `source ~/.bashrc`.");
    Ok(())
}

pub fn handle_start(app_config: &AppConfig, command: String) -> io::Result<()> {
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let command_run_info = CommandRunInfo {
        command,
        start_time_ms: start_time,
    };
    let file = File::create(&app_config.temp_path)?;
    serde_json::to_writer(file, &command_run_info)?;
    Ok(())
}

pub fn handle_end(app_config: &AppConfig) -> io::Result<()> {
    let temp_file = match File::open(&app_config.temp_path) {
        Ok(file) => file,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };

    let command_run_info: CommandRunInfo = serde_json::from_reader(temp_file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let end_time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    let duration_ms = end_time_ms.saturating_sub(command_run_info.start_time_ms);

    let mut stats_db: StatsDB = if app_config.stats_path.exists() {
        let file = File::open(&app_config.stats_path)?;
        serde_json::from_reader(BufReader::new(file)).unwrap_or_default()
    } else {
        StatsDB::new()
    };

    update_stats_db(&mut stats_db, command_run_info.command, duration_ms);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&app_config.stats_path)?;
    serde_json::to_writer_pretty(BufWriter::new(file), &stats_db)?;

    fs::remove_file(&app_config.temp_path)?;
    Ok(())
}

pub fn handle_stats(app_config: &AppConfig) -> io::Result<()> {
    if !app_config.stats_path.exists() {
        println!("No statistics recorded yet. Run some commands first!");
        return Ok(());
    }

    let file = File::open(&app_config.stats_path)?;
    let stats_db: StatsDB = serde_json::from_reader(BufReader::new(file))?;

    if stats_db.is_empty() {
        println!("No statistics recorded yet.");
        return Ok(());
    }

    let mut sorted_stats: Vec<_> = stats_db.into_iter().collect();
    sorted_stats.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.total_duration_ms));

    let mut table = Table::new();
    table.set_header(vec!["Command", "Count", "Total Time", "Mean Time"]);
    for (command, stats) in sorted_stats {
        if stats.count == 0 {
            continue;
        }
        let mean_time_ms = stats.total_duration_ms / stats.count as u128;
        table.add_row(vec![
            Cell::new(command),
            Cell::new(stats.count),
            Cell::new(format_duration(stats.total_duration_ms)),
            Cell::new(format_duration(mean_time_ms)),
        ]);
    }
    println!("{table}");
    Ok(())
}
