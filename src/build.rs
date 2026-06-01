use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Prefer HOMUN_VERSION env (set by Docker/CI), fall back to git tag, then "dev".
    let version = env::var("HOMUN_VERSION")
        .ok()
        .filter(|s| !s.is_empty() && s != "dev")
        .or_else(|| {
            Command::new("git")
                .args(["describe", "--tags", "--always"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "dev".to_string());

    println!("cargo:rustc-env=HOMUN_VERSION={}", version);

    // Generate runtime.rs — Homun's builtin + std + re + heap runtime.
    generate_runtime();

    // Compile .hom files → .rs into OUT_DIR (inside target/).
    // Generated .rs never pollute src/. cargo clean removes everything.
    compile_hom_files();
}

/// Generate runtime.rs in OUT_DIR by concatenating .rs files from src/hom/
/// (homun-std submodule). No homunc needed — just cat the .rs files together.
fn generate_runtime() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let runtime_path = out_dir.join("runtime.rs");
    let hom = PathBuf::from("hom-std");

    // builtin.rs — macros (range!, len!, filter!, map!, dict!, set!, slice!, homun_in!),
    // traits (HomunIndex, HomunLen, HomunContains)
    let builtin =
        std::fs::read_to_string(hom.join("builtin.rs")).expect("hom-std/builtin.rs not found");

    // std/ — standard library (str, math, collection, dict, stack, deque, io)
    // Read mod.rs but strip include!() lines (we inline the sub-files directly)
    let std_mod: String = std::fs::read_to_string(hom.join("std/mod.rs"))
        .unwrap()
        .lines()
        .filter(|l| !l.trim().starts_with("include!("))
        .collect::<Vec<_>>()
        .join("\n");
    let std_str = std::fs::read_to_string(hom.join("std/str.rs")).unwrap();
    let std_math = std::fs::read_to_string(hom.join("std/math.rs")).unwrap();
    let std_collection = std::fs::read_to_string(hom.join("std/collection.rs")).unwrap();
    let std_dict = std::fs::read_to_string(hom.join("std/dict.rs")).unwrap();
    let std_stack = std::fs::read_to_string(hom.join("std/stack.rs")).unwrap();
    let std_deque = std::fs::read_to_string(hom.join("std/deque.rs")).unwrap();
    let std_io = std::fs::read_to_string(hom.join("std/io.rs")).unwrap();

    // re.rs — regex helpers
    let re = std::fs::read_to_string(hom.join("re.rs")).unwrap();
    // heap.rs — BinaryHeap helpers
    let heap = std::fs::read_to_string(hom.join("heap.rs")).unwrap();
    // path.rs — path utilities
    let path = std::fs::read_to_string(hom.join("path.rs")).unwrap();
    // fs.rs — filesystem utilities
    let fs = std::fs::read_to_string(hom.join("fs.rs")).unwrap();
    // chars.rs — character classification (used by lexer.hom via `use chars`)
    let chars = std::fs::read_to_string(hom.join("chars.rs")).unwrap();

    let code = format!(
        "// ── builtin ────────────────────────────────────────────────\n\
         {builtin}\n\n\
         // ── std ────────────────────────────────────────────────────\n\
         {std_mod}\n{std_str}\n{std_math}\n{std_collection}\n{std_dict}\n{std_stack}\n{std_deque}\n{std_io}\n\n\
         // ── re ─────────────────────────────────────────────────────\n\
         {re}\n\n\
         // ── heap ───────────────────────────────────────────────────\n\
         {heap}\n\n\
         // ── path ───────────────────────────────────────────────────\n\
         {path}\n\n\
         // ── fs ─────────────────────────────────────────────────────\n\
         {fs}\n\n\
         // ── chars (sub-module — re-exported to avoid name clashes\n\
         //   with std/str.rs's is_alpha/is_digit/is_alnum) ──────────\n\
         pub mod chars {{\n{chars}\n}}\n"
    );

    // Strip #[cfg(test)] mod tests { ... } blocks from the concatenated runtime
    // to avoid duplicate `mod tests` errors when all files are in one module.
    let code = strip_test_modules(&code);

    std::fs::write(&runtime_path, &code).unwrap();
    println!(
        "cargo:warning=Generated runtime.rs ({} bytes) from hom-std/",
        code.len()
    );

    // Rerun if any hom runtime file changes
    println!("cargo:rerun-if-changed=hom-std/builtin.rs");
    println!("cargo:rerun-if-changed=hom-std/std/mod.rs");
    println!("cargo:rerun-if-changed=hom-std/re.rs");
    println!("cargo:rerun-if-changed=hom-std/heap.rs");
    println!("cargo:rerun-if-changed=hom-std/path.rs");
    println!("cargo:rerun-if-changed=hom-std/fs.rs");
    println!("cargo:rerun-if-changed=hom-std/chars.rs");
}

fn find_homunc() -> PathBuf {
    let local = PathBuf::from(".tmp/homunc");
    if local.exists() {
        return local;
    }
    // Try PATH
    if Command::new("homunc").arg("--version").output().is_ok() {
        return PathBuf::from("homunc");
    }
    // Download from GitHub releases
    std::fs::create_dir_all(".tmp").unwrap();
    let url = "https://github.com/homun-lang/homun/releases/latest/download/homunc-linux-x86_64";
    let status = Command::new("wget")
        .args(["-q", url, "-O", local.to_str().unwrap()])
        .status();
    if let Ok(s) = status
        && s.success()
    {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&local, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        println!("cargo:warning=Downloaded homunc to .tmp/homunc");
        return local;
    }
    panic!("Cannot find or download homunc");
}

fn compile_hom_files() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Watch the src/ directory so that adding a new .hom file triggers a rebuild.
    println!("cargo:rerun-if-changed=src");

    let Ok(entries) = std::fs::read_dir("src") else {
        return;
    };

    // Collect .hom files first; skip find_homunc() if there are none.
    let hom_entries: Vec<_> = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "hom"))
        .collect();

    if hom_entries.is_empty() {
        return;
    }

    let homunc = find_homunc();

    println!("cargo:rerun-if-changed=src/ast.hom");

    for entry in hom_entries {
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let rs_path = out_dir.join(format!("{}.rs", stem));

        // Only recompile if .hom or its _imp.rs is newer than the generated .rs
        let imp_path = PathBuf::from(format!("src/{}_imp.rs", stem));
        let rs_mtime = std::fs::metadata(&rs_path)
            .ok()
            .and_then(|m| m.modified().ok());
        let hom_newer = rs_mtime.is_none()
            || std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|hom_mtime| hom_mtime > rs_mtime.unwrap())
                .unwrap_or(true);
        let imp_newer = imp_path.exists()
            && std::fs::metadata(&imp_path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|imp_mtime| rs_mtime.map(|rs| imp_mtime > rs).unwrap_or(true))
                .unwrap_or(false);
        let needs_compile = !rs_path.exists() || hom_newer || imp_newer;
        if imp_path.exists() {
            println!("cargo:rerun-if-changed={}", imp_path.display());
        }

        if needs_compile {
            let mut cmd_args = vec!["--module".to_string()];
            if stem != "ast" {
                let ast_path = PathBuf::from("src/ast.hom");
                if ast_path.exists() {
                    cmd_args.push("--import".to_string());
                    cmd_args.push(ast_path.to_string_lossy().into_owned());
                }
            }
            if stem == "parser" {
                let lexer_path = PathBuf::from("src/lexer.hom");
                if lexer_path.exists() {
                    cmd_args.push("--import".to_string());
                    cmd_args.push(lexer_path.to_string_lossy().into_owned());
                }
            }
            cmd_args.push(path.to_string_lossy().into_owned());
            cmd_args.push("-o".to_string());
            cmd_args.push(rs_path.to_string_lossy().into_owned());

            let status = Command::new(&homunc).args(&cmd_args).status();
            match status {
                Ok(s) if s.success() => {
                    if let Ok(content) = std::fs::read_to_string(&rs_path) {
                        let mut cleaned = strip_test_modules(&content);
                        if stem == "ast" {
                            cleaned = fix_cross_enum_overboxing(&cleaned);
                        }
                        let _ = std::fs::write(&rs_path, cleaned);
                    }
                    println!(
                        "cargo:warning=Compiled {} -> {}",
                        path.display(),
                        rs_path.display()
                    );
                }
                Ok(s) => {
                    println!(
                        "cargo:warning=homunc failed for {} (exit code {:?})",
                        path.display(),
                        s.code()
                    );
                }
                Err(e) => {
                    println!("cargo:warning=homunc error for {}: {}", path.display(), e);
                }
            }
        }
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

