// ============================================================
// Homun Standard Library — runtime helpers for generated Rust
// ============================================================

use std::collections::{HashMap, HashSet};

// ── range!(n), range!(a,b), range!(a,b,step) ────────────────

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

// ── len!(x) ─────────────────────────────────────────────────

macro_rules! len {
    ($e:expr) => { ($e).homun_len() as i32 }
}

pub trait HomunLen { fn homun_len(&self) -> usize; }
impl<T> HomunLen for Vec<T>         { fn homun_len(&self) -> usize { self.len() } }
impl<K, V> HomunLen for HashMap<K, V> { fn homun_len(&self) -> usize { self.len() } }
impl<T> HomunLen for HashSet<T>     { fn homun_len(&self) -> usize { self.len() } }
impl HomunLen for String             { fn homun_len(&self) -> usize { self.len() } }
impl HomunLen for str                { fn homun_len(&self) -> usize { self.len() } }

// ── filter!(vec, fn), map!(vec, fn), reduce!(vec, fn) ───────

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

// ── Indexing: vec[i], dict[key] ─────────────────────────────

pub trait HomunIndex<K> {
    type Output;
    fn homun_idx(&self, key: K) -> Self::Output;
}

impl<T: Clone> HomunIndex<i32> for Vec<T> {
    type Output = T;
    fn homun_idx(&self, key: i32) -> T {
        self[if key < 0 { self.len() as i32 + key } else { key } as usize].clone()
    }
}

impl<V: Clone> HomunIndex<i32> for HashMap<i32, V> {
    type Output = V;
    fn homun_idx(&self, key: i32) -> V { self[&key].clone() }
}

impl<V: Clone> HomunIndex<&str> for HashMap<String, V> {
    type Output = V;
    fn homun_idx(&self, key: &str) -> V { self[key].clone() }
}

// ── Slicing: arr[1:], arr[::2], arr[::-1] ───────────────────

pub fn homun_slice<T: Clone>(v: &Vec<T>, start: i64, end: i64, step: i64) -> Vec<T> {
    let len = v.len() as i64;
    let norm = |i: i64| -> usize {
        let i = if i < 0 { len + i } else { i };
        i.max(0).min(len) as usize
    };
    let s = norm(start);
    let e = norm(end);
    if step > 0 {
        (s..e).step_by(step as usize).map(|i| v[i].clone()).collect()
    } else if step < 0 {
        let s2 = if end   == i64::MAX { 0 } else { norm(end) };
        let e2 = if start == 0        { len as usize } else { norm(start) };
        (s2..e2).rev().step_by((-step) as usize).map(|i| v[i].clone()).collect()
    } else {
        vec![]
    }
}

// ── Vec concat: a + b ───────────────────────────────────────

pub fn homun_concat<T>(mut a: Vec<T>, b: Vec<T>) -> Vec<T> {
    a.extend(b);
    a
}

// ── str(x) → String ────────────────────────────────────────

pub fn str_of<T: std::fmt::Display>(x: T) -> String { x.to_string() }
