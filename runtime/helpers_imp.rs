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
