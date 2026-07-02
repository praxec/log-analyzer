//! `log-analyzer` — LLM-efficient structured access to text logs.
//!
//! # Modules
//! - [`parse`]   — tokenise raw log text into [`parse::LogEntry`] values.
//! - [`query`]   — filter and search over a slice of entries.
//! - [`summary`] — aggregate statistics (level counts, etc.).

pub mod parse;
pub mod query;
pub mod summary;

// Flatten the most-used items to the crate root for ergonomic use.
pub use parse::{Level, LogEntry, parse};
pub use query::{filter_by_level, grep};
pub use summary::level_counts;

// ---------------------------------------------------------------------------
// Integration-level smoke tests (lib.rs tests exercise the public API surface)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_LOG: &str = "\
2024-01-01T00:00:00Z ERROR disk full
2024-01-01T00:00:01Z WARN  memory pressure
2024-01-01T00:00:02Z INFO  server started
2024-01-01T00:00:03Z DEBUG entering request handler
2024-01-01T00:00:04Z INFO  request completed
";

    #[test]
    fn test_round_trip_parse_and_count() {
        let entries = parse(SAMPLE_LOG);
        assert_eq!(entries.len(), 5);

        let counts = level_counts(&entries);
        assert_eq!(counts["ERROR"], 1);
        assert_eq!(counts["WARN"], 1);
        assert_eq!(counts["INFO"], 2);
        assert_eq!(counts["DEBUG"], 1);
    }

    #[test]
    fn test_filter_by_level_via_lib_root() {
        let entries = parse(SAMPLE_LOG);
        let errors = filter_by_level(&entries, Level::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, Level::Error);
    }

    #[test]
    fn test_grep_via_lib_root() {
        let entries = parse(SAMPLE_LOG);
        let results = grep(&entries, "request");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_and_grep_composable() {
        let entries = parse(SAMPLE_LOG);
        // Narrow to Info-and-above, clone into an owned Vec, then grep.
        let important: Vec<LogEntry> = filter_by_level(&entries, Level::Info)
            .into_iter()
            .cloned()
            .collect();
        let hits = grep(&important, "request");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].message.contains("request completed"));
    }
}
