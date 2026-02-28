// dep/mod.rs — hand-written Rust helpers for .hom sema + codegen modules.
//
// Re-exports AST types so that .hom modules that `use dep` can access them
// directly (Stmt, Expr, Pat, TypeExpr, BinOp, UnOp, Param, MatchArm, etc.).
//
// Includes bridge files that provide free-function accessors over the Rust
// AST enums (Phase 1/2).  These files are intentionally empty in Phase 0 and
// will be filled in by tickets P1.1, P1.2, P2.1, and P2.2.
//
// IMPORTANT: uses fully-qualified `std::rc::Rc` / `std::cell::RefCell`
// throughout (not `use` statements) to avoid E0252 "defined multiple times"
// errors when multiple dep .rs files are inlined into the same module scope.

pub use crate::ast::*;

include!("ast_access.rs");
include!("scope.rs");
include!("codegen_helpers.rs");
