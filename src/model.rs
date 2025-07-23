use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandRunInfo {
    pub command: String,
    pub start_time_ms: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CommandStats {
    pub count: u64,
    pub total_duration_ms: u128,
    pub latest_duration_ms: u128,
}

pub type StatsDB = HashMap<String, CommandStats>;

pub fn update_stats_db(db: &mut StatsDB, command: String, duration_ms: u128) {
    let stats = db.entry(command).or_insert(CommandStats {
        count: 0,
        total_duration_ms: 0,
        latest_duration_ms: 0,
    });
    stats.count += 1;
    stats.total_duration_ms += duration_ms;
    stats.latest_duration_ms = duration_ms;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_new_command() {
        let mut db = StatsDB::new();
        let command = "ls -l".to_string();
        update_stats_db(&mut db, command.clone(), 150);
        let stats = db.get(&command).unwrap();
        assert_eq!(stats.count, 1);
        assert_eq!(stats.total_duration_ms, 150);
        assert_eq!(stats.latest_duration_ms, 150);
    }

    #[test]
    fn test_update_existing_command() {
        let mut db = StatsDB::new();
        let command = "git status".to_string();
        db.insert(
            command.clone(),
            CommandStats {
                count: 5,
                total_duration_ms: 1000,
                latest_duration_ms: 100,
            },
        );
        update_stats_db(&mut db, command.clone(), 250);
        let stats = db.get(&command).unwrap();
        assert_eq!(stats.count, 6);
        assert_eq!(stats.total_duration_ms, 1250);
        assert_eq!(stats.latest_duration_ms, 250);
    }

    #[test]
    fn test_serialization_and_deserialization() {
        let mut db = StatsDB::new();
        db.insert(
            "cmd1".to_string(),
            CommandStats {
                count: 1,
                total_duration_ms: 10,
                latest_duration_ms: 10,
            },
        );
        let serialized_json = serde_json::to_string(&db).unwrap();
        let deserialized_db: StatsDB = serde_json::from_str(&serialized_json).unwrap();
        assert_eq!(db, deserialized_db);
    }
}
