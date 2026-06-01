// codegen_imp.rs — Type aliases and accessors for codegen.hom.
//
// Importing this file via `use codegen_imp` in codegen.hom sets has_rs_dep=true
// in the homunc sema checker, disabling undefined-reference checks for dep/*
// functions (scope_*, stmt_kind, expr_kind, codegen_type, etc.) and for
// runtime functions (join, push, etc.) that are available at include!() time
// in lib.rs but unknown to the homunc static checker.
//
// Type aliases:
//   RsContent = HashMap<String, String>   — resolved .rs file content map
//   HomFiles  = HashSet<String>           — resolved .hom dependency names
//
// Accessor helpers (owned-value signatures for .hom interop):
//   rs_content_get(map, key)  -> Option<String>
//   hom_files_contains(set, key) -> bool
//
// fn-signature registry:
//   fn_mut_params_insert / fn_defaults_insert / current_mut_ref_params_*
//   is_mut_ref_param / is_param_mutable_in_call / fn_defaults_get_for
//   (Logic lives in codegen.hom; these are raw storage + primitive accessors.)

pub type RsContent = std::collections::HashMap<String, String>;
pub type HomFiles = std::collections::HashSet<String>;

// ─── fn-signature registry ────────────────────────────────────────────────────

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static FN_MUT_PARAMS: RefCell<HashMap<String, Vec<bool>>> =
        RefCell::new(HashMap::new());
    static FN_DEFAULTS: RefCell<HashMap<String, Vec<Option<Expr>>>> =
        RefCell::new(HashMap::new());
    static CURRENT_MUT_REF_PARAMS: RefCell<std::collections::HashSet<String>> =
        RefCell::new(std::collections::HashSet::new());
}

pub fn fn_mut_params_insert(name: String, flags: Vec<bool>) {
    FN_MUT_PARAMS.with(|m| { m.borrow_mut().insert(name, flags); });
}

pub fn fn_defaults_insert(name: String, defaults: Vec<Option<Expr>>) {
    FN_DEFAULTS.with(|m| { m.borrow_mut().insert(name, defaults); });
}

pub fn current_mut_ref_params_clear() {
    CURRENT_MUT_REF_PARAMS.with(|m| m.borrow_mut().clear());
}

pub fn current_mut_ref_params_add(name: String) {
    CURRENT_MUT_REF_PARAMS.with(|m| { m.borrow_mut().insert(name); });
}

pub fn is_mut_ref_param(name: String) -> bool {
    CURRENT_MUT_REF_PARAMS.with(|m| m.borrow().contains(&name))
}

pub fn is_param_mutable_in_call(fn_name: String, arg_idx: i32) -> bool {
    FN_MUT_PARAMS.with(|m| {
        m.borrow()
            .get(&fn_name)
            .and_then(|flags| flags.get(arg_idx as usize).copied())
            .unwrap_or(false)
    })
}

pub fn fn_defaults_get_for(fn_name: String) -> Vec<Option<Expr>> {
    FN_DEFAULTS.with(|m| m.borrow().get(&fn_name).cloned().unwrap_or_default())
}

/// Look up a key in the rs_content map.  Returns None if absent.
pub fn rs_content_get(map: RsContent, key: String) -> Option<String> {
    map.get(&key).cloned()
}

/// Return true if the hom_files set contains the given key.
pub fn hom_files_contains(set: HomFiles, key: String) -> bool {
    set.contains(&key)
}

// ─── Generic list concat (was dep/codegen_helpers.rs) ───────────────────────

pub fn vec_extend<T>(mut a: Vec<T>, b: Vec<T>) -> Vec<T> {
    a.extend(b);
    a
}

// ─── Self-recursive type registry ───────────────────────────────────────────

thread_local! {
    static SELF_RECURSIVE_TYPES: RefCell<std::collections::HashSet<String>> =
        RefCell::new(std::collections::HashSet::new());
}

pub fn register_recursive_type(name: String) {
    SELF_RECURSIVE_TYPES.with(|s| s.borrow_mut().insert(name));
}

pub fn clear_recursive_types() {
    SELF_RECURSIVE_TYPES.with(|s| s.borrow_mut().clear());
}

