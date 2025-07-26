use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CommandStats {
    pub count: u64,
    pub total_duration: Duration,
    pub last_run_duration: Duration,
    pub success_count: u64,
    pub fail_count: u64,
}

#[derive(Debug)]
pub struct InFlightCommand {
    pub start_time: Instant,
    pub command_text: String,
}

#[derive(Default, Debug)]
pub struct DaemonState {
    pub in_flight: HashMap<u32, InFlightCommand>,
    pub aggregated_stats: HashMap<String, CommandStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_command_stats_serialization_deserialization() {
        let original_stats = CommandStats {
            count: 5,
            total_duration: Duration::from_millis(1234),
            last_run_duration: Duration::from_millis(150),
            success_count: 4,
            fail_count: 1,
        };

        let json_string = serde_json::to_string(&original_stats).unwrap();
        let deserialized_stats: CommandStats = serde_json::from_str(&json_string).unwrap();
        assert_eq!(original_stats, deserialized_stats);
    }
}
