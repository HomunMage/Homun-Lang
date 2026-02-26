// ============================================================
// Homun Standard Library — import with: use std
// ============================================================

// ── range(n), range(a,b), range(a,b,step) ───────────────────

macro_rules! range {
    ($n:expr) => { 0..$n };
    ($s:expr, $e:expr) => { $s..$e };
    ($s:expr, $e:expr, $st:expr) => {{
        let (s_, e_, st_) = ($s, $e, $st);
        let mut i_ = s_;
        std::iter::from_fn(move || {
            if (st_ > 0 && i_ < e_) || (st_ < 0 && i_ > e_) {
                let cur = i_; i_ += st_; Some(cur)
            } else { None }
        })
    }};
}

// ── len(x) ──────────────────────────────────────────────────

macro_rules! len {
    ($e:expr) => { ($e).homun_len() as i32 }
}

pub trait HomunLen { fn homun_len(&self) -> usize; }
impl<T> HomunLen for Vec<T>                                { fn homun_len(&self) -> usize { self.len() } }
impl<K, V> HomunLen for std::collections::HashMap<K, V>    { fn homun_len(&self) -> usize { self.len() } }
impl<T> HomunLen for std::collections::HashSet<T>          { fn homun_len(&self) -> usize { self.len() } }
impl HomunLen for String                                    { fn homun_len(&self) -> usize { self.len() } }
impl HomunLen for str                                       { fn homun_len(&self) -> usize { self.len() } }

// ── filter(vec, fn), map(vec, fn), reduce(vec, fn) ──────────

macro_rules! filter {
    ($v:expr, $f:expr) => {
        ($v).iter().cloned().filter(|x| ($f)(x.clone())).collect::<Vec<_>>()
    };
}

macro_rules! map {
    ($v:expr, $f:expr) => {
        ($v).iter().cloned().map($f).collect::<Vec<_>>()
    };
}

macro_rules! reduce {
    ($v:expr, $f:expr) => {
        ($v).into_iter().reduce($f)
    };
}

// ── String Library ───────────────────────────────────────────

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

// ── Math Library ─────────────────────────────────────────────

pub fn abs<T: std::ops::Neg<Output = T> + PartialOrd + Default + Copy>(x: T) -> T {
    if x < T::default() { -x } else { x }
}

pub fn min<T: PartialOrd>(a: T, b: T) -> T { if a <= b { a } else { b } }
pub fn max<T: PartialOrd>(a: T, b: T) -> T { if a >= b { a } else { b } }

pub fn clamp<T: PartialOrd>(x: T, lo: T, hi: T) -> T {
    if x < lo { lo } else if x > hi { hi } else { x }
}

// ── Collection Library ───────────────────────────────────────

pub fn sorted<T: Clone + Ord>(v: &[T]) -> Vec<T> {
    let mut out = v.to_vec(); out.sort(); out
}

pub fn reversed<T: Clone>(v: &[T]) -> Vec<T> {
    let mut out = v.to_vec(); out.reverse(); out
}

pub fn enumerate<T: Clone>(v: &[T]) -> Vec<(i32, T)> {
    v.iter().cloned().enumerate().map(|(i, x)| (i as i32, x)).collect()
}

pub fn zip<A: Clone, B: Clone>(a: &[A], b: &[B]) -> Vec<(A, B)> {
    a.iter().cloned().zip(b.iter().cloned()).collect()
}

pub fn flatten<T: Clone>(v: &[Vec<T>]) -> Vec<T> {
    v.iter().flat_map(|x| x.iter().cloned()).collect()
}

pub fn any<T: Clone>(v: &[T], f: impl Fn(T) -> bool) -> bool {
    v.iter().cloned().any(|x| f(x))
}

pub fn all<T: Clone>(v: &[T], f: impl Fn(T) -> bool) -> bool {
    v.iter().cloned().all(|x| f(x))
}

pub fn count<T: Clone>(v: &[T], f: impl Fn(T) -> bool) -> i32 {
    v.iter().cloned().filter(|x| f(x.clone())).count() as i32
}

pub fn unique<T: Clone + Eq + std::hash::Hash>(v: &[T]) -> Vec<T> {
    let mut seen = std::collections::HashSet::new();
    v.iter().cloned().filter(|x| seen.insert(x.clone())).collect()
}

pub fn index_of<T: Clone + PartialEq>(v: &[T], item: &T) -> i32 {
    v.iter().position(|x| x == item).map(|i| i as i32).unwrap_or(-1)
}
