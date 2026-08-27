// resolver_imp.rs — I/O helpers for resolver.hom.
//
// Types ResolvedFile, Found, ResolverState are now defined in resolver.hom.
//
// Importing this file via `use resolver_imp` in resolver.hom sets has_rs_dep=true
// in the homunc sema checker, disabling undefined-reference checks for dep/*
// functions and runtime functions that are available at include!() time in
// lib.rs but unknown to homunc's static checker.
//
// Types (remain here):
//   ResolvedProgram  — resolver output type
//
// ResolvedProgram constructor and accessor:
//   make_resolved_program / resolved_program_files
//
// ResolvedFile constructors and accessors:
//   make_resolved_file / resolved_file_path / resolved_file_rust_code /
//   resolved_file_exports
//
// ResolverState constructor:
//   resolver_new
//
// HashSet<String> helpers:
//   hashset_new / hashset_insert / hashset_contains / hashset_extend /
//   hashset_to_vec / hashset_from_vec
//
// Rust-side helpers kept in Rust (PathBuf-heavy operations):
//   find_dep(dir, name)               -> (String, String, String)
//   expand_rs_file(path)              -> Result<String, String>
//   expand_rs_includes(src, base_dir) -> Result<String, String>
//   parse_include_line(line)          -> Option<String>

/// Result of resolving an entire dependency graph.
#[derive(Clone)]
pub struct ResolvedProgram {
    /// Compiled fragments in topological order (leaves first).
    pub files: Vec<ResolvedFile>,
}

// ── Return type aliases for resolver.hom ─────────────────────────────────────
// Homun generates bare `-> TypeName` return annotations. These aliases give
// resolver.hom a single-token name for each Result variant it returns.

/// Return type of resolve_file: Ok(exported names) or Err(message).
pub type ResolveFileResult = Result<Vec<String>, String>;

/// Return type of resolve / resolve_module: Ok(program) or Err(message).
pub type ResolveProgramResult = Result<ResolvedProgram, String>;

pub fn make_resolved_file(path: String, rust_code: String, exports: Vec<String>) -> ResolvedFile {
    ResolvedFile { path, rust_code, exports }
}

pub fn make_resolved_program(files: Vec<ResolvedFile>) -> ResolvedProgram {
    ResolvedProgram { files }
}

// ── --runtime-path ───────────────────────────────────────────────────────────
// `--module` output uses the Homun runtime's macros and traits (`len!`,
// `HomunLen`, ...) but has nowhere to import them from, because the consumer
// decides where the runtime lives. `--runtime-path PATH` emits `use PATH::*;`
// at the top of each module so the expansion resolves.

thread_local! {
    static RUNTIME_PATH: std::cell::RefCell<String> =
        std::cell::RefCell::new(String::new());
}

/// Set the Rust path the runtime is reachable at, e.g. `super::runtime`.
pub fn runtime_path_set(path: String) {
    RUNTIME_PATH.with(|p| *p.borrow_mut() = path);
}

/// The configured runtime path, or an empty string when none was given.
pub fn runtime_path_get() -> String {
    RUNTIME_PATH.with(|p| p.borrow().clone())
}

// ── --include search path ────────────────────────────────────────────────────
// `use foo` resolves against the importing file's own directory. `--include DIR`
// adds directories to search after it, so a host can point the compiler at
// sources it owns without copying them next to the input.

thread_local! {
    static INCLUDE_DIRS: std::cell::RefCell<Vec<String>> =
        std::cell::RefCell::new(Vec::new());
}

/// Append a directory to the search path. Called once per `--include` flag.
pub fn include_dir_add(dir: String) {
    INCLUDE_DIRS.with(|d| d.borrow_mut().push(dir));
}

/// Drop the search path, so one process can compile independent inputs.
pub fn include_dirs_clear() {
    INCLUDE_DIRS.with(|d| d.borrow_mut().clear());
}

// ── --extern registry ────────────────────────────────────────────────────────
// `--extern NAME` declares that `use NAME` refers to a module the consumer
// already provides alongside the output: read nothing, inline nothing, emit a
// Rust `use` instead.
//
// This is `skip_embed` generalised from hom-std to a named dependency. hom-std
// has no crate to reference, so it must keep being inlined; a linked Rust crate
// must not be, or every module that mentions it gets its own copy of its types.

thread_local! {
    static EXTERN_NAMES: std::cell::RefCell<std::collections::HashSet<String>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
}

/// Declare `name` external. Called once per `--extern` flag.
pub fn extern_register(name: String) {
    EXTERN_NAMES.with(|s| {
        s.borrow_mut().insert(name);
    });
}

