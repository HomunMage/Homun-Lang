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
    clippy::while_immutable_condition,
    clippy::identity_op,
    clippy::needless_borrow,
    clippy::op_ref,
    clippy::missing_const_for_thread_local,
    clippy::single_match,
    noop_method_call
)]

// ── Homun runtime (builtin + std + re + heap) ──────────────────────────────
#[macro_use]
pub mod runtime {
    include!(concat!(env!("OUT_DIR"), "/runtime.rs"));
}

// ── Compiler pipeline modules ────────────────────────────────────────────────
// ast is now compiled from src/ast.hom by build.rs
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/ast.rs"));
}

// ── parser_hom — parser compiled from parser.hom ────────────────────────────
pub mod parser_hom {
    use crate::ast::*;
    use crate::lexer_hom::{Token, TokenKind};
    use crate::runtime::*;
    // is_upper: use the first-char-only version from chars, not the all-chars
    // version from std/str.rs (which replaced is_upper_first_str).
    use crate::runtime::chars::is_upper;
    include!(concat!(env!("OUT_DIR"), "/parser.rs"));
}

// ── lexer_hom — tokeniser compiled from lexer.hom ────────────────────────────
pub mod lexer_hom {
    use crate::ast::*;
    use crate::runtime::*;
    // chars helpers (is_alpha/is_digit/is_alnum/is_whitespace/is_newline)
    // — pulled from the runtime sub-module since std/str.rs in runtime root
    // also exports is_alpha/is_digit/is_alnum with different (all-char) semantics.
    use crate::runtime::chars::{is_alnum, is_alpha, is_digit, is_newline, is_whitespace};
    include!(concat!(env!("OUT_DIR"), "/lexer.rs"));
}

// ── sema_hom — semantic analysis compiled from sema.hom ──────────────────────
pub mod sema_hom {
    use crate::ast::*;
    use crate::codegen_hom::{
        expr_break_value, expr_for_final, expr_if_else_expr, expr_if_else_stmts, expr_if_has_else,
        expr_slice_from, expr_slice_step, expr_slice_to, expr_while_final,
    };
    use crate::runtime::*;
    include!(concat!(env!("OUT_DIR"), "/sema.rs"));
}

// ── codegen_hom — code generator compiled from codegen.hom ───────────────────
pub mod codegen_hom {
    use crate::ast::*;
    use crate::runtime::*;
    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}

// ── resolver_hom — resolver compiled from resolver.hom ───────────────────────
pub mod resolver_hom {
    use crate::ast::*;
    use crate::runtime::*;
    include!(concat!(env!("OUT_DIR"), "/resolver.rs"));
}

// ── main_hom — CLI entry point compiled from main.hom ────────────────────────
#[allow(clippy::println_empty_string)]
pub mod main_hom {
    use crate::ast::*;
    use crate::runtime::*;
    use crate::{
        builtin_rs, codegen_hom, embedded_rs, lexer_hom, parser_hom, resolver_hom, sema_hom,
    };
    include!(concat!(env!("OUT_DIR"), "/main.rs"));
}

// ── Unit tests for hom-std runtime modules ──────────────────────────────────
// Tests are in tests/std-tests/, runtime source in hom-std/.
#[cfg(test)]
mod hom_tests {
    // heap.rs and re.rs are included in crate::runtime — use their public fns directly.
    use crate::runtime::*;

    include!("../tests/std-tests/test_heap.rs");
    include!("../tests/std-tests/test_re.rs");

    // chars.rs, str_ext.rs, dict.rs are NOT in runtime (embedded-only modules).
    // Include the source file first to get the function definitions, then the tests.
    macro_rules! embed_test_mod {
        ($name:ident) => {
            mod $name {
                include!(concat!("../hom-std/", stringify!($name), ".rs"));
                include!(concat!(
                    "../tests/std-tests/test_",
                    stringify!($name),
                    ".rs"
                ));
            }
        };
    }
    embed_test_mod!(chars);
    embed_test_mod!(str_ext);
    embed_test_mod!(dict);
    embed_test_mod!(set);
    embed_test_mod!(path);
    embed_test_mod!(fs);
}

// ── Embedded runtime library content ─────────────────────────────────────────
// builtin_rs() is used by main_hom::preamble() to get the builtin.rs content.
// include_str! resolves relative to this file (src/lib.rs → src/hom/builtin.rs).
pub fn builtin_rs() -> &'static str {
    include_str!("../hom-std/builtin.rs")
}

// resolver.rs calls embedded_rs() from the crate root, so it must be pub here.
pub fn embedded_rs(name: &str) -> Option<String> {
    match name {
        "std" => {
            let mod_rs: String = include_str!("../hom-std/std/mod.rs")
                .lines()
                .filter(|l| !l.trim().starts_with("include!("))
                .collect::<Vec<_>>()
                .join("\n");
            Some(format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                mod_rs,
                include_str!("../hom-std/std/str.rs"),
                include_str!("../hom-std/std/math.rs"),
                include_str!("../hom-std/std/collection.rs"),
                include_str!("../hom-std/std/dict.rs"),
                include_str!("../hom-std/std/stack.rs"),
                include_str!("../hom-std/std/deque.rs"),
                include_str!("../hom-std/std/io.rs"),
            ))
        }
        "re" => Some(include_str!("../hom-std/re.rs").to_string()),
        "heap" => Some(include_str!("../hom-std/heap.rs").to_string()),
        "chars" => Some(include_str!("../hom-std/chars.rs").to_string()),
        "str_ext" => Some(include_str!("../hom-std/str_ext.rs").to_string()),
        // dict/set re-import HashMap/HashSet for their standalone unit tests,
        // but builtin.rs already brings both into scope via
        // `use std::collections::{HashMap, HashSet};`. Strip the redundant
        // imports at embed time so `use dict` / `use set` don't collide.
        "dict" => Some(strip_collections_imports(include_str!(
            "../hom-std/dict.rs"
        ))),
        "set" => Some(strip_collections_imports(include_str!("../hom-std/set.rs"))),
        "path" => Some(include_str!("../hom-std/path.rs").to_string()),
        "fs" => Some(include_str!("../hom-std/fs.rs").to_string()),
        _ => None,
    }
}

fn strip_collections_imports(src: &str) -> String {
    src.lines()
        .filter(|l| {
            let t = l.trim();
            t != "use std::collections::HashMap;" && t != "use std::collections::HashSet;"
        })
        .collect::<Vec<_>>()
        .join("\n")
}
