//! homunc — Homun to Rust compiler (library crate).
//!
//! Exposes compiler internals so that future .hom-compiled modules (sema_hom,
//! codegen_hom) can be wired in without touching src/main.rs.
//!
//! Phase 0: re-exports the existing hand-written Rust modules.
//! Phase 1: sema_hom will replace sema.rs.
//! Phase 2: codegen_hom will replace codegen.rs.

#![allow(
    unused_variables,
    unused_mut,
    dead_code,
    unused_imports,
    unused_macros,
    unused_assignments
)]
#![allow(non_snake_case)]
#![allow(
    clippy::clone_on_copy,
    clippy::redundant_field_names,
    clippy::assign_op_pattern,
    clippy::no_effect,
    clippy::unused_unit,
    clippy::needless_return,
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::ptr_arg,
    clippy::comparison_to_empty,
    clippy::redundant_clone,
    clippy::inherent_to_string,
    clippy::useless_conversion,
    clippy::unnecessary_to_owned,
    clippy::too_many_arguments,
    clippy::redundant_closure,
    clippy::bool_comparison,
    clippy::while_immutable_condition,
    clippy::identity_op,
    clippy::almost_swapped,
    clippy::needless_borrow,
    clippy::op_ref,
    clippy::iter_overeager_cloned
)]

// ── Homun runtime (builtin + std + re + heap) ──────────────────────────────
#[macro_use]
pub mod runtime {
    include!(concat!(env!("OUT_DIR"), "/runtime.rs"));
}

// ── dep/ — hand-written Rust helpers for .hom modules (Phase 1/2 bridges) ──
#[path = "dep/mod.rs"]
pub mod dep;

// ── Compiler pipeline modules ────────────────────────────────────────────────
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod resolver;

// ── sema_hom — semantic analysis compiled from sema.hom ──────────────────────
pub mod sema_hom {
    use crate::dep::*;
    use crate::runtime::*;
    include!(concat!(env!("OUT_DIR"), "/sema.rs"));
}

// ── codegen_hom — code generator compiled from codegen.hom ───────────────────
pub mod codegen_hom {
    use crate::dep::*;
    use crate::runtime::*;
    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}

// ── Embedded runtime library content ─────────────────────────────────────────
// resolver.rs calls embedded_rs() from the crate root, so it must be pub here.
pub fn embedded_rs(name: &str) -> Option<String> {
    match name {
        "std" => {
            let mod_rs: String = include_str!("hom/std/mod.rs")
                .lines()
                .filter(|l| !l.trim().starts_with("include!("))
                .collect::<Vec<_>>()
                .join("\n");
            Some(format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                mod_rs,
                include_str!("hom/std/str.rs"),
                include_str!("hom/std/math.rs"),
                include_str!("hom/std/collection.rs"),
                include_str!("hom/std/dict.rs"),
                include_str!("hom/std/stack.rs"),
                include_str!("hom/std/deque.rs"),
                include_str!("hom/std/io.rs"),
            ))
        }
        "re" => Some(include_str!("hom/re.rs").to_string()),
        "heap" => Some(include_str!("hom/heap.rs").to_string()),
        "chars" => Some(include_str!("hom/chars.rs").to_string()),
        "str_ext" => Some(include_str!("hom/str_ext.rs").to_string()),
        "dict" => Some(include_str!("hom/dict.rs").to_string()),
        _ => None,
    }
}
