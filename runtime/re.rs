// ============================================================
// Homun Runtime — re.rs: Regex Pattern Matching + Caching
// Part B2 — REQUIRES external crate: regex = "1"
//
// DEPENDENCY NOTE: Projects that `use re` must add to Cargo.toml:
//   [dependencies]
//   regex = "1"
//
// Usage in .hom:
//   use re
//
//   matched, text, end := re_match("[a-zA-Z_][a-zA-Z0-9_]*", src, pos)
//   if (matched) do {
//     print("found: ${text} ending at ${end}")
//   }
//
//   ok := re_is_match("[0-9]+", "hello 42 world")
//
// API:
//   re_match(pattern, text, pos) -> (bool, String, usize)
//     Anchored match at byte offset `pos`.  Returns (matched, matched_str, end_pos).
//     Equivalent to Python's re.compile(pattern).match(text, pos).
//
//   re_is_match(pattern, text) -> bool
//     True if pattern matches anywhere in text.
//     Equivalent to Python's re.search(pattern, text) is not None.
//
// Caching:
//   Patterns are compiled once per thread and cached in a thread-local
//   HashMap<String, Regex>.  Subsequent calls with the same pattern string
//   reuse the compiled Regex object.
// ============================================================

use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static REGEX_CACHE: RefCell<HashMap<String, Regex>> = RefCell::new(HashMap::new());
}

/// Get or compile a Regex for `pattern`, using the thread-local cache.
/// Panics with a clear message if the pattern is invalid.
fn get_or_compile(pattern: &str) -> Regex {
    REGEX_CACHE.with(|cache| {
        let mut map = cache.borrow_mut();
        if !map.contains_key(pattern) {
            let re = Regex::new(pattern)
                .unwrap_or_else(|e| panic!("re: invalid regex pattern {:?}: {}", pattern, e));
            map.insert(pattern.to_string(), re);
        }
        // Regex is cheap to clone (Arc-backed)
        map.get(pattern).unwrap().clone()
    })
}

/// Match `pattern` anchored at byte offset `pos` in `text`.
///
/// Returns `(matched, captured_text, end_pos)` where:
/// - `matched`        — true if the pattern matches starting exactly at `pos`
/// - `captured_text`  — the matched substring (empty string when no match)
/// - `end_pos`        — byte offset just after the match (`pos` when no match)
///
/// Equivalent to Python's `re.compile(pattern).match(text, pos)`.
pub fn re_match(pattern: &str, text: &str, pos: usize) -> (bool, String, usize) {
    if pos > text.len() {
        return (false, String::new(), pos);
    }
    let re = get_or_compile(pattern);
    let haystack = &text[pos..];
    match re.find(haystack) {
        Some(m) if m.start() == 0 => {
            let matched_str = m.as_str().to_string();
            let end = pos + m.end();
            (true, matched_str, end)
        }
        _ => (false, String::new(), pos),
    }
}

