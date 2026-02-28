// dep/scope.rs — Rc<RefCell<HashSet<String>>> scope wrapper for .hom sema/codegen.
//
// Provides a mutable, reference-counted scope (set of bound names) that can be
// passed through .hom code without losing mutations.  Because .hom's codegen
// wraps every argument in `.clone()`, a plain HashSet would deep-copy on each
// call — mutations would be invisible to callers.  Using Rc<RefCell<...>> means
// `.clone()` is a cheap reference-count bump that shares the same underlying
// data.  Use `scope_clone` (deep copy) when you need an independent snapshot.
//
// IMPORTANT: uses fully-qualified `std::rc::Rc` / `std::cell::RefCell` /
// `std::collections::HashSet` rather than `use` statements to avoid E0252
// "defined multiple times" when multiple dep .rs files are inlined together by
// the homunc build system.
//
// ─── Exported API ────────────────────────────────────────────────────────────
//
//   Scope = Rc<RefCell<HashSet<String>>>
//     scope_new()                    -> Scope          (empty scope)
//     scope_contains(sc, name)       -> bool
//     scope_insert(sc, name)                           (mutate in place)
//     scope_clone(sc)                -> Scope          (deep copy, independent)
//     scope_from_list(list)          -> Scope          (build from Vec<String>)
//     scope_union(a, b)              -> Scope          (new scope = a ∪ b)
//     scope_to_list(sc)              -> Vec<String>    (snapshot as sorted list)

pub type Scope =
    std::rc::Rc<std::cell::RefCell<std::collections::HashSet<String>>>;

/// Create a new empty scope.
pub fn scope_new() -> Scope {
    std::rc::Rc::new(std::cell::RefCell::new(
        std::collections::HashSet::new(),
    ))
}

/// Return `true` if `name` is present in `sc`.
pub fn scope_contains(sc: Scope, name: String) -> bool {
    sc.borrow().contains(&name)
}

/// Insert `name` into `sc` (mutates in place; all Rc-clones see the change).
pub fn scope_insert(sc: Scope, name: String) {
    sc.borrow_mut().insert(name);
}

/// Return an independent deep copy of `sc` (new Rc with cloned HashSet data).
/// Use this when you need a child scope that should not affect the parent.
pub fn scope_clone(sc: Scope) -> Scope {
    let snapshot: std::collections::HashSet<String> = sc.borrow().clone();
    std::rc::Rc::new(std::cell::RefCell::new(snapshot))
}

/// Build a scope pre-populated from all elements of `list`.
/// Accepts `Vec<String>` or `Vec<&str>` (any `S: Into<String>`).
pub fn scope_from_list<S: Into<String>>(list: Vec<S>) -> Scope {
    let set: std::collections::HashSet<String> = list.into_iter().map(|s| s.into()).collect();
    std::rc::Rc::new(std::cell::RefCell::new(set))
}

/// Return a new independent scope containing all names from both `a` and `b`.
pub fn scope_union(a: Scope, b: Scope) -> Scope {
    let mut set: std::collections::HashSet<String> = a.borrow().clone();
    set.extend(b.borrow().clone());
    std::rc::Rc::new(std::cell::RefCell::new(set))
}

/// Return all names in `sc` as a sorted Vec<String>.
/// Sorting ensures deterministic output for tests.
pub fn scope_to_list(sc: Scope) -> Vec<String> {
    let mut v: Vec<String> = sc.borrow().iter().cloned().collect();
    v.sort();
    v
}
