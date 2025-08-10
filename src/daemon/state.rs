use crate::dto::CommandStats;

use std::collections::HashMap;
use std::time::{Duration, Instant};

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

impl DaemonState {
    pub fn handle_start(&mut self, pid: u32, command_text: String) {
        let command = InFlightCommand {
            start_time: Instant::now(),
            command_text,
        };
        self.in_flight.insert(pid, command);
    }

    pub fn handle_end(&mut self, pid: u32, exit_code: i32) -> Option<Duration> {
        if let Some(in_flight_command) = self.in_flight.remove(&pid) {
            let duration = in_flight_command.start_time.elapsed();
            let is_success = exit_code == 0;

            let stats = self
                .aggregated_stats
                .entry(in_flight_command.command_text)
                .or_insert(CommandStats {
                    count: 0,
                    total_duration: Duration::from_secs(0),
                    last_run_duration: Duration::from_secs(0),
                    success_count: 0,
                    fail_count: 0,
                });

            stats.count += 1;
            stats.total_duration += duration;
            stats.last_run_duration = duration;

            if is_success {
                stats.success_count += 1;
            } else {
                stats.fail_count += 1;
            }

            Some(duration)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    mod unit_tests {
        use super::*;

        #[test]
        fn command_stats_serialization_deserialization() {
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

        #[test]
        fn handle_start_adds_command_to_in_flight() {
            let mut state = DaemonState::default();
            let pid = 1234;
            let cmd_text = "sleep 5".to_string();

            state.handle_start(pid, cmd_text.clone());

            assert_eq!(
                state.in_flight.len(),
                1,
                "Should have one in-flight command"
            );
            assert!(
                state.aggregated_stats.is_empty(),
                "Aggregated stats should not be touched"
            );

            let in_flight_cmd = state.in_flight.get(&pid).unwrap();
            assert_eq!(in_flight_cmd.command_text, cmd_text);
            assert!(in_flight_cmd.start_time.elapsed() < Duration::from_secs(1));
        }

        #[test]
        fn handle_end_does_nothing_for_unknown_pid() {
            let mut state = DaemonState::default();
            let unknown_pid = 999;

            let result = state.handle_end(unknown_pid, 0);

            assert!(result.is_none(), "Should return None for an unknown PID");
            assert!(
                state.in_flight.is_empty(),
                "In-flight map should remain empty"
            );
            assert!(
                state.aggregated_stats.is_empty(),
                "Aggregated stats should remain empty"
            );
        }

        #[test]
        fn handle_end_updates_stats_for_known_pid() {
            let mut state = DaemonState::default();
            let pid = 1234;
            let cmd_text = "echo 'hello'".to_string();

            state.in_flight.insert(
                pid,
                InFlightCommand {
                    start_time: Instant::now(),
                    command_text: cmd_text.clone(),
                },
            );

            std::thread::sleep(Duration::from_millis(10));
            let result = state.handle_end(pid, 0);

            assert!(result.is_some(), "Should return the duration");
            assert!(
                !state.aggregated_stats.is_empty(),
                "Stats should now be populated"
            );

            let stats = state.aggregated_stats.get(&cmd_text).unwrap();
            assert_eq!(stats.count, 1);
            assert_eq!(stats.success_count, 1);
            assert_eq!(stats.fail_count, 0);
            assert!(stats.total_duration >= Duration::from_millis(10));
        }
    }

    mod component_tests {
        use super::*;

        #[test]
        fn single_command_lifecycle() {
            let mut state = DaemonState::default();
            let cmd_text = "ls -l".to_string();
            let pid = 1234;

            state.handle_start(pid, cmd_text.clone());
            std::thread::sleep(Duration::from_millis(50));
            let duration_opt = state.handle_end(pid, 0);

            assert!(duration_opt.is_some());
            assert!(state.in_flight.is_empty());

            let stats = state.aggregated_stats.get(&cmd_text).unwrap();
            assert_eq!(stats.count, 1);
            assert_eq!(stats.success_count, 1);
        }

        #[test]
        fn multiple_commands_are_aggregated_correctly() {
            let mut state = DaemonState::default();
            let cmd_text = "git status".to_string();
            let pid1 = 1001;
            let pid2 = 1002;

            state.handle_start(pid1, cmd_text.clone());
            std::thread::sleep(Duration::from_millis(20));
            let duration1 = state.handle_end(pid1, 0).unwrap();

            state.handle_start(pid2, cmd_text.clone());
            std::thread::sleep(Duration::from_millis(30));
            let duration2 = state.handle_end(pid2, 1).unwrap();

            let stats = state.aggregated_stats.get(&cmd_text).unwrap();
            assert_eq!(stats.count, 2);
            assert_eq!(stats.success_count, 1);
            assert_eq!(stats.fail_count, 1);
            assert_eq!(stats.total_duration, duration1 + duration2);
            assert_eq!(stats.last_run_duration, duration2);
        }
    }
}