/// Return `true` if `pattern` matches anywhere in `text`.
///
/// Equivalent to Python's `re.search(pattern, text) is not None`.
pub fn re_is_match(pattern: &str, text: &str) -> bool {
    let re = get_or_compile(pattern);
    re.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── re_is_match ──────────────────────────────────────────
    #[test]
    fn test_re_is_match_digit_present() {
        assert!(re_is_match("[0-9]+", "hello 42 world"));
    }

    #[test]
    fn test_re_is_match_digit_absent() {
        assert!(!re_is_match("[0-9]+", "no digits here"));
    }

    #[test]
    fn test_re_is_match_identifier() {
        assert!(re_is_match("[a-zA-Z_][a-zA-Z0-9_]*", "foo_bar"));
        assert!(!re_is_match("[a-zA-Z_][a-zA-Z0-9_]*", "123"));
    }

    #[test]
    fn test_re_is_match_empty_text() {
        assert!(!re_is_match("[a-z]+", ""));
    }

    #[test]
    fn test_re_is_match_full_string() {
        assert!(re_is_match("^hello$", "hello"));
        assert!(!re_is_match("^hello$", "hello world"));
    }

    // ── re_match — anchoring at pos ──────────────────────────
    #[test]
    fn test_re_match_at_start() {
        let (matched, text, end) = re_match("[a-zA-Z_][a-zA-Z0-9_]*", "hello world", 0);
        assert!(matched);
        assert_eq!(text, "hello");
        assert_eq!(end, 5);
    }

    #[test]
    fn test_re_match_at_offset() {
        // "world" starts at byte 6 in "hello world"
        let (matched, text, end) = re_match("[a-zA-Z_][a-zA-Z0-9_]*", "hello world", 6);
        assert!(matched);
        assert_eq!(text, "world");
        assert_eq!(end, 11);
    }

    #[test]
    fn test_re_match_anchored_mid_word() {
        // pos=3 is mid-word in "hello"; pattern matches "lo" starting there
        let (matched, text, end) = re_match("[a-zA-Z_]+", "hello world", 3);
        assert!(matched);
        assert_eq!(text, "lo");
        assert_eq!(end, 5);
    }

    #[test]
    fn test_re_match_no_match_at_pos() {
        // No digit at pos=0 of "hello 42"
        let (matched, text, end) = re_match("[0-9]+", "hello 42", 0);
        assert!(!matched);
        assert_eq!(text, "");
        assert_eq!(end, 0);
    }

    #[test]
    fn test_re_match_digits_at_offset() {
        let (matched, text, end) = re_match("[0-9]+", "abc 123 def", 4);
        assert!(matched);
        assert_eq!(text, "123");
        assert_eq!(end, 7);
    }

    #[test]
    fn test_re_match_empty_text() {
        let (matched, _, _) = re_match("[a-z]+", "", 0);
        assert!(!matched);
    }

    #[test]
    fn test_re_match_pos_at_end() {
        let text = "hello";
        let (matched, _, _) = re_match("[a-z]+", text, text.len());
        assert!(!matched);
    }

    #[test]
    fn test_re_match_pos_beyond_end() {
        let text = "hello";
        let (matched, _, ret_pos) = re_match("[a-z]+", text, text.len() + 1);
        assert!(!matched);
        assert_eq!(ret_pos, text.len() + 1);
    }

    // ── caching: same pattern compiled only once ─────────────
    #[test]
    fn test_re_match_cached_repeated_calls() {
        let pattern = "[a-z]+";
        let text = "foobar";
        for _ in 0..10 {
            let (matched, m, end) = re_match(pattern, text, 0);
            assert!(matched);
            assert_eq!(m, "foobar");
            assert_eq!(end, 6);
        }
    }

    #[test]
    fn test_re_is_match_cached_repeated_calls() {
        let pattern = r"\d+";
        for _ in 0..10 {
            assert!(re_is_match(pattern, "x99y"));
        }
    }

    // ── mermaid-parser style patterns ────────────────────────
    #[test]
    fn test_re_match_node_id_pattern() {
        let pattern = "[a-zA-Z_][a-zA-Z0-9_]*";
        let (matched, m, end) = re_match(pattern, "node_A123 rest", 0);
        assert!(matched);
        assert_eq!(m, "node_A123");
        assert_eq!(end, 9);
    }

    #[test]
    fn test_re_match_arrow_pattern() {
        let (matched, m, end) = re_match("-->", "--> next", 0);
        assert!(matched);
        assert_eq!(m, "-->");
        assert_eq!(end, 3);
    }

    #[test]
    fn test_re_match_quoted_string() {
        let pattern = r#""[^"]*""#;
        let (matched, m, end) = re_match(pattern, r#""hello" world"#, 0);
        assert!(matched);
        assert_eq!(m, "\"hello\"");
        assert_eq!(end, 7);
    }

    #[test]
    fn test_re_match_whitespace_skip() {
        let pattern = r"[ \t]+";
        let (matched, m, end) = re_match(pattern, "  \tabc", 0);
        assert!(matched);
        assert_eq!(m, "  \t");
        assert_eq!(end, 3);
    }

    #[test]
    fn test_re_match_multiple_patterns_cached() {
        // Exercise cache with several different patterns
        let text = "flowchart LR";
        let (m1, t1, _) = re_match("[a-z]+", text, 0);
        let (m2, t2, _) = re_match("[A-Z]+", text, 10);
        assert!(m1);
        assert_eq!(t1, "flowchart");
        assert!(m2);
        assert_eq!(t2, "LR");
    }
}
