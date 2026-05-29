// dep/codegen_helpers.rs — String and type helpers for .hom codegen module.
//
// These functions are extracted from codegen.rs so that the future codegen.hom
// can call them through the dep bridge.  They contain Rust-specific logic that
// cannot be expressed in Homun:
//   - parse_interp() / escape_str() : character-level string scanning
//   - codegen_type()                : recursive TypeExpr → Rust type mapping
//   - codegen_params_mut()          : Param[] → "mut p: T" strings
//   - infer_generics()              : count un-typed params → T/U/V generic list
//
// All functions take owned values (not references) so that .hom-generated code,
// which wraps every argument in `.clone()`, can call them without type errors.

// ─── Indentation ─────────────────────────────────────────────────────────────

/// Returns `n * 4` spaces as an indentation string.
pub fn ind(n: i32) -> String {
    " ".repeat((n * 4) as usize)
}

/// Concatenate two `Vec<String>` values (helper for codegen.hom list concat).
pub fn vec_extend_strings(mut a: Vec<String>, b: Vec<String>) -> Vec<String> {
    a.extend(b);
    a
}

// ─── Utilities ───────────────────────────────────────────────────────────────

/// Returns `true` if the first character of `s` is an ASCII uppercase letter.
/// Used in codegen to distinguish enum variants (PascalCase) from struct fields.
pub fn is_upper_first(s: String) -> bool {
    s.chars().next().is_some_and(|c| c.is_uppercase())
}

// ─── Homun macro names ───────────────────────────────────────────────────────

/// Names of Homun builtins that are emitted as Rust macros (`name!(...)`)
/// rather than regular function calls.
pub const HOMUN_MACROS: &[&str] = &[
    "range", "len", "filter", "map", "reduce", "slice", "dict", "set",
];

/// Returns `true` if `name` is a Homun macro name.
pub fn is_homun_macro(name: String) -> bool {
    HOMUN_MACROS.contains(&name.as_str())
}

// ─── Self-recursive type registry ───────────────────────────────────────────

thread_local! {
    static SELF_RECURSIVE_TYPES: std::cell::RefCell<std::collections::HashSet<String>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
}

/// Mark `name` as a self-recursive type for the duration of its variant emission.
pub fn register_recursive_type(name: String) {
    SELF_RECURSIVE_TYPES.with(|s| s.borrow_mut().insert(name));
}

/// Clear the self-recursive type registry after variant emission is done.
pub fn clear_recursive_types() {
    SELF_RECURSIVE_TYPES.with(|s| s.borrow_mut().clear());
}

/// Returns true if `name` is currently registered as a self-recursive type.
pub fn is_self_recursive_type(name: String) -> bool {
    SELF_RECURSIVE_TYPES.with(|s| s.borrow().contains(&name))
}

// ─── Variant field-type registry (for Box<T> auto-deref in match patterns) ───

thread_local! {
    static VARIANT_FIELD_TYPES: std::cell::RefCell<std::collections::HashMap<String, Vec<TypeExpr>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Register field types for a variant keyed by `EnumName.VariantName`.
pub fn register_variant_field_types(qual: String, fields: Vec<TypeExpr>) {
    VARIANT_FIELD_TYPES.with(|s| {
        s.borrow_mut().insert(qual, fields);
    });
}

/// Look up registered field types for `EnumName.VariantName`. Empty if unknown.
pub fn variant_field_types_get(qual: String) -> Vec<TypeExpr> {
    VARIANT_FIELD_TYPES.with(|s| s.borrow().get(&qual).cloned().unwrap_or_default())
}

/// Returns true if a variant has been registered.
pub fn variant_field_types_known(qual: String) -> bool {
    VARIANT_FIELD_TYPES.with(|s| s.borrow().contains_key(&qual))
}

// ─── Thread-local variable registry ─────────────────────────────────────────

thread_local! {
    static THREAD_LOCAL_VARS: std::cell::RefCell<std::collections::HashSet<String>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
}

/// Register `name` as a @thread_local binding.
pub fn register_thread_local_var(name: String) {
    THREAD_LOCAL_VARS.with(|s| s.borrow_mut().insert(name));
}

/// Returns true if `name` was declared as a @thread_local binding.
pub fn is_thread_local_var(name: String) -> bool {
    THREAD_LOCAL_VARS.with(|s| s.borrow().contains(&name))
}

/// Clear the thread-local variable registry (call at start of each compilation).
pub fn clear_thread_local_vars() {
    THREAD_LOCAL_VARS.with(|s| s.borrow_mut().clear());
}

// ─── Preamble helpers ────────────────────────────────────────────────────────

/// Format the generic type-parameter clause for a function.
/// Returns `"<T: Clone, U: Clone>"` when non-empty, or `""` when empty.
pub fn generics_str(generics: Vec<String>) -> String {
    if generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", generics.join(", "))
    }
}

