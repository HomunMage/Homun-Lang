// ============================================================
// Homun String Library — included by std.rs
// ============================================================

pub fn split(s: &str, sep: &str) -> Vec<String> {
    s.split(sep).map(|x| x.to_string()).collect()
}

pub fn join(v: &[String], sep: &str) -> String {
    v.join(sep)
}

pub fn trim(s: &str) -> String { s.trim().to_string() }
pub fn trim_start(s: &str) -> String { s.trim_start().to_string() }
pub fn trim_end(s: &str) -> String { s.trim_end().to_string() }

pub fn starts_with(s: &str, prefix: &str) -> bool { s.starts_with(prefix) }
pub fn ends_with(s: &str, suffix: &str) -> bool { s.ends_with(suffix) }

pub fn replace(s: &str, from: &str, to: &str) -> String { s.replace(from, to) }

pub fn to_upper(s: &str) -> String { s.to_uppercase() }
pub fn to_lower(s: &str) -> String { s.to_lowercase() }

pub fn chars(s: &str) -> Vec<String> {
    s.chars().map(|c| c.to_string()).collect()
}

pub fn find(s: &str, sub: &str) -> i32 {
    s.find(sub).map(|i| i as i32).unwrap_or(-1)
}

pub fn contains(s: &str, sub: &str) -> bool { s.contains(sub) }

pub fn repeat(s: &str, n: i32) -> String { s.repeat(n as usize) }

pub fn substr(s: &str, start: i32, end: i32) -> String {
    let len = s.len() as i32;
    let s_ = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
    let e_ = if end < 0 { (len + end).max(0) } else { end.min(len) } as usize;
    s.get(s_..e_).unwrap_or("").to_string()
}

pub fn strip_prefix(s: &str, prefix: &str) -> String {
    s.strip_prefix(prefix).unwrap_or(s).to_string()
}

pub fn strip_suffix(s: &str, suffix: &str) -> String {
    s.strip_suffix(suffix).unwrap_or(s).to_string()
}

pub fn lines(s: &str) -> Vec<String> {
    s.lines().map(|l| l.to_string()).collect()
}

pub fn is_empty(s: &str) -> bool { s.is_empty() }

pub fn is_digit(s: &str) -> bool { s.chars().all(|c| c.is_ascii_digit()) }
pub fn is_alpha(s: &str) -> bool { s.chars().all(|c| c.is_alphabetic()) }
pub fn is_alnum(s: &str) -> bool { s.chars().all(|c| c.is_alphanumeric()) }
pub fn is_upper(s: &str) -> bool { s.chars().all(|c| c.is_uppercase()) }
pub fn is_lower(s: &str) -> bool { s.chars().all(|c| c.is_lowercase()) }

pub fn pad_left(s: &str, width: i32, fill: &str) -> String {
    let w = width as usize;
    if s.len() >= w { return s.to_string(); }
    let pad = fill.repeat((w - s.len()) / fill.len().max(1) + 1);
    format!("{}{}", &pad[..w - s.len()], s)
}

pub fn pad_right(s: &str, width: i32, fill: &str) -> String {
    let w = width as usize;
    if s.len() >= w { return s.to_string(); }
    let pad = fill.repeat((w - s.len()) / fill.len().max(1) + 1);
    format!("{}{}", s, &pad[..w - s.len()])
}

pub fn char_at(s: &str, i: i32) -> String {
    let len = s.chars().count() as i32;
    let idx = if i < 0 { (len + i).max(0) } else { i } as usize;
    s.chars().nth(idx).map(|c| c.to_string()).unwrap_or_default()
}

pub fn parse_int(s: &str) -> i32 {
    s.trim().parse::<i32>().unwrap_or(0)
}

pub fn parse_float(s: &str) -> f32 {
    s.trim().parse::<f32>().unwrap_or(0.0)
}
