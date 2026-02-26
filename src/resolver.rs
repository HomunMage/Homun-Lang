/// Multi-file dependency resolver for Homun.
///
/// Performs DFS on `use foo` statements, resolving them against `.hom` files
/// in the same directory. Produces topologically-ordered (leaves-first)
/// compiled fragments ready for concatenation.
use crate::ast::*;
use crate::{codegen, lexer, parser, sema};
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
    /// Set of module names that were resolved as .hom files.
    pub resolved_names: HashSet<String>,
}

/// Three-color DFS state for cycle detection.
#[derive(Clone, Copy, PartialEq)]
enum Color {
    /// Not yet visited.
    White,
    /// Currently on the DFS stack (cycle detection).
    Gray,
    /// Fully processed and emitted.
    Black,
}

struct Resolver {
    color: HashMap<PathBuf, Color>,
    stack: Vec<PathBuf>,
    files: Vec<ResolvedFile>,
    resolved_names: HashSet<String>,
}

impl Resolver {
    fn new() -> Self {
        Self {
            color: HashMap::new(),
            stack: Vec::new(),
            files: Vec::new(),
            resolved_names: HashSet::new(),
        }
    }

    fn resolve_file(&mut self, path: &Path) -> Result<HashSet<String>, String> {
        let canonical = std::fs::canonicalize(path)
            .map_err(|e| format!("Cannot resolve path {}: {}", path.display(), e))?;

        match self.color.get(&canonical).copied().unwrap_or(Color::White) {
            Color::Black => {
                // Already processed — return cached exports.
                if let Some(rf) = self.files.iter().find(|f| f.path == canonical) {
                    return Ok(rf.exports.clone());
                }
                Ok(HashSet::new())
            }
            Color::Gray => {
                // Cycle detected.
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

                // Read and parse the file.
                let source = std::fs::read_to_string(&canonical)
                    .map_err(|e| format!("Cannot read {}: {}", canonical.display(), e))?;
                let tokens = lexer::lex(&source)
                    .map_err(|e| format!("{}: Lex error: {}", canonical.display(), e))?;
                let ast = parser::parse(tokens)
                    .map_err(|e| format!("{}: Parse error: {}", canonical.display(), e))?;

                // Resolve dependencies first (DFS into children).
                let parent_dir = canonical
                    .parent()
                    .ok_or_else(|| format!("No parent dir for {}", canonical.display()))?;
                let mut imported_names: HashSet<String> = HashSet::new();

                for stmt in &ast {
                    if let Stmt::Use(path_segs) = stmt {
                        if path_segs.len() == 1 && path_segs[0] != "std" {
                            let dep_file = parent_dir.join(format!("{}.hom", path_segs[0]));
                            if dep_file.exists() {
                                let dep_exports = self.resolve_file(&dep_file)?;
                                imported_names.extend(dep_exports);
                                self.resolved_names.insert(path_segs[0].clone());
                            }
                        }
                    }
                }

                // Semantic analysis with imported names visible.
                sema::analyze_program_with_imports(&ast, &imported_names).map_err(|errs| {
                    let msgs: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
                    format!(
                        "{}: Semantic errors:\n{}",
                        canonical.display(),
                        msgs.join("\n")
                    )
                })?;

                // Codegen, skipping resolved .hom use statements.
                let rust_code =
                    codegen::codegen_program_with_resolved(&ast, &self.resolved_names);

                // Collect exports: top-level binds, structs, enums.
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
    let mut resolver = Resolver::new();
    resolver.resolve_file(entry_path)?;
    Ok(ResolvedProgram {
        files: resolver.files,
        resolved_names: resolver.resolved_names,
    })
}
