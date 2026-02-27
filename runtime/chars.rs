// ============================================================
// Homun Runtime — chars.rs: Character Classification
// Part B3 — stdlib, no external crates required.
//
// Usage in .hom:
//   use chars
//   is_alpha("a")   // true
//   is_alnum("3")   // true
//   is_digit("7")   // true
//   is_ws(" ")      // true
//
// Functions accept a &str (typically a single-character string)
// and return true if ALL characters in the string satisfy the
// predicate (empty string → true, matching Rust's Iterator::all).
// ============================================================

/// True if every character in `s` is alphabetic (Unicode).
pub fn is_alpha(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphabetic())
}

/// True if every character in `s` is alphanumeric (Unicode).
pub fn is_alnum(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric())
}

/// True if every character in `s` is an ASCII decimal digit (0–9).
pub fn is_digit(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// True if every character in `s` is ASCII whitespace (space, tab, \n, \r).
pub fn is_ws(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_alpha ────────────────────────────────────────────
    #[test]
    fn test_is_alpha_lowercase() {
        assert!(is_alpha("a"));
        assert!(is_alpha("z"));
        assert!(is_alpha("abc"));
    }

    #[test]
    fn test_is_alpha_uppercase() {
        assert!(is_alpha("A"));
        assert!(is_alpha("Z"));
        assert!(is_alpha("Hello"));
    }

    #[test]
    fn test_is_alpha_false_digit() {
        assert!(!is_alpha("3"));
        assert!(!is_alpha("a1"));
    }

    #[test]
    fn test_is_alpha_false_space() {
        assert!(!is_alpha(" "));
        assert!(!is_alpha("a "));
    }

    #[test]
    fn test_is_alpha_empty() {
        assert!(!is_alpha(""));
    }

    // ── is_alnum ────────────────────────────────────────────
    #[test]
    fn test_is_alnum_letters() {
        assert!(is_alnum("a"));
        assert!(is_alnum("Z"));
    }

    #[test]
    fn test_is_alnum_digits() {
        assert!(is_alnum("0"));
        assert!(is_alnum("9"));
        assert!(is_alnum("42"));
    }

    #[test]
    fn test_is_alnum_mixed() {
        assert!(is_alnum("a3"));
        assert!(is_alnum("foo123"));
    }

    #[test]
    fn test_is_alnum_false_special() {
        assert!(!is_alnum("_"));
        assert!(!is_alnum("a_b"));
        assert!(!is_alnum(" "));
    }

    #[test]
    fn test_is_alnum_empty() {
        assert!(!is_alnum(""));
    }

    // ── is_digit ────────────────────────────────────────────
    #[test]
    fn test_is_digit_single() {
        for d in '0'..='9' {
            assert!(is_digit(&d.to_string()));
        }
    }

    #[test]
    fn test_is_digit_multi() {
        assert!(is_digit("123"));
        assert!(is_digit("007"));
    }

    #[test]
    fn test_is_digit_false_alpha() {
        assert!(!is_digit("a"));
        assert!(!is_digit("1a"));
    }

    #[test]
    fn test_is_digit_false_space() {
        assert!(!is_digit(" "));
    }

    #[test]
    fn test_is_digit_empty() {
        assert!(!is_digit(""));
    }

    // ── is_ws ───────────────────────────────────────────────
    #[test]
    fn test_is_ws_space() {
        assert!(is_ws(" "));
        assert!(is_ws("   "));
    }

    #[test]
    fn test_is_ws_tab() {
        assert!(is_ws("\t"));
    }

    #[test]
    fn test_is_ws_newline() {
        assert!(is_ws("\n"));
        assert!(is_ws("\r\n"));
    }

    #[test]
    fn test_is_ws_false_letter() {
        assert!(!is_ws("a"));
        assert!(!is_ws(" a"));
    }

    #[test]
    fn test_is_ws_false_digit() {
        assert!(!is_ws("1"));
    }

    #[test]
    fn test_is_ws_empty() {
        assert!(!is_ws(""));
    }
}
