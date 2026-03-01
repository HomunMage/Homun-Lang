// resolver_imp.rs — Types and I/O helpers for resolver.hom.
//
// Importing this file via `use resolver_imp` in resolver.hom sets has_rs_dep=true
// in the homunc sema checker, disabling undefined-reference checks for dep/*
// functions and runtime functions that are available at include!() time in
// lib.rs but unknown to homunc's static checker.
//
// Re-exports:
//   ResolvedFile, ResolvedProgram  — from crate::resolver (struct defs stay in Rust)
//
// Types:
//   Found         — result of find_dep() with constructors and accessors
//   ResolverState — mutable DFS state wrapped in Rc<RefCell<>>
//
// Color is represented as String in .hom: "White" | "Gray" | "Black".
//
// I/O wrappers (String args/returns for .hom interop):
//   read_file(path)         -> Result<String, String>
//   file_exists(path)       -> bool
//
// Path helpers:
//   path_join(dir, name)    -> String
//   path_parent(path)       -> String
//   path_canonicalize(path) -> Result<String, String>
//
// HashSet<String> helpers:
//   hashset_new / hashset_insert / hashset_contains / hashset_extend /
//   hashset_to_vec / hashset_from_vec
//
// ResolvedFile constructors and accessors:
//   make_resolved_file / resolved_file_path / resolved_file_rust_code /
//   resolved_file_exports
//
// ResolvedProgram constructor and accessor:
//   make_resolved_program / resolved_program_files
//
// ResolverState helpers:
//   resolver_new / resolver_color_get / resolver_color_set /
//   resolver_stack_push / resolver_stack_pop / resolver_stack_from_canonical /
//   resolver_files_push / resolver_files_get / resolver_files_find_exports /
//   resolver_hom_names_get / resolver_hom_names_insert /
//   resolver_rs_content_get_map / resolver_rs_content_insert /
//   resolver_skip_embed
//
// Rust-side helpers kept in Rust (PathBuf-heavy operations):
//   find_dep(dir, name)               -> (String, String, String)
//   expand_rs_file(path)              -> Result<String, String>
//   expand_rs_includes(src, base_dir) -> Result<String, String>
//   parse_include_line(line)          -> Option<String>

pub use crate::resolver::{ResolvedFile, ResolvedProgram};

// ── Found — result of find_dep() ─────────────────────────────────────────────

#[derive(Clone)]
pub enum Found {
    /// .hom file path
    Hom(String),
    /// .rs file path
    Rs(String),
}

pub fn found_hom(path: String) -> Found {
    Found::Hom(path)
}

pub fn found_rs(path: String) -> Found {
    Found::Rs(path)
}

/// Returns "Hom" or "Rs".
pub fn found_kind(f: Found) -> String {
    match f {
        Found::Hom(_) => "Hom".to_string(),
        Found::Rs(_) => "Rs".to_string(),
    }
}

/// Returns the file path from the Found value.
pub fn found_path(f: Found) -> String {
    match f {
        Found::Hom(p) | Found::Rs(p) => p,
    }
}

// ── I/O helpers ───────────────────────────────────────────────────────────────

/// Read a file's contents as a String. Returns Err on failure.
pub fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Cannot read {}: {}", path, e))
}

/// True if the path exists on the filesystem.
pub fn file_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

// ── Path helpers ──────────────────────────────────────────────────────────────

/// Join a directory path and a name/subpath. Returns the joined path as String.
pub fn path_join(dir: String, name: String) -> String {
    std::path::Path::new(&dir)
        .join(&name)
        .to_string_lossy()
        .into_owned()
}

/// Return the parent directory of a path as String. Returns "" if none.
pub fn path_parent(path: String) -> String {
    std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Canonicalize a path (resolve symlinks, `..`, etc.). Returns Err if path doesn't exist.
pub fn path_canonicalize(path: String) -> Result<String, String> {
    std::fs::canonicalize(&path)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| format!("Cannot resolve path {}: {}", path, e))
}

// ── HashSet<String> helpers ───────────────────────────────────────────────────

pub fn hashset_new() -> std::collections::HashSet<String> {
    std::collections::HashSet::new()
}

pub fn hashset_insert(
    mut s: std::collections::HashSet<String>,
    val: String,
) -> std::collections::HashSet<String> {
    s.insert(val);
    s
}

pub fn hashset_contains(s: std::collections::HashSet<String>, val: String) -> bool {
    s.contains(&val)
}

pub fn hashset_extend(
    mut a: std::collections::HashSet<String>,
    b: std::collections::HashSet<String>,
) -> std::collections::HashSet<String> {
    a.extend(b);
    a
}

