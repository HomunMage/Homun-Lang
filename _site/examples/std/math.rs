// ============================================================
// Homun Math Library — included by std.rs
// ============================================================

pub fn abs<T: std::ops::Neg<Output = T> + PartialOrd + Default + Copy>(x: T) -> T {
    if x < T::default() { -x } else { x }
}

pub fn min<T: PartialOrd>(a: T, b: T) -> T { if a <= b { a } else { b } }
pub fn max<T: PartialOrd>(a: T, b: T) -> T { if a >= b { a } else { b } }

pub fn clamp<T: PartialOrd>(x: T, lo: T, hi: T) -> T {
    if x < lo { lo } else if x > hi { hi } else { x }
}