/// Was `name` declared external?
pub fn extern_is(name: String) -> bool {
    EXTERN_NAMES.with(|s| s.borrow().contains(&name))
}

/// Drop every declaration, so one process can compile independent inputs.
pub fn extern_clear() {
    EXTERN_NAMES.with(|s| s.borrow_mut().clear());
}

// ── ResolverState constructor ────────────────────────────────────────────────
// ResolverState is defined in resolver.hom; constructor stays here for
// HashMap/HashSet/Vec initialization.

pub fn resolver_new(skip_embed: bool) -> ResolverState {
    ResolverState {
        color: std::collections::HashMap::new(),
        stack: Vec::new(),
        files: Vec::new(),
        resolved_hom_names: Vec::new(),
        resolved_rs_content: std::collections::HashMap::new(),
        skip_embed,
    }
}

// ── Vec field helpers ─────────────────────────────────────────────────────────
// homun_concat(rs.field, ...) tries to move a field from &mut ResolverState.
// Using named function calls causes homunc's codegen to add .clone() to every
// argument, avoiding the move error.

pub fn vec_push<T>(mut v: Vec<T>, item: T) -> Vec<T> {
    v.push(item);
    v
}

pub fn vec_pop<T: Clone>(v: Vec<T>) -> Vec<T> {
    v[..v.len() - 1].to_vec()
}

// ── String→String dict helpers ───────────────────────────────────────────────
// Used from resolver.hom to work around homunc's dict-field index-assignment
// limitation (dict[key] := val on struct fields generates wrong `as usize` code).

/// Return the value for key in m, or default if key not found.
pub fn str_dict_get(
    m: std::collections::HashMap<String, String>,
    key: String,
    default: String,
) -> String {
    m.get(&key).cloned().unwrap_or(default)
}

/// Insert key→val into m and return the updated map.
pub fn str_dict_set(
    mut m: std::collections::HashMap<String, String>,
    key: String,
    val: String,
) -> std::collections::HashMap<String, String> {
    m.insert(key, val);
    m
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

// ── Pipeline wrappers — called from resolver.hom ─────────────────────────────
//
// These delegate to the compiled .hom modules and crate-level functions.
// They are only valid when included inside the resolver_hom module in lib.rs
// (after R3 wiring), where crate::lexer_hom, crate::parser, etc. are defined.

/// find_dep wrapper that returns Result instead of (kind, path, err) triple.
/// Resolve `use name` against the importing file's directory, then against each
/// `--include` directory in order. An ambiguity inside any one directory is
/// still an error; the directories themselves are tried in sequence.
pub fn find_dep_result(dir: String, name: String) -> Result<(String, String), String> {
    let (kind, path, err) = find_dep(dir, name.clone());
    if !err.is_empty() {
        return Err(err);
    }
    if kind != "None" {
        return Ok((kind, path));
    }

    let dirs = INCLUDE_DIRS.with(|d| d.borrow().clone());
    for extra in dirs {
        let (kind, path, err) = find_dep(extra, name.clone());
        if !err.is_empty() {
            return Err(err);
        }
        if kind != "None" {
            return Ok((kind, path));
        }
    }
    Ok(("None".to_string(), String::new()))
}

/// Lex source text using the compiled lexer_hom module.
pub fn do_lex(source: String) -> Result<Vec<crate::lexer_hom::Token>, String> {
    crate::lexer_hom::lex(source)
}

/// Parse a token list using parser_hom (compiled from parser.hom).
pub fn do_parse(tokens: Vec<crate::lexer_hom::Token>) -> Result<Vec<Stmt>, String> {
    crate::parser_hom::parse(tokens)
}

/// Run semantic analysis via sema_hom.  If skip_undef is true, undefined
/// reference checks are suppressed (used when .rs deps are present).
pub fn do_sema(ast: Vec<Stmt>, imported_list: Vec<String>, skip_undef: bool) -> Vec<String> {
    if skip_undef {
        crate::sema_hom::sema_analyze_skip_undef(ast, imported_list)
    } else {
        crate::sema_hom::sema_analyze(ast, imported_list)
    }
}

/// Run code generation via codegen_hom.
pub fn do_codegen(
    ast: Vec<Stmt>,
    hom_names: std::collections::HashSet<String>,
    rs_content: std::collections::HashMap<String, String>,
) -> String {
    crate::codegen_hom::register_known_dep_fns();
    crate::codegen_hom::codegen_program_with_resolved(ast, hom_names, rs_content)
}

/// Look up an embedded runtime library by name.
pub fn do_embedded_rs(name: String) -> Option<String> {
    crate::embedded_rs(&name)
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
