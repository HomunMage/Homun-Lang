/// Multi-file dependency resolver for Homun.
///
/// Performs DFS on `use foo` statements, resolving them against one of four
/// candidates in the same directory (exactly one must exist):
///
///   1. foo/mod.hom  — Homun folder namespace (compile recursively)
///   2. foo.hom      — single Homun file (compile and inline)
///   3. foo/mod.rs   — Rust folder namespace (read & inline content)
///   4. foo.rs       — single Rust file (read & inline content)
///
/// If 0 match → fall through to Rust `use` pass-through.
/// If 2+ match → compile error (ambiguous).
///
/// .rs deps are fully expanded: any `include!("...")` within them is
/// recursively resolved and inlined, so the final output is self-contained.
use crate::ast::*;
use crate::{codegen_hom, embedded_rs, lexer, parser, sema_hom};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A single compiled file's output plus its exported names.
pub struct ResolvedFile {
    pub path: PathBuf,
    pub rust_code: String,
    pub exports: HashSet<String>,
}

/// Result of resolving an entire dependency graph.
pub struct ResolvedProgram {
    /// Compiled fragments in topological order (leaves first).
    pub files: Vec<ResolvedFile>,
}

/// Three-color DFS state for cycle detection.
#[derive(Clone, Copy, PartialEq)]
enum Color {
    White,
    Gray,
    Black,
}

/// Result of searching for a dependency named `foo` in a directory.
enum Found {
    /// foo/mod.hom or foo.hom
    Hom(PathBuf),
    /// foo/mod.rs or foo.rs — carries the absolute path
    Rs(PathBuf),
}

struct Resolver {
    color: HashMap<PathBuf, Color>,
    stack: Vec<PathBuf>,
    files: Vec<ResolvedFile>,
    /// Names of `use` targets that were resolved as .hom (so codegen skips them).
    resolved_hom_names: HashSet<String>,
    /// Map from `use` target name to its fully-expanded .rs content.
    resolved_rs_content: HashMap<String, String>,
    /// When true, embedded runtime libraries (std, re, heap, chars) are NOT
    /// inlined — they are emitted as plain Rust `use` statements instead.
    /// Used for module compilation where the parent crate provides runtime.
    skip_embed: bool,
}

impl Resolver {
    fn new(skip_embed: bool) -> Self {
        Self {
            color: HashMap::new(),
            stack: Vec::new(),
            files: Vec::new(),
            resolved_hom_names: HashSet::new(),
            resolved_rs_content: HashMap::new(),
            skip_embed,
        }
    }

    /// Search for dependency `name` in `dir`. Returns Found variant or None.
    /// Errors if 2+ candidates exist (ambiguous).
    fn find_dep(dir: &Path, name: &str) -> Result<Option<Found>, String> {
        let mut candidates: Vec<(String, Found)> = Vec::new();

        // 1. foo/mod.hom
        let folder_mod_hom = dir.join(name).join("mod.hom");
        if folder_mod_hom.exists() {
            candidates.push((format!("{}/mod.hom", name), Found::Hom(folder_mod_hom)));
        }

        // 2. foo.hom
        let flat_hom = dir.join(format!("{}.hom", name));
        if flat_hom.exists() {
            candidates.push((format!("{}.hom", name), Found::Hom(flat_hom)));
        }

        // 3. foo/mod.rs
        let folder_mod_rs = dir.join(name).join("mod.rs");
        if folder_mod_rs.exists() {
            candidates.push((format!("{}/mod.rs", name), Found::Rs(folder_mod_rs)));
        }

        // 4. foo.rs
        let flat_rs = dir.join(format!("{}.rs", name));
        if flat_rs.exists() {
            candidates.push((format!("{}.rs", name), Found::Rs(flat_rs)));
        }

        match candidates.len() {
            0 => Ok(None),
            1 => Ok(Some(candidates.into_iter().next().unwrap().1)),
            _ => {
                let names: Vec<String> = candidates.iter().map(|(n, _)| n.clone()).collect();
                Err(format!(
                    "Ambiguous import '{}': found multiple candidates: {}",
                    name,
                    names.join(", ")
                ))
            }
        }
    }

