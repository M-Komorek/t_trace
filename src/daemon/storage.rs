use super::state::DaemonState;

use crate::dto::CommandStats;

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

fn get_stats_file_path() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find local data directory"))?
        .join("t_trace");
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join("stats.json"))
}

pub fn save_state(state: &DaemonState) -> Result<()> {
    let final_path = get_stats_file_path()?;
    let temp_path = final_path.with_extension("json.tmp");

    let temp_file = File::create(&temp_path)
        .with_context(|| format!("Failed to create temp file: {:?}", &temp_path))?;
    let writer = BufWriter::new(temp_file);
    serde_json::to_writer_pretty(writer, &state.aggregated_stats)
        .with_context(|| "Failed to serialize state to JSON")?;

    std::fs::rename(&temp_path, &final_path)
        .with_context(|| "Failed to rename temp file to final path")?;

    tracing::debug!("Successfully saved state to {:?}", &final_path);
    Ok(())
}

pub fn load_state() -> Result<HashMap<String, CommandStats>> {
    let path = get_stats_file_path()?;
    if !path.exists() {
        tracing::debug!(
            "No existing state file found at {:?}. Starting fresh.",
            &path
        );
        return Ok(HashMap::new());
    }

    let file =
        File::open(&path).with_context(|| format!("Failed to open state file: {:?}", &path))?;
    let reader = BufReader::new(file);
    let stats: HashMap<String, CommandStats> =
        serde_json::from_reader(reader).with_context(|| "Failed to deserialize state from JSON")?;

    tracing::debug!(
        "Successfully loaded {} records from state file.",
        stats.len()
    );
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_state() {
        let dir = tempdir().unwrap();
        let stats_path = dir.path().join("stats.json");

        let initial_stats = load_state_from_path(&stats_path).unwrap();
        assert!(initial_stats.is_empty());

        let mut state = DaemonState::default();
        state.handle_start(1, "cmd1".to_string());
        state.handle_end(1, 0);

        save_state_to_path(&state, &stats_path).unwrap();

        let loaded_stats = load_state_from_path(&stats_path).unwrap();
        assert_eq!(loaded_stats.len(), 1);
        assert!(loaded_stats.contains_key("cmd1"));
        assert_eq!(loaded_stats.get("cmd1").unwrap().count, 1);
    }

    fn save_state_to_path(state: &DaemonState, path: &Path) -> Result<()> {
        let temp_path = path.with_extension("json.tmp");
        let file = File::create(&temp_path)?;
        serde_json::to_writer_pretty(BufWriter::new(file), &state.aggregated_stats)?;
        std::fs::rename(&temp_path, path)?;
        Ok(())
    }

    fn load_state_from_path(path: &Path) -> Result<HashMap<String, CommandStats>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let file = File::open(path)?;
        Ok(serde_json::from_reader(BufReader::new(file))?)
    }
}