/// Fix cross-enum over-boxing in generated ast.rs.
///
/// Some bootstrap compiler versions register ALL enum names as recursive in a
/// pre-pass, causing `codegen_type_variant_field` to wrap cross-enum references
/// in `Box<>` (e.g. `Box<Expr>` inside `Stmt`, `Box<BinOp>` inside `Expr`).
/// Only self-recursive references should be boxed (e.g. `Box<Expr>` inside `Expr`).
///
/// This strips the over-boxing so that the generated types match the auto-wrap
/// and auto-deref behavior (which only handles self-recursive).
fn fix_cross_enum_overboxing(src: &str) -> String {
    let enum_names: &[&str] = &[
        "BinOp", "UnOp", "TypeExpr", "LValue", "Pat", "Expr", "Stmt", "MatchArm",
    ];
    let mut result = String::with_capacity(src.len());
    let mut current_enum: Option<&str> = None;
    let mut depth: i32 = 0;

    for line in src.lines() {
        let trimmed = line.trim();
        if current_enum.is_none()
            && let Some(rest) = trimmed.strip_prefix("pub enum ")
            && let Some(name_end) = rest.find(|c: char| !c.is_alphanumeric() && c != '_')
        {
            let name = &rest[..name_end];
            if enum_names.contains(&name) {
                current_enum = Some(name);
                depth = 0;
            }
        }

        if current_enum.is_some() {
            depth += line.chars().filter(|&c| c == '{').count() as i32;
            depth -= line.chars().filter(|&c| c == '}').count() as i32;
        }

        let output_line = if let Some(self_name) = current_enum {
            let mut fixed = line.to_string();
            for &en in enum_names {
                if en == self_name {
                    continue;
                }
                let boxed = format!("Box<{en}>");
                let opt_boxed = format!("Option<Box<{en}>>");
                let opt_plain = format!("Option<{en}>");
                fixed = fixed.replace(&opt_boxed, &opt_plain);
                fixed = fixed.replace(&boxed, en);
            }
            fixed
        } else {
            line.to_string()
        };

        result.push_str(&output_line);
        result.push('\n');

        if current_enum.is_some() && depth <= 0 {
            current_enum = None;
        }
    }
    result
}

/// Strip `#[cfg(test)] mod tests { ... }` blocks from concatenated Rust source.
/// Handles nested braces correctly by counting brace depth.
fn strip_test_modules(src: &str) -> String {
    let mut result = String::with_capacity(src.len());
    let mut lines = src.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed == "#[cfg(test)]" {
            // Peek: if next non-empty line starts with "mod tests", skip the whole block
            if let Some(&next) = lines.peek()
                && next.trim().starts_with("mod tests")
            {
                // Skip the #[cfg(test)] line and the mod tests { ... } block
                let mod_line = lines.next().unwrap();
                // Count braces to find the matching close
                let mut depth: i32 = mod_line.chars().filter(|&c| c == '{').count() as i32
                    - mod_line.chars().filter(|&c| c == '}').count() as i32;
                while depth > 0 {
                    if let Some(inner) = lines.next() {
                        depth += inner.chars().filter(|&c| c == '{').count() as i32;
                        depth -= inner.chars().filter(|&c| c == '}').count() as i32;
                    } else {
                        break;
                    }
                }
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}
