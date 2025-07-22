pub fn format_duration(ms: u128) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60 * 1000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else {
        let seconds = ms / 1000;
        format!("{}m {}s", seconds / 60, seconds % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_milliseconds() {
        assert_eq!(format_duration(0), "0ms");
        assert_eq!(format_duration(999), "999ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(1000), "1.00s");
        assert_eq!(format_duration(1520), "1.52s");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(60_000), "1m 0s");
        assert_eq!(format_duration(135_000), "2m 15s");
    }
}