pub fn is_self_recursive_type(name: String) -> bool {
    SELF_RECURSIVE_TYPES.with(|s| s.borrow().contains(&name))
}

// ─── Variant field-type registry (for Box<T> auto-deref in match patterns) ───

thread_local! {
    static VARIANT_FIELD_TYPES: RefCell<HashMap<String, Vec<TypeExpr>>> =
        RefCell::new(HashMap::new());
}

pub fn register_variant_field_types(qual: String, fields: Vec<TypeExpr>) {
    VARIANT_FIELD_TYPES.with(|s| { s.borrow_mut().insert(qual, fields); });
}

pub fn variant_field_types_get(qual: String) -> Vec<TypeExpr> {
    VARIANT_FIELD_TYPES.with(|s| s.borrow().get(&qual).cloned().unwrap_or_default())
}

pub fn variant_field_types_known(qual: String) -> bool {
    VARIANT_FIELD_TYPES.with(|s| s.borrow().contains_key(&qual))
}

// ─── Thread-local variable registry ─────────────────────────────────────────

thread_local! {
    static THREAD_LOCAL_VARS: RefCell<std::collections::HashSet<String>> =
        RefCell::new(std::collections::HashSet::new());
}

pub fn register_thread_local_var(name: String) {
    THREAD_LOCAL_VARS.with(|s| s.borrow_mut().insert(name));
}

pub fn is_thread_local_var(name: String) -> bool {
    THREAD_LOCAL_VARS.with(|s| s.borrow().contains(&name))
}

pub fn clear_thread_local_vars() {
    THREAD_LOCAL_VARS.with(|s| s.borrow_mut().clear());
}

// ─── Expr accessors (Option<Box<T>> fields that auto-deref can't peel) ──────

pub fn expr_slice_from(e: Expr) -> Option<Expr> {
    match e { Expr::Slice(_, from, _, _) => from.map(|x| *x), _ => panic!("not Slice") }
}

pub fn expr_slice_to(e: Expr) -> Option<Expr> {
    match e { Expr::Slice(_, _, to, _) => to.map(|x| *x), _ => panic!("not Slice") }
}

pub fn expr_slice_step(e: Expr) -> Option<Expr> {
    match e { Expr::Slice(_, _, _, step) => step.map(|x| *x), _ => panic!("not Slice") }
}

pub fn expr_if_has_else(e: Expr) -> bool {
    match e { Expr::If(_, _, _, ec) => ec.is_some(), _ => panic!("not If") }
}

pub fn expr_if_else_stmts(e: Expr) -> Vec<Stmt> {
    match e {
        Expr::If(_, _, _, Some((stmts, _))) => stmts,
        Expr::If(_, _, _, None) => panic!("no else branch"),
        _ => panic!("not If"),
    }
}

pub fn expr_if_else_expr(e: Expr) -> Expr {
    match e {
        Expr::If(_, _, _, Some((_, ee))) => *ee,
        Expr::If(_, _, _, None) => panic!("no else branch"),
        _ => panic!("not If"),
    }
}

pub fn expr_for_final(e: Expr) -> Option<Expr> {
    match e { Expr::For(_, _, _, fe) => fe.map(|x| *x), _ => panic!("not For") }
}

pub fn expr_while_final(e: Expr) -> Option<Expr> {
    match e { Expr::While(_, _, fe) => fe.map(|x| *x), _ => panic!("not While") }
}

pub fn expr_break_value(e: Expr) -> Option<Expr> {
    match e { Expr::Break(v) => v.map(|x| *x), _ => panic!("not Break") }
}

pub fn expr_range_start(e: Expr) -> Option<Expr> {
    match e { Expr::Range(s, _, _) => s.map(|x| *x), _ => panic!("not Range") }
}

pub fn expr_range_end(e: Expr) -> Option<Expr> {
    match e { Expr::Range(_, end, _) => end.map(|x| *x), _ => panic!("not Range") }
}

pub fn expr_range_step(e: Expr) -> Option<Expr> {
    match e { Expr::Range(_, _, st) => st.map(|x| *x), _ => panic!("not Range") }
}