pub fn hashset_to_vec(s: std::collections::HashSet<String>) -> Vec<String> {
    s.into_iter().collect()
}

pub fn hashset_from_vec(v: Vec<String>) -> std::collections::HashSet<String> {
    v.into_iter().collect()
}

// ── ResolvedFile constructors and accessors ───────────────────────────────────

pub fn make_resolved_file(
    path: String,
    rust_code: String,
    exports: std::collections::HashSet<String>,
) -> ResolvedFile {
    ResolvedFile {
        path: std::path::PathBuf::from(path),
        rust_code,
        exports,
    }
}

pub fn resolved_file_path(f: ResolvedFile) -> String {
    f.path.to_string_lossy().into_owned()
}

pub fn resolved_file_rust_code(f: ResolvedFile) -> String {
    f.rust_code
}

pub fn resolved_file_exports(f: ResolvedFile) -> std::collections::HashSet<String> {
    f.exports
}

// ── ResolvedProgram constructor and accessor ──────────────────────────────────

pub fn make_resolved_program(files: Vec<ResolvedFile>) -> ResolvedProgram {
    ResolvedProgram { files }
}

pub fn resolved_program_files(p: ResolvedProgram) -> Vec<ResolvedFile> {
    p.files
}

// ── ResolverState — mutable DFS state wrapped in Rc<RefCell<>> ───────────────
//
// Color is a String: "White" (unvisited), "Gray" (in-progress), "Black" (done).
// Using Rc<RefCell<>> so .hom .clone() semantics preserve mutations.

struct ResolverInner {
    color: std::collections::HashMap<String, String>,
    stack: Vec<String>,
    files: Vec<ResolvedFile>,
    resolved_hom_names: std::collections::HashSet<String>,
    resolved_rs_content: std::collections::HashMap<String, String>,
    skip_embed: bool,
}

#[derive(Clone)]
pub struct ResolverState(std::rc::Rc<std::cell::RefCell<ResolverInner>>);

pub fn resolver_new(skip_embed: bool) -> ResolverState {
    ResolverState(std::rc::Rc::new(std::cell::RefCell::new(ResolverInner {
        color: std::collections::HashMap::new(),
        stack: Vec::new(),
        files: Vec::new(),
        resolved_hom_names: std::collections::HashSet::new(),
        resolved_rs_content: std::collections::HashMap::new(),
        skip_embed,
    })))
}

/// Get the color for a canonical path. Returns "White" if not set.
pub fn resolver_color_get(rs: ResolverState, path: String) -> String {
    rs.0.borrow()
        .color
        .get(&path)
        .cloned()
        .unwrap_or_else(|| "White".to_string())
}

/// Set the color for a canonical path. Returns the (mutated) state.
pub fn resolver_color_set(rs: ResolverState, path: String, color: String) -> ResolverState {
    rs.0.borrow_mut().color.insert(path, color);
    rs
}

/// Push a canonical path onto the DFS stack. Returns state.
pub fn resolver_stack_push(rs: ResolverState, path: String) -> ResolverState {
    rs.0.borrow_mut().stack.push(path);
    rs
}

/// Pop the top of the DFS stack. Returns state.
pub fn resolver_stack_pop(rs: ResolverState) -> ResolverState {
    rs.0.borrow_mut().stack.pop();
    rs
}

/// Return stack entries from `canonical` onwards (for cycle description).
pub fn resolver_stack_from_canonical(rs: ResolverState, canonical: String) -> Vec<String> {
    let inner = rs.0.borrow();
    inner
        .stack
        .iter()
        .skip_while(|p| **p != canonical)
        .cloned()
        .collect()
}

/// Append a ResolvedFile to the files list. Returns state.
pub fn resolver_files_push(rs: ResolverState, f: ResolvedFile) -> ResolverState {
    rs.0.borrow_mut().files.push(f);
    rs
}

/// Return a clone of the entire files list.
pub fn resolver_files_get(rs: ResolverState) -> Vec<ResolvedFile> {
    rs.0.borrow().files.clone()
}

/// Return the exports of the file with the given canonical path, or empty set.
pub fn resolver_files_find_exports(
    rs: ResolverState,
    canonical: String,
) -> std::collections::HashSet<String> {
    rs.0.borrow()
        .files
        .iter()
        .find(|f| f.path.to_string_lossy() == canonical.as_str())
        .map(|f| f.exports.clone())
        .unwrap_or_default()
}

