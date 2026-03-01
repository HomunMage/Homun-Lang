// lexer_imp.rs — Type definitions and helper accessors for lexer.hom.
//
// Importing this file via `use lexer_imp` in lexer.hom sets has_rs_dep=true
// in the homunc sema checker, disabling undefined-reference checks for dep/*
// functions and runtime functions that are available at include!() time in
// lib.rs but unknown to homunc's static checker.
//
// Type definitions:
//   Pos, Token, TokenKind   — lexer types (defined here)
//
// Pos constructors / accessors:
//   make_pos(line, col) -> Pos          — i64 args (Homun int → usize cast)
//   pos_line(p) -> i64
//   pos_col(p) -> i64
//   pos_inc_col(p) -> Pos               — advance col by 1
//   pos_add_col(p, n) -> Pos            — advance col by n
//   pos_newline(p) -> Pos               — increment line, reset col to 1
//
// Token constructors (value-carrying variants):
//   make_token_int(val, pos) -> Token
//   make_token_float(val, pos) -> Token
//   make_token_bool(val, pos) -> Token
//   make_token_str(val, pos) -> Token
//   make_token_char(val, pos) -> Token
//   make_token_ident(val, pos) -> Token
//   make_token_none(pos) -> Token
//
// Token constructor (parameterless variants via string dispatch):
//   make_token(kind, pos) -> Token
//     Accepts: "Use","Struct","Enum","For","In","While","Do","If","Else",
//              "Match","Break","Continue","And","Or","Not","As","Rec",
//              "Assign","Arrow","FatArrow","Pipe","Dot","Plus","Minus",
//              "Star","Slash","Percent","Eq","Neq","Lt","Gt","Le","Ge",
//              "Colon","Comma","Semi","Underscore","At","Question",
//              "LParen","RParen","LBrace","RBrace","LBracket","RBracket","Eof"
//
// Char-testing helpers:
//   is_alpha(c) -> bool
//   is_digit(c) -> bool
//   is_alnum(c) -> bool      — alphanumeric or '_'
//   is_whitespace(c) -> bool
//   is_newline(c) -> bool

/// Lexer: type definitions for Homun tokens.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    None,
    // Identifiers & Keywords
    Ident(String),
    Use,
    Struct,
    Enum,
    For,
    In,
    While,
    Do,
    If,
    Else,
    Match,
    Break,
    Continue,
    And,
    Or,
    Not,
    As,
    Rec,
    // Operators
    MutAssign, // ::=
    Assign,    // :=
    Arrow,     // ->
    FatArrow,  // =>
    Pipe,      // |
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,  // ==
    Neq, // !=
    Lt,
    Gt,
    Le, // <=
    Ge, // >=
    Colon,
    Comma,
    Semi,
    Underscore,
    At,
    Question,
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eof,
}

// ── Pos constructors / accessors ─────────────────────────────────────────────

