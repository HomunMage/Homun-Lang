// ============================================================
// Homun Collection Library — included by std.rs
// ============================================================

// HashSet is already imported by builtin.rs

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
    let mut seen = HashSet::new();
    v.iter().cloned().filter(|x| seen.insert(x.clone())).collect()
}

pub fn index_of<T: Clone + PartialEq>(v: &[T], item: &T) -> i32 {
    v.iter().position(|x| x == item).map(|i| i as i32).unwrap_or(-1)
}

pub fn push<T>(v: &mut Vec<T>, item: T) {
    v.push(item);
}

pub fn pop<T>(v: &mut Vec<T>) -> Option<T> {
    v.pop()
}

pub fn remove<T>(v: &mut Vec<T>, index: i32) -> T {
    let i = if index < 0 { (v.len() as i32 + index).max(0) as usize } else { index as usize };
    v.remove(i)
}
