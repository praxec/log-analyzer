use std::collections::BTreeMap;

use crate::parse::LogEntry;

/// Summarise `entries` as a count per level label.
///
/// Returns a `BTreeMap<String, usize>` keyed by the canonical level string
/// (`"ERROR"`, `"WARN"`, `"INFO"`, `"DEBUG"`, `"OTHER"`).  Only levels that
/// actually appear in `entries` are included; levels with zero occurrences are
/// omitted so the map stays compact.
///
/// This is intentionally a *projection* — a small, LLM-efficient summary of
/// an arbitrarily large log rather than the full entry list.
pub fn level_counts(entries: &[LogEntry]) -> BTreeMap<String, usize> {
    let mut map: BTreeMap<String, usize> = BTreeMap::new();
    for entry in entries {
        *map.entry(entry.level.as_str().to_owned()).or_insert(0) += 1;
    }
    map
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[test]
    fn test_level_counts_empty() {
        let map = level_counts(&[]);
        assert!(map.is_empty());
    }

    #[test]
    fn test_level_counts_single_error() {
        let entries = parse("ERROR boom");
        let map = level_counts(&entries);
        assert_eq!(map.get("ERROR"), Some(&1));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_level_counts_mixed() {
        let src = "ERROR a\nERROR b\nWARN c\nINFO d\nDEBUG e\nunknown";
        let entries = parse(src);
        let map = level_counts(&entries);
        assert_eq!(map["ERROR"], 2);
        assert_eq!(map["WARN"], 1);
        assert_eq!(map["INFO"], 1);
        assert_eq!(map["DEBUG"], 1);
        assert_eq!(map["OTHER"], 1);
    }

    #[test]
    fn test_level_counts_only_present_levels_appear() {
        let entries = parse("INFO hello\nINFO world");
        let map = level_counts(&entries);
        assert_eq!(map.len(), 1);
        assert_eq!(map["INFO"], 2);
        assert!(!map.contains_key("ERROR"));
    }

    #[test]
    fn test_level_counts_keys_are_strings() {
        // Verify the BTreeMap<String, usize> contract — keys are owned Strings.
        let entries = parse("WARN slow");
        let map = level_counts(&entries);
        let key: &str = map.keys().next().unwrap();
        assert_eq!(key, "WARN");
    }

    #[test]
    fn test_level_counts_btreemap_is_sorted() {
        // BTreeMap keys are alphabetically sorted: DEBUG, ERROR, INFO, OTHER, WARN
        let src = "WARN a\nERROR b\nDEBUG c\nINFO d\nunknown";
        let entries = parse(src);
        let map = level_counts(&entries);
        let keys: Vec<&str> = map.keys().map(String::as_str).collect();
        let mut expected = keys.clone();
        expected.sort();
        assert_eq!(keys, expected);
    }
}
