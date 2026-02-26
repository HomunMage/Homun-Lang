// ============================================================
// Homun Built-in — always included by the compiler
// ============================================================

use std::collections::{HashMap, HashSet};

// ── Indexing: arr[i], dict[key] ─────────────────────────────

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

// ── Slicing: arr[a:b:c] ────────────────────────────────────

macro_rules! slice {
    ($v:expr, $start:expr, $end:expr, $step:expr) => {
        homun_slice(&$v, $start, $end, $step)
    };
}

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

// ── Dict/Set literals: @{k:v}, @(items) ─────────────────────

macro_rules! dict {
    ($($k:expr => $v:expr),* $(,)?) => { HashMap::from([$(($k, $v)),*]) };
    () => { HashMap::new() };
}

macro_rules! set {
    ($($v:expr),* $(,)?) => { HashSet::from([$($v),*]) };
    () => { HashSet::new() };
}

// ── Membership: x in collection ─────────────────────────────

macro_rules! homun_in {
    ($item:expr, $coll:expr) => { ($coll).homun_contains(&($item)) }
}

pub trait HomunContains<T> {
    fn homun_contains(&self, item: &T) -> bool;
}

impl<T: PartialEq> HomunContains<T> for Vec<T> {
    fn homun_contains(&self, item: &T) -> bool { self.contains(item) }
}
impl<T: Eq + std::hash::Hash> HomunContains<T> for HashSet<T> {
    fn homun_contains(&self, item: &T) -> bool { self.contains(item) }
}
impl<K: Eq + std::hash::Hash, V> HomunContains<K> for HashMap<K, V> {
    fn homun_contains(&self, item: &K) -> bool { self.contains_key(item) }
}
impl HomunContains<&str> for Vec<String> {
    fn homun_contains(&self, item: &&str) -> bool { self.iter().any(|s| s == *item) }
}
impl<V> HomunContains<&str> for HashMap<String, V> {
    fn homun_contains(&self, item: &&str) -> bool { self.contains_key(*item) }
}
impl HomunContains<char> for str {
    fn homun_contains(&self, item: &char) -> bool { self.contains(*item) }
}
impl HomunContains<char> for String {
    fn homun_contains(&self, item: &char) -> bool { self.contains(*item) }
}
impl HomunContains<&str> for str {
    fn homun_contains(&self, item: &&str) -> bool { self.contains(*item) }
}
impl HomunContains<&str> for String {
    fn homun_contains(&self, item: &&str) -> bool { self.contains(*item) }
}

// ── str(x) → String ────────────────────────────────────────

pub fn str_of<T: std::fmt::Display>(x: T) -> String { x.to_string() }
