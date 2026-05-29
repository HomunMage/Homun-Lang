// lexer_imp.rs — Type definitions and helper accessors for lexer.hom.
//
// Importing this file via `use lexer_imp` in lexer.hom sets has_rs_dep=true
// in the homunc sema checker, disabling undefined-reference checks for dep/*
// functions and runtime functions that are available at include!() time in
// lib.rs but unknown to homunc's static checker.
//
// Pos, Token, TokenKind are now defined in lexer.hom (migrated by R3).
//
// Pos constructors / accessors:
//   make_pos(line, col) -> Pos          — i64 args (Homun int → i32 cast)
//   pos_line(p) -> i64
//   pos_col(p) -> i64
//
// Token char helper:
//   make_token_char_from_str(val, pos) -> Token  — String → char payload
//
// Char-testing helpers (is_alpha, is_digit, is_alnum, is_whitespace,
// is_newline) live in hom-std/chars.rs — import via `use chars` in lexer.hom.
//
// Operator dispatch (now return TokenKind / Option<TokenKind>):
//   ls_try_multi_op(s) -> (TokenKind, i64)   — (Eof, 0) = no match
//   ls_try_single_op(c) -> Option<TokenKind>  — migrated to lexer.hom

// Lexer: helper functions for Homun tokens.

// ── Pos constructors / accessors ─────────────────────────────────────────────

/// Create a Pos from (line, col).  Homun int is i64; cast to i32 here.
pub fn make_pos(line: i64, col: i64) -> Pos {
    Pos {
        line: line as i32,
        col: col as i32,
    }
}

/// Return the line number as i64.
pub fn pos_line(p: Pos) -> i64 {
    p.line as i64
}

/// Return the column number as i64.
pub fn pos_col(p: Pos) -> i64 {
    p.col as i64
}

// ── Token char helper ─────────────────────────────────────────────────────────

/// Convert a 1-char String to a Token with Char kind.
/// Used by lexer.hom which works with String, not char.
pub fn make_token_char_from_str(val: String, pos: Pos) -> Token {
    Token {
        kind: TokenKind::Char(val.chars().next().unwrap_or('\0')),
        pos,
    }
}

// ── LexState — mutable state machine for the lexer ───────────────────────────

/// All mutable state for the character-walking lexer loop.
/// Helpers take and return owned LexState (callers pass .clone() via Homun codegen).
#[derive(Clone)]
pub struct LexState {
    chars: Vec<char>,
    i: i64,
    line: i64,
    col: i64,
    err: String,
}

pub fn make_lex_state(source: String) -> LexState {
    LexState {
        chars: source.chars().collect(),
        i: 0,
        line: 1,
        col: 1,
        err: String::new(),
    }
}

// ── LexState accessors ────────────────────────────────────────────────────────

pub fn ls_len(s: LexState) -> i64 {
    s.chars.len() as i64
}

/// True if `i >= len(chars)` AND `err` is empty — combined while-condition.
pub fn ls_should_continue(s: LexState) -> bool {
    s.i < s.chars.len() as i64 && s.err.is_empty()
}

pub fn ls_has_err(s: LexState) -> bool {
    !s.err.is_empty()
}

pub fn ls_get_err(s: LexState) -> String {
    s.err
}

pub fn ls_line(s: LexState) -> i64 {
    s.line
}

pub fn ls_col(s: LexState) -> i64 {
    s.col
}

pub fn ls_pos(s: LexState) -> Pos {
    Pos {
        line: s.line as i32,
        col: s.col as i32,
    }
}

/// Current character as a 1-char String, or "" if at/past end.
/// Returns String (not char) so .hom code can compare with string literals.
pub fn ls_cur(s: LexState) -> String {
    s.chars
        .get(s.i as usize)
        .map(|c| c.to_string())
        .unwrap_or_default()
}

/// Character at chars[i + offset] as a 1-char String, or "" if out of bounds.
/// Returns String (not char) so .hom code can compare with string literals.
pub fn ls_peek(s: LexState, offset: i64) -> String {
    s.chars
        .get((s.i + offset) as usize)
        .map(|c| c.to_string())
        .unwrap_or_default()
}