    fn resolve_file(&mut self, path: &Path) -> Result<HashSet<String>, String> {
        let canonical = std::fs::canonicalize(path)
            .map_err(|e| format!("Cannot resolve path {}: {}", path.display(), e))?;

        match self.color.get(&canonical).copied().unwrap_or(Color::White) {
            Color::Black => {
                if let Some(rf) = self.files.iter().find(|f| f.path == canonical) {
                    return Ok(rf.exports.clone());
                }
                Ok(HashSet::new())
            }
            Color::Gray => {
                let cycle_desc: Vec<String> = self
                    .stack
                    .iter()
                    .skip_while(|p| **p != canonical)
                    .map(|p| p.display().to_string())
                    .collect();
                Err(format!(
                    "Import cycle detected: {} -> {}",
                    cycle_desc.join(" -> "),
                    canonical.display()
                ))
            }
            Color::White => {
                self.color.insert(canonical.clone(), Color::Gray);
                self.stack.push(canonical.clone());

                let source = std::fs::read_to_string(&canonical)
                    .map_err(|e| format!("Cannot read {}: {}", canonical.display(), e))?;
                let tokens = lexer::lex(&source)
                    .map_err(|e| format!("{}: Lex error: {}", canonical.display(), e))?;
                let ast = parser::parse(tokens)
                    .map_err(|e| format!("{}: Parse error: {}", canonical.display(), e))?;

                let parent_dir = canonical
                    .parent()
                    .ok_or_else(|| format!("No parent dir for {}", canonical.display()))?;
                let mut imported_names: HashSet<String> = HashSet::new();
                let mut has_rs_dep = false;

                for stmt in &ast {
                    if let Stmt::Use(path_segs) = stmt
                        && path_segs.len() == 1
                    {
                        let dep_name = &path_segs[0];
                        match Self::find_dep(parent_dir, dep_name)? {
                            Some(Found::Hom(dep_path)) => {
                                let dep_exports = self.resolve_file(&dep_path)?;
                                imported_names.extend(dep_exports);
                                self.resolved_hom_names.insert(dep_name.clone());
                            }
                            Some(Found::Rs(rs_path)) => {
                                let content = expand_rs_file(&rs_path)?;
                                self.resolved_rs_content.insert(dep_name.clone(), content);
                                has_rs_dep = true;
                            }
                            None => {
                                // Check embedded runtime libraries before falling through.
                                if !self.skip_embed {
                                    if let Some(content) = embedded_rs(dep_name) {
                                        self.resolved_rs_content.insert(dep_name.clone(), content);
                                        has_rs_dep = true;
                                    }
                                } else {
                                    // In skip_embed mode, embedded libs are still known
                                    // to sema (skip undef checks). Mark as resolved so
                                    // codegen drops the `use` statement entirely.
                                    if embedded_rs(dep_name).is_some() {
                                        has_rs_dep = true;
                                        self.resolved_hom_names.insert(dep_name.clone());
                                    }
                                }
                                // Otherwise: not a local dep — will be emitted as Rust `use`.
                            }
                        }
                    }
                }

                // Semantic analysis with imported names visible.
                // If there are .rs deps, skip undefined checks (can't introspect .rs).
                let sema_errs = if has_rs_dep {
                    sema_hom::sema_analyze_skip_undef(
                        ast.clone(),
                        imported_names.iter().cloned().collect(),
                    )
                } else {
                    sema_hom::sema_analyze(ast.clone(), imported_names.iter().cloned().collect())
                };
                if !sema_errs.is_empty() {
                    return Err(format!(
                        "{}: Semantic errors:\n{}",
                        canonical.display(),
                        sema_errs.join("\n")
                    ));
                }

                // Codegen, providing both resolved .hom names and .rs content.
                let rust_code = codegen_hom::codegen_program_with_resolved(
                    ast.clone(),
                    self.resolved_hom_names.clone(),
                    self.resolved_rs_content.clone(),
                );

                let exports: HashSet<String> = ast
                    .iter()
                    .filter_map(|s| match s {
                        Stmt::Bind(n, _) | Stmt::StructDef(n, _) | Stmt::EnumDef(n, _) => {
                            Some(n.clone())
                        }
                        _ => None,
                    })
                    .collect();

                self.stack.pop();
                self.color.insert(canonical.clone(), Color::Black);

                self.files.push(ResolvedFile {
                    path: canonical,
                    rust_code,
                    exports: exports.clone(),
                });

                Ok(exports)
            }
        }
    }
}

/// Resolve all dependencies starting from `entry_path` and return
/// compiled fragments in topological order (leaves first).
pub fn resolve(entry_path: &Path) -> Result<ResolvedProgram, String> {
    let mut resolver = Resolver::new(false);
    resolver.resolve_file(entry_path)?;
    Ok(ResolvedProgram {
        files: resolver.files,
    })
}

/// Like `resolve`, but skips embedding runtime libraries (std, re, heap, chars).
/// Used for module compilation where the parent crate provides runtime.
pub fn resolve_module(entry_path: &Path) -> Result<ResolvedProgram, String> {
    let mut resolver = Resolver::new(true);
    resolver.resolve_file(entry_path)?;
    Ok(ResolvedProgram {
        files: resolver.files,
    })
}

/// Read a .rs file and recursively expand any `include!("...")` lines
/// relative to the file's directory. Returns fully self-contained Rust source.
fn expand_rs_file(path: &Path) -> Result<String, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let dir = path
        .parent()
        .ok_or_else(|| format!("No parent dir for {}", path.display()))?;
    expand_rs_includes(&content, dir)
}

/// Replace all `include!("filename");` in `source` with the expanded
/// content of that file, resolved relative to `base_dir`.
fn expand_rs_includes(source: &str, base_dir: &Path) -> Result<String, String> {
    let mut output = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(inc_path) = parse_include_line(trimmed) {
            let full_path = base_dir.join(inc_path);
            let expanded = expand_rs_file(&full_path)?;
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
fn parse_include_line(line: &str) -> Option<&str> {
    let s = line.strip_prefix("include!(\"")?;
    let s = s.strip_suffix("\");")?;
    Some(s)
}
