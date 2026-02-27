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

// ── Sub-libraries ───────────────────────────────────────────

include!("str.rs");
include!("math.rs");
include!("collection.rs");
include!("dict.rs");
include!("stack.rs");
include!("deque.rs");
include!("io.rs");
