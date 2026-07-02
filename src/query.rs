use crate::parse::{Level, LogEntry};

/// Return all entries whose level is **at least as severe** as `min`.
///
/// Severity ordering: `Error` (most severe) < `Warn` < `Info` < `Debug` < `Other`.
/// Because `Level` derives `Ord` in that order, "at least as severe as `min`"
/// means `entry.level <= min`.
///
/// Examples
/// --------
/// `filter_by_level(entries, Level::Warn)` → entries with `Error` or `Warn`.
pub fn filter_by_level(entries: &[LogEntry], min: Level) -> Vec<&LogEntry> {
    entries.iter().filter(|e| e.level <= min).collect()
}

/// Return all entries whose message contains `needle` (case-sensitive substring match).
pub fn grep<'a>(entries: &'a [LogEntry], needle: &'_ str) -> Vec<&'a LogEntry> {
    entries
        .iter()
        .filter(|e| e.message.contains(needle))
        .collect()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    fn sample() -> Vec<LogEntry> {
        parse(
            "ERROR disk full\n\
             WARN  memory low\n\
             INFO  started\n\
             DEBUG entering loop\n\
             just a plain line",
        )
    }

    // --- filter_by_level ---

    #[test]
    fn test_filter_error_only() {
        let entries = sample();
        let result = filter_by_level(&entries, Level::Error);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].level, Level::Error);
    }

    #[test]
    fn test_filter_warn_includes_error() {
        let entries = sample();
        let result = filter_by_level(&entries, Level::Warn);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].level, Level::Error);
        assert_eq!(result[1].level, Level::Warn);
    }

    #[test]
    fn test_filter_info_includes_error_warn_info() {
        let entries = sample();
        let result = filter_by_level(&entries, Level::Info);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_filter_debug_includes_four() {
        let entries = sample();
        let result = filter_by_level(&entries, Level::Debug);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_filter_other_includes_all() {
        let entries = sample();
        let result = filter_by_level(&entries, Level::Other);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_filter_empty_input() {
        let result = filter_by_level(&[], Level::Error);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_returns_references_to_original() {
        let entries = sample();
        let result = filter_by_level(&entries, Level::Error);
        // Pointer equality: the returned ref must point into `entries`.
        assert!(std::ptr::eq(result[0], &entries[0]));
    }

    // --- grep ---

    #[test]
    fn test_grep_finds_substring() {
        let entries = sample();
        let result = grep(&entries, "disk");
        assert_eq!(result.len(), 1);
        assert!(result[0].message.contains("disk"));
    }

    #[test]
    fn test_grep_no_match() {
        let entries = sample();
        let result = grep(&entries, "banana");
        assert!(result.is_empty());
    }

    #[test]
    fn test_grep_multiple_matches() {
        let entries = parse("INFO alpha\nINFO beta\nERROR gamma");
        let result = grep(&entries, "INFO");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_grep_case_sensitive() {
        let entries = parse("INFO hello");
        // "info" (lowercase) should NOT match "INFO"
        assert!(grep(&entries, "info").is_empty());
        assert_eq!(grep(&entries, "INFO").len(), 1);
    }

    #[test]
    fn test_grep_empty_needle_matches_all() {
        let entries = sample();
        // An empty string is a substring of every string.
        let result = grep(&entries, "");
        assert_eq!(result.len(), entries.len());
    }

    #[test]
    fn test_grep_empty_input() {
        let result = grep(&[], "anything");
        assert!(result.is_empty());
    }
}
