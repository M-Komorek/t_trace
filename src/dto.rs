use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CommandStats {
    pub count: u64,
    pub total_duration: Duration,
    pub last_run_duration: Duration,
    pub success_count: u64,
    pub fail_count: u64,
}
