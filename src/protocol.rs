use anyhow::{Result, anyhow, bail};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Request {
    Stop,
    HealthCheck,
    CommandBegin { pid: u32, command: String },
    CommandEnd { pid: u32, exit_code: i32 },
    GetStats,
}

impl Request {
    pub fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();

        if s == "STOP" {
            return Ok(Request::Stop);
        }
        if s == "HEALTH_CHECK" {
            return Ok(Request::HealthCheck);
        }
        if s == "GET_STATS" {
            return Ok(Request::GetStats);
        }

        let mut parts = s.splitn(3, ' ');
        let verb = parts.next().ok_or_else(|| anyhow!("Missing verb"))?;
        let pid_str = parts.next().ok_or_else(|| anyhow!("Missing PID"))?;
        let pid = pid_str.parse::<u32>()?;

        match verb {
            "COMMAND_BEGIN" => {
                let command = parts
                    .next()
                    .ok_or_else(|| anyhow!("Missing command string"))?
                    .to_string();
                Ok(Request::CommandBegin { pid, command })
            }
            "COMMAND_END" => {
                let exit_code_str = parts.next().ok_or_else(|| anyhow!("Missing exit code"))?;
                let exit_code = exit_code_str.parse::<i32>()?;
                Ok(Request::CommandEnd { pid, exit_code })
            }
            _ => bail!("Unknown verb: {}", verb),
        }
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Request::Stop => write!(f, "STOP"),
            Request::HealthCheck => write!(f, "HEALTH_CHECK"),
            Request::CommandBegin { pid, command } => {
                write!(f, "COMMAND_BEGIN {} {}", pid, command)
            }
            Request::CommandEnd { pid, exit_code } => {
                write!(f, "COMMAND_END {} {}", pid, exit_code)
            }
            Request::GetStats => write!(f, "GET_STATS"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_begin_request_parsing() {
        let input = "COMMAND_BEGIN 1234 ls -l /home/user";
        let expected = Request::CommandBegin {
            pid: 1234,
            command: "ls -l /home/user".to_string(),
        };
        assert_eq!(Request::from_str(input).unwrap(), expected);
    }

    #[test]
    fn test_command_end_request_parsing() {
        let input = "COMMAND_END 5678 0";
        let expected = Request::CommandEnd {
            pid: 5678,
            exit_code: 0,
        };
        assert_eq!(Request::from_str(input).unwrap(), expected);
    }

    #[test]
    fn test_command_end_request_parsing_with_negative_exit_code() {
        let input = "COMMAND_END 5678 -1";
        let expected = Request::CommandEnd {
            pid: 5678,
            exit_code: -1,
        };
        assert_eq!(Request::from_str(input).unwrap(), expected);
    }

    #[test]
    fn test_request_formatting() {
        let begin_req = Request::CommandBegin {
            pid: 1234,
            command: "git commit -m \"a message\"".to_string(),
        };
        let end_req = Request::CommandEnd {
            pid: 5678,
            exit_code: 0,
        };

        let expected_begin = "COMMAND_BEGIN 1234 git commit -m \"a message\"";
        let expected_end = "COMMAND_END 5678 0";

        assert_eq!(begin_req.to_string(), expected_begin);
        assert_eq!(end_req.to_string(), expected_end);
    }

    #[test]
    fn test_parsing_fails_on_unknown_verb() {
        let input = "FOO 1234 bar";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_empty_input() {
        let input = "";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_missing_pid() {
        let input = "COMMAND_BEGIN";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_invalid_pid() {
        let input = "COMMAND_BEGIN not_a_pid ls";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_missing_command_string_for_start() {
        let input = "COMMAND_BEGIN 1234";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_missing_exit_code_for_end() {
        let input = "COMMAND_END 1234";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_invalid_exit_code() {
        let input = "COMMAND_END 1234 not_an_int";
        assert!(Request::from_str(input).is_err());
    }
}
