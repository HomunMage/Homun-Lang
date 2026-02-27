// ── use helpers_imp ──
// ============================================================
// Homun helpers — Rust implementation (imported by helpers.hom)
// ============================================================

/// Macro names that should be emitted as `name!(args)` instead of `name(args)`.
pub const HOMUN_MACRO_LIST: &[&str] = &[
    "range", "len", "filter", "map", "reduce", "slice", "dict", "set",
];

pub fn is_homun_macro(name: &str) -> bool {
    HOMUN_MACRO_LIST.contains(&name)
}

pub fn indent(mut n: i32) -> String {
    repeat("    ", n)
}
pub fn is_upper_first(mut s: String) -> bool {
    char_at(s.clone(), 0) >= "A" && char_at(s.clone(), 0) <= "Z"
}