/// Return a clone of the resolved_hom_names set.
pub fn resolver_hom_names_get(rs: ResolverState) -> std::collections::HashSet<String> {
    rs.0.borrow().resolved_hom_names.clone()
}

/// Insert a name into resolved_hom_names. Returns state.
pub fn resolver_hom_names_insert(rs: ResolverState, name: String) -> ResolverState {
    rs.0.borrow_mut().resolved_hom_names.insert(name);
    rs
}

/// Return a clone of the resolved_rs_content map.
pub fn resolver_rs_content_get_map(rs: ResolverState) -> std::collections::HashMap<String, String> {
    rs.0.borrow().resolved_rs_content.clone()
}

/// Insert a (name, content) pair into resolved_rs_content. Returns state.
pub fn resolver_rs_content_insert(
    rs: ResolverState,
    name: String,
    content: String,
) -> ResolverState {
    rs.0.borrow_mut().resolved_rs_content.insert(name, content);
    rs
}

/// Return the skip_embed flag.
pub fn resolver_skip_embed(rs: ResolverState) -> bool {
    rs.0.borrow().skip_embed
}

// ── find_dep — search for a dependency file (kept in Rust for PathBuf ops) ────
//
// Returns (kind, path, error_msg).
//   kind      = "Hom" | "Rs" | "None"
//   path      = absolute path string (empty on None or error)
//   error_msg = non-empty on ambiguity error

pub fn find_dep(dir: String, name: String) -> (String, String, String) {
    let dir_path = std::path::Path::new(&dir);
    let mut candidates: Vec<(String, String, String)> = Vec::new();

    // 1. foo/mod.hom
    let folder_mod_hom = dir_path.join(&name).join("mod.hom");
    if folder_mod_hom.exists() {
        candidates.push((
            format!("{}/mod.hom", name),
            "Hom".to_string(),
            folder_mod_hom.to_string_lossy().into_owned(),
        ));
    }

    // 2. foo.hom
    let flat_hom = dir_path.join(format!("{}.hom", name));
    if flat_hom.exists() {
        candidates.push((
            format!("{}.hom", name),
            "Hom".to_string(),
            flat_hom.to_string_lossy().into_owned(),
        ));
    }

    // 3. foo/mod.rs
    let folder_mod_rs = dir_path.join(&name).join("mod.rs");
    if folder_mod_rs.exists() {
        candidates.push((
            format!("{}/mod.rs", name),
            "Rs".to_string(),
            folder_mod_rs.to_string_lossy().into_owned(),
        ));
    }

    // 4. foo.rs
    let flat_rs = dir_path.join(format!("{}.rs", name));
    if flat_rs.exists() {
        candidates.push((
            format!("{}.rs", name),
            "Rs".to_string(),
            flat_rs.to_string_lossy().into_owned(),
        ));
    }

    match candidates.len() {
        0 => ("None".to_string(), String::new(), String::new()),
        1 => {
            let (_, kind, path) = candidates.into_iter().next().unwrap();
            (kind, path, String::new())
        }
        _ => {
            let labels: Vec<String> = candidates.iter().map(|(l, _, _)| l.clone()).collect();
            (
                "None".to_string(),
                String::new(),
                format!(
                    "Ambiguous import '{}': found multiple candidates: {}",
                    name,
                    labels.join(", ")
                ),
            )
        }
    }
}

// ── expand_rs_file / expand_rs_includes / parse_include_line ─────────────────
// Kept in Rust because they use PathBuf operations and recursion.

/// Read a .rs file and recursively inline any `include!("...")` lines.
pub fn expand_rs_file(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    let content = std::fs::read_to_string(p).map_err(|e| format!("Cannot read {}: {}", path, e))?;
    let dir = p
        .parent()
        .ok_or_else(|| format!("No parent dir for {}", path))?;
    expand_rs_includes(content, dir.to_string_lossy().into_owned())
}

/// Replace all `include!("filename");` lines in `source` with expanded file content.
pub fn expand_rs_includes(source: String, base_dir: String) -> Result<String, String> {
    let base = std::path::Path::new(&base_dir);
    let mut output = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(inc_path) = parse_include_line(trimmed.to_string()) {
            let full_path = base.join(&inc_path);
            let expanded = expand_rs_file(full_path.to_string_lossy().into_owned())?;
            output.push_str(&expanded);
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    Ok(output)
}

/// If `line` is `include!("some/file.rs");`, return `Some("some/file.rs")`.
pub fn parse_include_line(line: String) -> Option<String> {
    let s = line.strip_prefix("include!(\"")?;
    let s = s.strip_suffix("\");")?;
    Some(s.to_string())
}
