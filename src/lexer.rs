/// Lexer: tokenises Homun source into Vec<Token>

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
    Assign,   // :=
    Arrow,    // ->
    FatArrow, // =>
    Pipe,     // |
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
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eof,
}

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    let mut line = 1usize;
    let mut col = 1usize;

    while i < chars.len() {
        let c = chars[i];

        // Whitespace
        if c.is_whitespace() {
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            i += 1;
            continue;
        }

        // Line comment
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Block comment
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let start_pos = Pos { line, col };
            i += 2;
            col += 2;
            loop {
                if i >= chars.len() {
                    return Err(format!(
                        "Unterminated block comment at line {}, col {}",
                        start_pos.line, start_pos.col
                    ));
                }
                if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    i += 2;
                    col += 2;
                    break;
                }
                if chars[i] == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                i += 1;
            }
            continue;
        }

        let pos = Pos { line, col };

        // String literal
        if c == '"' {
            i += 1;
            col += 1;
            let mut s = String::new();
            loop {
                if i >= chars.len() {
                    return Err("Unterminated string literal".to_string());
                }
                let ch = chars[i];
                if ch == '"' {
                    i += 1;
                    col += 1;
                    break;
                }
                if ch == '\\' {
                    i += 1;
                    col += 1;
                    if i >= chars.len() {
                        return Err("Unterminated string literal".to_string());
                    }
                    let esc = match chars[i] {
                        'n' => '\n',
                        't' => '\t',
                        '\\' => '\\',
                        '"' => '"',
                        other => other,
                    };
                    s.push(esc);
                    i += 1;
                    col += 1;
                } else {
                    s.push(ch);
                    i += 1;
                    col += 1;
                }
            }
            tokens.push(Token {
                kind: TokenKind::Str(s),
                pos,
            });
            continue;
        }

        // Number
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let raw: String = chars[start..i].iter().collect();
            let len = raw.len();
            let kind = if raw.contains('.') {
                TokenKind::Float(
                    raw.parse::<f64>()
                        .map_err(|e| format!("Invalid float: {}", e))?,
                )
            } else {
                TokenKind::Int(
                    raw.parse::<i64>()
                        .map_err(|e| format!("Invalid int: {}", e))?,
                )
            };
            tokens.push(Token { kind, pos });
            col += len;
            continue;
        }

        // Identifier / keyword
        if c.is_alphabetic() {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let len = word.len();
            let kind = keyword(&word);
            tokens.push(Token { kind, pos });
            col += len;
            continue;
        }

        // Multi-char operators (check before single-char)
        if c == ':' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(Token {
                kind: TokenKind::Assign,
                pos,
            });
            i += 2;
            col += 2;
            continue;
        }
        if c == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
            tokens.push(Token {
                kind: TokenKind::Arrow,
                pos,
            });
            i += 2;
            col += 2;
            continue;
        }
        if c == '=' && i + 1 < chars.len() && chars[i + 1] == '>' {
            tokens.push(Token {
                kind: TokenKind::FatArrow,
                pos,
            });
            i += 2;
            col += 2;
            continue;
        }
        if c == '=' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(Token {
                kind: TokenKind::Eq,
                pos,
            });
            i += 2;
            col += 2;
            continue;
        }
        if c == '!' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(Token {
                kind: TokenKind::Neq,
                pos,
            });
            i += 2;
            col += 2;
            continue;
        }
        if c == '<' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(Token {
                kind: TokenKind::Le,
                pos,
            });
            i += 2;
            col += 2;
            continue;
        }
        if c == '>' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(Token {
                kind: TokenKind::Ge,
                pos,
            });
            i += 2;
            col += 2;
            continue;
        }

        // Single-char tokens
        let kind = match c {
            '|' => TokenKind::Pipe,
            '.' => TokenKind::Dot,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '<' => TokenKind::Lt,
            '>' => TokenKind::Gt,
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semi,
            '_' => TokenKind::Underscore,
            '@' => TokenKind::At,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            _ => {
                return Err(format!(
                    "Unexpected character '{}' at line {}, col {}",
                    c, line, col
                ));
            }
        };
        tokens.push(Token { kind, pos });
        i += 1;
        col += 1;
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        pos: Pos { line, col },
    });
    Ok(tokens)
}

fn keyword(s: &str) -> TokenKind {
    match s {
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
        _ => TokenKind::Ident(s.to_string()),
    }
}
