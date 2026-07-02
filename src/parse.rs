/// Log severity level, ordered from most-severe (Error=0) to least (Other=4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Other,
}

impl Level {
    /// Convert to a stable display string (used as BTreeMap key in summary).
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Other => "OTHER",
        }
    }
}

/// A single parsed log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub line_no: usize,
    pub level: Level,
    pub message: String,
}

/// Detect the severity level from a raw log line by scanning for the
/// first occurrence of a known keyword (case-insensitive).
fn detect_level(line: &str) -> Level {
    let upper = line.to_ascii_uppercase();
    // Check in priority order so "ERROR" beats "WARN" if both appear.
    if upper.contains("ERROR") {
        Level::Error
    } else if upper.contains("WARN") {
        Level::Warn
    } else if upper.contains("INFO") {
        Level::Info
    } else if upper.contains("DEBUG") {
        Level::Debug
    } else {
        Level::Other
    }
}

/// Parse a multi-line log string into a `Vec<LogEntry>`.
///
/// Each non-empty line becomes one entry. Blank lines are skipped so that
/// trailing newlines and paragraph separators do not produce phantom entries.
pub fn parse(src: &str) -> Vec<LogEntry> {
    src.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(idx, line)| LogEntry {
            line_no: idx + 1, // 1-based
            level: detect_level(line),
            message: line.to_owned(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_string() {
        assert!(parse("").is_empty());
    }

    #[test]
    fn test_parse_blank_lines_skipped() {
        let src = "\n\n\n";
        assert!(parse(src).is_empty());
    }

    #[test]
    fn test_parse_single_error_line() {
        let entries = parse("2024-01-01 ERROR something went wrong");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, Level::Error);
        assert_eq!(entries[0].line_no, 1);
    }

    #[test]
    fn test_parse_level_detection_case_insensitive() {
        let src = "error: boom\nwarn: slow\ninfo: ok\ndebug: trace\nunknown line";
        let entries = parse(src);
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].level, Level::Error);
        assert_eq!(entries[1].level, Level::Warn);
        assert_eq!(entries[2].level, Level::Info);
        assert_eq!(entries[3].level, Level::Debug);
        assert_eq!(entries[4].level, Level::Other);
    }

    #[test]
    fn test_parse_line_numbers_are_one_based() {
        let src = "INFO first\nDEBUG second\nERROR third";
        let entries = parse(src);
        assert_eq!(entries[0].line_no, 1);
        assert_eq!(entries[1].line_no, 2);
        assert_eq!(entries[2].line_no, 3);
    }

    #[test]
    fn test_parse_message_preserved() {
        let line = "2024-01-01T00:00:00Z INFO hello world";
        let entries = parse(line);
        assert_eq!(entries[0].message, line);
    }

    #[test]
    fn test_parse_mixed_case_keywords() {
        // "Error" (mixed-case) should still map to Level::Error
        let entries = parse("Error: disk full");
        assert_eq!(entries[0].level, Level::Error);

        let entries = parse("Warning: low mem");
        assert_eq!(entries[0].level, Level::Warn);
    }

    #[test]
    fn test_level_ordering() {
        // Error < Warn < Info < Debug < Other  (lower ordinal = higher severity)
        assert!(Level::Error < Level::Warn);
        assert!(Level::Warn < Level::Info);
        assert!(Level::Info < Level::Debug);
        assert!(Level::Debug < Level::Other);
    }

    #[test]
    fn test_level_as_str() {
        assert_eq!(Level::Error.as_str(), "ERROR");
        assert_eq!(Level::Warn.as_str(), "WARN");
        assert_eq!(Level::Info.as_str(), "INFO");
        assert_eq!(Level::Debug.as_str(), "DEBUG");
        assert_eq!(Level::Other.as_str(), "OTHER");
    }
}
