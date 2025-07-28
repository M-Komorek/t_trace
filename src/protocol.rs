use anyhow::{Result, anyhow, bail};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Request {
    Start { pid: u32, command: String },
    End { pid: u32, exit_code: i32 },
}

impl Request {
    pub fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        let mut parts = s.splitn(3, ' ');

        let verb = parts.next().ok_or_else(|| anyhow!("Missing verb"))?;
        let pid_str = parts.next().ok_or_else(|| anyhow!("Missing PID"))?;
        let pid = pid_str.parse::<u32>()?;

        match verb {
            "START" => {
                let command = parts
                    .next()
                    .ok_or_else(|| anyhow!("Missing command string"))?
                    .to_string();
                Ok(Request::Start { pid, command })
            }
            "END" => {
                let exit_code_str = parts.next().ok_or_else(|| anyhow!("Missing exit code"))?;
                let exit_code = exit_code_str.parse::<i32>()?;
                Ok(Request::End { pid, exit_code })
            }
            _ => bail!("Unknown verb: {}", verb),
        }
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Request::Start { pid, command } => write!(f, "START {} {}", pid, command),
            Request::End { pid, exit_code } => write!(f, "END {} {}", pid, exit_code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_request_parsing() {
        let input = "START 1234 ls -l /home/user";
        let expected = Request::Start {
            pid: 1234,
            command: "ls -l /home/user".to_string(),
        };
        assert_eq!(Request::from_str(input).unwrap(), expected);
    }

    #[test]
    fn test_end_request_parsing() {
        let input = "END 5678 0";
        let expected = Request::End {
            pid: 5678,
            exit_code: 0,
        };
        assert_eq!(Request::from_str(input).unwrap(), expected);
    }

    #[test]
    fn test_end_request_parsing_with_negative_exit_code() {
        let input = "END 5678 -1";
        let expected = Request::End {
            pid: 5678,
            exit_code: -1,
        };
        assert_eq!(Request::from_str(input).unwrap(), expected);
    }

    #[test]
    fn test_request_formatting() {
        let start_req = Request::Start {
            pid: 1234,
            command: "git commit -m \"a message\"".to_string(),
        };
        let end_req = Request::End {
            pid: 5678,
            exit_code: 0,
        };

        let expected_start = "START 1234 git commit -m \"a message\"";
        let expected_end = "END 5678 0";

        assert_eq!(start_req.to_string(), expected_start);
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
        let input = "START";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_invalid_pid() {
        let input = "START not_a_pid ls";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_missing_command_string_for_start() {
        let input = "START 1234";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_missing_exit_code_for_end() {
        let input = "END 1234";
        assert!(Request::from_str(input).is_err());
    }

    #[test]
    fn test_parsing_fails_on_invalid_exit_code() {
        let input = "END 1234 not_an_int";
        assert!(Request::from_str(input).is_err());
    }
}