/// Create a Pos from (line, col).  Homun int is i64; cast to usize here.
pub fn make_pos(line: i64, col: i64) -> Pos {
    Pos {
        line: line as usize,
        col: col as usize,
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

/// Return a new Pos with col incremented by 1.
pub fn pos_inc_col(p: Pos) -> Pos {
    Pos {
        line: p.line,
        col: p.col + 1,
    }
}

/// Return a new Pos with col incremented by n.
pub fn pos_add_col(p: Pos, n: i64) -> Pos {
    Pos {
        line: p.line,
        col: p.col + n as usize,
    }
}

/// Return a new Pos for the start of the next line (line+1, col=1).
pub fn pos_newline(p: Pos) -> Pos {
    Pos {
        line: p.line + 1,
        col: 1,
    }
}

// ── Value-carrying token constructors ────────────────────────────────────────

pub fn make_token_int(val: i64, pos: Pos) -> Token {
    Token {
        kind: TokenKind::Int(val),
        pos,
    }
}

pub fn make_token_float(val: f64, pos: Pos) -> Token {
    Token {
        kind: TokenKind::Float(val),
        pos,
    }
}

pub fn make_token_bool(val: bool, pos: Pos) -> Token {
    Token {
        kind: TokenKind::Bool(val),
        pos,
    }
}

pub fn make_token_str(val: String, pos: Pos) -> Token {
    Token {
        kind: TokenKind::Str(val),
        pos,
    }
}

pub fn make_token_char(val: char, pos: Pos) -> Token {
    Token {
        kind: TokenKind::Char(val),
        pos,
    }
}

pub fn make_token_ident(val: String, pos: Pos) -> Token {
    Token {
        kind: TokenKind::Ident(val),
        pos,
    }
}

pub fn make_token_none(pos: Pos) -> Token {
    Token {
        kind: TokenKind::None,
        pos,
    }
}

// ── Parameterless-variant constructor (string dispatch) ───────────────────────

/// Construct a Token whose kind carries no payload.
/// Pass the variant name as a string (e.g. "Assign", "LParen", "Eof").
/// Panics on unknown kind name — intended for use with literal constants only.
pub fn make_token(kind: String, pos: Pos) -> Token {
    let k = match kind.as_str() {
        // Keywords
        "Use" => TokenKind::Use,
        "Struct" => TokenKind::Struct,
        "Enum" => TokenKind::Enum,
        "For" => TokenKind::For,
        "In" => TokenKind::In,
        "While" => TokenKind::While,
        "Do" => TokenKind::Do,
        "If" => TokenKind::If,
        "Else" => TokenKind::Else,
        "Match" => TokenKind::Match,
        "Break" => TokenKind::Break,
        "Continue" => TokenKind::Continue,
        "And" => TokenKind::And,
        "Or" => TokenKind::Or,
        "Not" => TokenKind::Not,
        "As" => TokenKind::As,
        "Rec" => TokenKind::Rec,
        // Operators
        "MutAssign" => TokenKind::MutAssign,
        "Assign" => TokenKind::Assign,
        "Arrow" => TokenKind::Arrow,
        "FatArrow" => TokenKind::FatArrow,
        "Pipe" => TokenKind::Pipe,
        "Dot" => TokenKind::Dot,
        "Plus" => TokenKind::Plus,
        "Minus" => TokenKind::Minus,
        "Star" => TokenKind::Star,
        "Slash" => TokenKind::Slash,
        "Percent" => TokenKind::Percent,
        "Eq" => TokenKind::Eq,
        "Neq" => TokenKind::Neq,
        "Lt" => TokenKind::Lt,
        "Gt" => TokenKind::Gt,
        "Le" => TokenKind::Le,
        "Ge" => TokenKind::Ge,
        "Colon" => TokenKind::Colon,
        "Comma" => TokenKind::Comma,
        "Semi" => TokenKind::Semi,
        "Underscore" => TokenKind::Underscore,
        "At" => TokenKind::At,
        "Question" => TokenKind::Question,
        // Delimiters
        "LParen" => TokenKind::LParen,
        "RParen" => TokenKind::RParen,
        "LBrace" => TokenKind::LBrace,
        "RBrace" => TokenKind::RBrace,
        "LBracket" => TokenKind::LBracket,
        "RBracket" => TokenKind::RBracket,
        "Eof" => TokenKind::Eof,
        _ => panic!("make_token: unknown parameterless kind '{}'", kind),
    };
    Token { kind: k, pos }
}

// ── Char-testing helpers ──────────────────────────────────────────────────────

/// True if c is an alphabetic character (Unicode-aware, matches Homun ident start).
/// Takes a 1-char String (as returned by ls_cur/ls_peek) for .hom interop.
pub fn is_alpha(c: String) -> bool {
    c.chars().next().is_some_and(|ch| ch.is_alphabetic())
}

/// True if c is an ASCII decimal digit.
/// Takes a 1-char String (as returned by ls_cur/ls_peek) for .hom interop.
pub fn is_digit(c: String) -> bool {
    c.chars().next().is_some_and(|ch| ch.is_ascii_digit())
}

/// True if c is alphanumeric or underscore (matches Homun ident continuation).
/// Takes a 1-char String (as returned by ls_cur/ls_peek) for .hom interop.
pub fn is_alnum(c: String) -> bool {
    c.chars()
        .next()
        .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
}

/// True if c is Unicode whitespace.
/// Takes a 1-char String (as returned by ls_cur/ls_peek) for .hom interop.
pub fn is_whitespace(c: String) -> bool {
    c.chars().next().is_some_and(|ch| ch.is_whitespace())
}

/// True if c is a newline character.
/// Takes a 1-char String (as returned by ls_cur/ls_peek) for .hom interop.
pub fn is_newline(c: String) -> bool {
    c == "\n"
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
        line: s.line as usize,
        col: s.col as usize,
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

// ── Complex inner-loop actions (done in Rust to avoid nested-while issues) ────

/// Skip from current position to (but not including) the next '\n'.
/// Caller must handle the '\n' itself as whitespace.
pub fn ls_skip_line_comment(mut s: LexState) -> LexState {
    while (s.i as usize) < s.chars.len() && s.chars[s.i as usize] != '\n' {
        s.i += 1;
    }
    s
}

/// Skip a `/* ... */` block comment starting at current position.
/// Sets `err` if the comment is unterminated.
pub fn ls_skip_block_comment(mut s: LexState) -> LexState {
    let start_line = s.line;
    let start_col = s.col;
    // Skip the opening /*
    s.i += 2;
    s.col += 2;
    loop {
        if s.i as usize >= s.chars.len() {
            s.err = format!(
                "Unterminated block comment at line {}, col {}",
                start_line, start_col
            );
            return s;
        }
        let c = s.chars[s.i as usize];
        let c1 = s.chars.get(s.i as usize + 1).copied().unwrap_or('\0');
        if c == '*' && c1 == '/' {
            s.i += 2;
            s.col += 2;
            return s;
        }
        if c == '\n' {
            s.line += 1;
            s.col = 1;
        } else {
            s.col += 1;
        }
        s.i += 1;
    }
}

/// Read a `"..."` string literal starting at the current `"`.
/// Returns `(new_state, content)`.  Sets `err` if unterminated.
pub fn ls_read_string(mut s: LexState) -> (LexState, String) {
    // Skip opening "
    s.i += 1;
    s.col += 1;
    let mut buf = String::new();
    loop {
        if s.i as usize >= s.chars.len() {
            s.err = "Unterminated string literal".to_string();
            return (s, buf);
        }
        let ch = s.chars[s.i as usize];
        if ch == '"' {
            s.i += 1;
            s.col += 1;
            return (s, buf);
        }
        if ch == '\\' {
            s.i += 1;
            s.col += 1;
            if s.i as usize >= s.chars.len() {
                s.err = "Unterminated string literal".to_string();
                return (s, buf);
            }
            let esc = match s.chars[s.i as usize] {
                'n' => '\n',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                other => other,
            };
            buf.push(esc);
            s.i += 1;
            s.col += 1;
        } else {
            buf.push(ch);
            s.i += 1;
            s.col += 1;
        }
    }
}

/// Read a `'x'` or `'\n'` char literal starting at the current `'`.
/// Returns `(new_state, char)`.  Sets `err` if malformed.
pub fn ls_read_char_lit(mut s: LexState) -> (LexState, char) {
    let start_line = s.line;
    let start_col = s.col;
    // Skip opening '
    s.i += 1;
    s.col += 1;
    if s.i as usize >= s.chars.len() {
        s.err = "Unterminated char literal".to_string();
        return (s, '\0');
    }
    let ch = if s.chars[s.i as usize] == '\\' {
        s.i += 1;
        s.col += 1;
        if s.i as usize >= s.chars.len() {
            s.err = "Unterminated char literal".to_string();
            return (s, '\0');
        }
        let esc = match s.chars[s.i as usize] {
            'n' => '\n',
            't' => '\t',
            '\\' => '\\',
            '\'' => '\'',
            '0' => '\0',
            other => other,
        };
        s.i += 1;
        s.col += 1;
        esc
    } else {
        let c = s.chars[s.i as usize];
        s.i += 1;
        s.col += 1;
        c
    };
    if s.i as usize >= s.chars.len() || s.chars[s.i as usize] != '\'' {
        s.err = format!(
            "Unterminated char literal at line {}, col {}",
            start_line, start_col
        );
        return (s, '\0');
    }
    // Skip closing '
    s.i += 1;
    s.col += 1;
    (s, ch)
}

/// Read an integer or float literal starting at the current digit.
/// Returns `(new_state, token)`.  Sets `err` on parse failure.
pub fn ls_read_number(mut s: LexState) -> (LexState, Token) {
    let start = s.i as usize;
    let n = s.chars.len();
    while (s.i as usize) < n
        && (s.chars[s.i as usize].is_ascii_digit() || s.chars[s.i as usize] == '.')
    {
        s.i += 1;
    }
    let raw: String = s.chars[start..s.i as usize].iter().collect();
    let raw_len = raw.len() as i64;
    let pos = Pos {
        line: s.line as usize,
        col: s.col as usize,
    };
    let kind = if raw.contains('.') {
        match raw.parse::<f64>() {
            Ok(v) => TokenKind::Float(v),
            Err(e) => {
                s.err = format!("Invalid float: {}", e);
                return (s, Token { kind: TokenKind::Eof, pos });
            }
        }
    } else {
        match raw.parse::<i64>() {
            Ok(v) => TokenKind::Int(v),
            Err(e) => {
                s.err = format!("Invalid int: {}", e);
                return (s, Token { kind: TokenKind::Eof, pos });
            }
        }
    };
    s.col += raw_len;
    (s, Token { kind, pos })
}

/// Read an identifier or keyword starting at the current alphabetic char.
/// Returns `(new_state, token)`.
pub fn ls_read_ident(mut s: LexState) -> (LexState, Token) {
    let start = s.i as usize;
    let n = s.chars.len();
    while (s.i as usize) < n
        && (s.chars[s.i as usize].is_alphanumeric() || s.chars[s.i as usize] == '_')
    {
        s.i += 1;
    }
    let word: String = s.chars[start..s.i as usize].iter().collect();
    let word_len = word.len() as i64;
    let pos = Pos {
        line: s.line as usize,
        col: s.col as usize,
    };
    let kind = ls_keyword(word);
    s.col += word_len;
    (s, Token { kind, pos })
}

fn ls_keyword(s: String) -> TokenKind {
    match s.as_str() {
        "use" => TokenKind::Use,
        "struct" => TokenKind::Struct,
        "enum" => TokenKind::Enum,
        "for" => TokenKind::For,
        "in" => TokenKind::In,
        "while" => TokenKind::While,
        "do" => TokenKind::Do,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "match" => TokenKind::Match,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "and" => TokenKind::And,
        "or" => TokenKind::Or,
        "not" => TokenKind::Not,
        "as" => TokenKind::As,
        "rec" => TokenKind::Rec,
        "true" => TokenKind::Bool(true),
        "false" => TokenKind::Bool(false),
        "none" => TokenKind::None,
        _ => TokenKind::Ident(s),
    }
}

// ── Operator dispatch helpers ─────────────────────────────────────────────────

/// Try to lex a two-character operator at the current position.
/// Returns `(kind_name, chars_consumed)` or `("", 0)` if none matched.
pub fn ls_try_multi_op(s: LexState) -> (String, i64) {
    let i = s.i as usize;
    let c = s.chars.get(i).copied().unwrap_or('\0');
    let c1 = s.chars.get(i + 1).copied().unwrap_or('\0');
    let c2 = s.chars.get(i + 2).copied().unwrap_or('\0');
    if (c, c1, c2) == (':', ':', '=') {
        return ("MutAssign".to_string(), 3);
    }
    match (c, c1) {
        (':', '=') => ("Assign".to_string(), 2),
        ('-', '>') => ("Arrow".to_string(), 2),
        ('=', '>') => ("FatArrow".to_string(), 2),
        ('=', '=') => ("Eq".to_string(), 2),
        ('!', '=') => ("Neq".to_string(), 2),
        ('<', '=') => ("Le".to_string(), 2),
        ('>', '=') => ("Ge".to_string(), 2),
        _ => (String::new(), 0),
    }
}

/// Try to lex a single-character operator/delimiter.
/// Takes a 1-char String (as returned by ls_cur) for .hom interop.
/// Returns the kind name string, or `""` if the character is unknown.
pub fn ls_try_single_op(c: String) -> String {
    match c.as_str() {
        "|" => "Pipe".to_string(),
        "." => "Dot".to_string(),
        "+" => "Plus".to_string(),
        "-" => "Minus".to_string(),
        "*" => "Star".to_string(),
        "/" => "Slash".to_string(),
        "%" => "Percent".to_string(),
        "<" => "Lt".to_string(),
        ">" => "Gt".to_string(),
        ":" => "Colon".to_string(),
        "," => "Comma".to_string(),
        ";" => "Semi".to_string(),
        "_" => "Underscore".to_string(),
        "@" => "At".to_string(),
        "?" => "Question".to_string(),
        "(" => "LParen".to_string(),
        ")" => "RParen".to_string(),
        "{" => "LBrace".to_string(),
        "}" => "RBrace".to_string(),
        "[" => "LBracket".to_string(),
        "]" => "RBracket".to_string(),
        _ => String::new(),
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