// ─── Expr discriminator ──────────────────────────────────────────────────────

pub fn expr_is_lambda(e: Expr) -> bool {
    matches!(e, Expr::Lambda { .. })
}

// ─── Expr accessors ───────────────────────────────────────────────────────────

pub fn expr_slice_from(e: Expr) -> Option<Expr> {
    match e {
        Expr::Slice(_, from, _, _) => from.map(|x| *x),
        _ => panic!("expr_slice_from: not Slice"),
    }
}

pub fn expr_slice_to(e: Expr) -> Option<Expr> {
    match e {
        Expr::Slice(_, _, to, _) => to.map(|x| *x),
        _ => panic!("expr_slice_to: not Slice"),
    }
}

pub fn expr_slice_step(e: Expr) -> Option<Expr> {
    match e {
        Expr::Slice(_, _, _, step) => step.map(|x| *x),
        _ => panic!("expr_slice_step: not Slice"),
    }
}

pub fn expr_if_has_else(e: Expr) -> bool {
    match e {
        Expr::If(_, _, _, ec) => ec.is_some(),
        _ => panic!("expr_if_has_else: not If"),
    }
}

pub fn expr_if_else_stmts(e: Expr) -> Vec<Stmt> {
    match e {
        Expr::If(_, _, _, Some((stmts, _))) => stmts,
        Expr::If(_, _, _, None) => panic!("expr_if_else_stmts: no else branch"),
        _ => panic!("expr_if_else_stmts: not If"),
    }
}

pub fn expr_if_else_expr(e: Expr) -> Expr {
    match e {
        Expr::If(_, _, _, Some((_, ee))) => *ee,
        Expr::If(_, _, _, None) => panic!("expr_if_else_expr: no else branch"),
        _ => panic!("expr_if_else_expr: not If"),
    }
}

pub fn expr_for_final(e: Expr) -> Option<Expr> {
    match e {
        Expr::For(_, _, _, fe) => fe.map(|x| *x),
        _ => panic!("expr_for_final: not For"),
    }
}

pub fn expr_while_final(e: Expr) -> Option<Expr> {
    match e {
        Expr::While(_, _, fe) => fe.map(|x| *x),
        _ => panic!("expr_while_final: not While"),
    }
}

pub fn expr_break_value(e: Expr) -> Option<Expr> {
    match e {
        Expr::Break(v) => v.map(|x| *x),
        _ => panic!("expr_break_value: not Break"),
    }
}

pub fn expr_lambda_params(e: Expr) -> Vec<Param> {
    match e {
        Expr::Lambda { params, .. } => params,
        _ => panic!("expr_lambda_params: not Lambda"),
    }
}

pub fn expr_lambda_stmts(e: Expr) -> Vec<Stmt> {
    match e {
        Expr::Lambda { stmts, .. } => stmts,
        _ => panic!("expr_lambda_stmts: not Lambda"),
    }
}

pub fn expr_lambda_final(e: Expr) -> Expr {
    match e {
        Expr::Lambda { final_expr, .. } => *final_expr,
        _ => panic!("expr_lambda_final: not Lambda"),
    }
}

pub fn expr_lambda_ret_ty(e: Expr) -> Option<TypeExpr> {
    match e {
        Expr::Lambda { ret_ty, .. } => ret_ty,
        _ => panic!("expr_lambda_ret_ty: not Lambda"),
    }
}

pub fn expr_lambda_void_mark(e: Expr) -> Option<TypeExpr> {
    match e {
        Expr::Lambda { void_mark, .. } => void_mark,
        _ => panic!("expr_lambda_void_mark: not Lambda"),
    }
}

pub fn expr_lambda_generics(e: Expr) -> Vec<String> {
    match e {
        Expr::Lambda { generics, .. } => generics,
        _ => panic!("expr_lambda_generics: not Lambda"),
    }
}

pub fn expr_range_start(e: Expr) -> Option<Expr> {
    match e {
        Expr::Range(s, _, _) => s.map(|x| *x),
        _ => panic!("expr_range_start: not Range"),
    }
}

pub fn expr_range_end(e: Expr) -> Option<Expr> {
    match e {
        Expr::Range(_, end, _) => end.map(|x| *x),
        _ => panic!("expr_range_end: not Range"),
    }
}

pub fn expr_range_step(e: Expr) -> Option<Expr> {
    match e {
        Expr::Range(_, _, st) => st.map(|x| *x),
        _ => panic!("expr_range_step: not Range"),
    }
}