// ── LexState mutators ─────────────────────────────────────────────────────────

/// Advance i by n, col by n (for non-newline characters).
pub fn ls_advance_col(s: LexState, n: i64) -> LexState {
    LexState {
        i: s.i + n,
        col: s.col + n,
        ..s
    }
}

/// Advance past a newline: i+1, line+1, col=1.
pub fn ls_advance_newline(s: LexState) -> LexState {
    LexState {
        i: s.i + 1,
        line: s.line + 1,
        col: 1,
        ..s
    }
}

/// Return a new LexState with the error field set.
pub fn ls_set_err(s: LexState, err: String) -> LexState {
    LexState { err, ..s }
}

// ── Operator dispatch helpers ─────────────────────────────────────────────────

/// Try to lex a two-character operator at the current position.
/// Returns `(kind, chars_consumed)` or `(Eof, 0)` if none matched.
pub fn ls_try_multi_op(s: LexState) -> (TokenKind, i64) {
    let i = s.i as usize;
    let c = s.chars.get(i).copied().unwrap_or('\0');
    let c1 = s.chars.get(i + 1).copied().unwrap_or('\0');
    let c2 = s.chars.get(i + 2).copied().unwrap_or('\0');
    if (c, c1, c2) == (':', ':', '=') {
        return (TokenKind::MutAssign, 3);
    }
    if (c, c1) == (':', ':') {
        return (TokenKind::DoubleColon, 2);
    }
    match (c, c1) {
        (':', '=') => (TokenKind::Assign, 2),
        ('-', '>') => (TokenKind::Arrow, 2),
        ('=', '>') => (TokenKind::FatArrow, 2),
        ('=', '=') => (TokenKind::Eq, 2),
        ('!', '=') => (TokenKind::Neq, 2),
        ('<', '=') => (TokenKind::Le, 2),
        ('>', '=') => (TokenKind::Ge, 2),
        _ => (TokenKind::Eof, 0),
    }
}

// ── Token list helpers ────────────────────────────────────────────────────────

pub fn tokens_new() -> Vec<Token> {
    vec![]
}

pub fn tokens_push(mut ts: Vec<Token>, t: Token) -> Vec<Token> {
    ts.push(t);
    ts
}

// ── Helpers for lexer.hom inner-loop migration ──────────────────────────────

/// Current position index.
pub fn ls_i(s: LexState) -> i64 {
    s.i
}

/// True if position is within bounds (ignores err — for inner loops).
pub fn ls_not_at_end(s: LexState) -> bool {
    (s.i as usize) < s.chars.len()
}

/// Advance i by n WITHOUT changing col/line (for skipping content where col doesn't matter).
pub fn ls_advance_i_only(s: LexState, n: i64) -> LexState {
    LexState { i: s.i + n, ..s }
}

/// Set line and col directly.
pub fn ls_set_line_col(s: LexState, line: i64, col: i64) -> LexState {
    LexState { line, col, ..s }
}

/// Extract chars[start..end] as a String.
pub fn ls_substr(s: LexState, start: i64, end: i64) -> String {
    s.chars[start as usize..end as usize].iter().collect()
}

/// String length (number of chars).
pub fn str_len(s: String) -> i64 {
    s.len() as i64
}

/// String contains a character (given as 1-char string).
pub fn str_contains(s: String, ch: String) -> bool {
    s.contains(&ch)
}

/// Parse a string as an integer (i32). Returns (true, value) on success, (false, 0) on failure.
pub fn parse_int_result(s: String) -> (bool, i32) {
    match s.parse::<i32>() {
        Ok(v) => (true, v),
        Err(_) => (false, 0),
    }
}

/// Parse a string as a float (f32). Returns (true, value) on success, (false, 0.0) on failure.
pub fn parse_float_result(s: String) -> (bool, f32) {
    match s.parse::<f32>() {
        Ok(v) => (true, v),
        Err(_) => (false, 0.0),
    }
}

/// Create an empty String.
pub fn str_new() -> String {
    String::new()
}

/// Build a string by appending a 1-char string to an existing string.
pub fn str_push(mut s: String, ch: String) -> String {
    s.push_str(&ch);
    s
}
