/// Regression tests for the v0.94 fixes.
///
/// Each case was found while making a game engine's components and systems
/// live in `.hom`; see report.md for the field report they came from.
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Compile a Homun snippet with `--module`; return (success, generated Rust,
/// combined stdout+stderr). Extra flags go before the input path.
fn compile_with(
    src: &str,
    name: &str,
    flags: &[&str],
    files: &[(&str, &str)],
) -> (bool, String, String) {
    let tmp = PathBuf::from(".tmp/homun_tests").join(format!("v94_{name}"));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    for (fname, content) in files {
        fs::write(tmp.join(fname), content).unwrap();
    }
    let hom = tmp.join("input.hom");
    let rs = tmp.join("input.rs");
    fs::write(&hom, src).unwrap();

    let mut args: Vec<String> = vec!["--module".into()];
    args.extend(flags.iter().map(|f| f.to_string()));
    args.push(hom.to_str().unwrap().into());
    args.push("-o".into());
    args.push(rs.to_str().unwrap().into());

    let out = Command::new(env!("CARGO_BIN_EXE_homunc"))
        .args(&args)
        .output()
        .expect("failed to run homunc");
    let mut log = String::from_utf8_lossy(&out.stdout).into_owned();
    log.push_str(&String::from_utf8_lossy(&out.stderr));
    let generated = fs::read_to_string(&rs).unwrap_or_default();
    (out.status.success(), generated, log)
}

fn compile(src: &str, name: &str) -> (bool, String, String) {
    compile_with(src, name, &[], &[])
}

/// Compile with the input in `<tmp>/game/` and the dependency in `<tmp>/dep/`,
/// so only a search path can connect them.
fn compile_split(name: &str, flags: &[&str], dep: (&str, &str)) -> (bool, String, String) {
    let tmp = PathBuf::from(".tmp/homun_tests").join(format!("v94_{name}"));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("game")).unwrap();
    fs::create_dir_all(tmp.join("dep")).unwrap();
    fs::write(tmp.join("dep").join(dep.0), dep.1).unwrap();
    let hom = tmp.join("game").join("input.hom");
    let rs = tmp.join("game").join("input.rs");
    fs::write(&hom, SRC).unwrap();

    let mut args: Vec<String> = vec!["--module".into()];
    for f in flags {
        args.push(if *f == "@DEP" {
            tmp.join("dep").to_str().unwrap().to_string()
        } else {
            f.to_string()
        });
    }
    args.push(hom.to_str().unwrap().into());
    args.push("-o".into());
    args.push(rs.to_str().unwrap().into());

    let out = Command::new(env!("CARGO_BIN_EXE_homunc"))
        .args(&args)
        .output()
        .expect("failed to run homunc");
    let mut log = String::from_utf8_lossy(&out.stdout).into_owned();
    log.push_str(&String::from_utf8_lossy(&out.stderr));
    (
        out.status.success(),
        fs::read_to_string(&rs).unwrap_or_default(),
        log,
    )
}

// ── Braced and glob `use` paths pass through to Rust ─────────────────────────

#[test]
fn braced_use_path_passes_through() {
    let (ok, rs, log) = compile(
        "use engine::{Transform, Vec3}\nf := () -> _ { }\n",
        "braced",
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(
        rs.contains("use engine::{Transform,Vec3};"),
        "braced path should pass through verbatim, got:\n{rs}"
    );
}

#[test]
fn deep_braced_use_path_passes_through() {
    let (ok, rs, log) = compile("use a::b::c::{D, E}\nf := () -> _ { }\n", "deepbraced");
    assert!(ok, "expected success, got:\n{log}");
    assert!(rs.contains("use a::b::c::{D,E};"), "got:\n{rs}");
}

#[test]
fn glob_use_path_passes_through() {
    let (ok, rs, log) = compile("use engine::*\nf := () -> _ { }\n", "glob");
    assert!(ok, "expected success, got:\n{log}");
    assert!(rs.contains("use engine::*;"), "got:\n{rs}");
}

// ── `_`-prefixed identifiers ────────────────────────────────────────────────

#[test]
fn underscore_prefixed_loop_variable_parses() {
    let (ok, rs, log) = compile(
        "f := (xs: @[int]) -> _ { for _p in xs { print(1) } }\n",
        "usloop",
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(rs.contains("for _p in"), "got:\n{rs}");
}

#[test]
fn underscore_prefixed_binding_parses() {
    let (ok, rs, log) = compile("f := () -> int { _n := 5\n  _n }\n", "usbind");
    assert!(ok, "expected success, got:\n{log}");
    assert!(rs.contains("_n"), "got:\n{rs}");
}

/// A bare `_` must still be the wildcard, not an identifier.
#[test]
fn bare_underscore_is_still_a_wildcard() {
    let (ok, rs, log) = compile(
        "f := (n: int) -> str { match n { 1 -> \"one\"\n  _ -> \"other\" } }\n",
        "wildcard",
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(
        rs.contains("_ =>"),
        "bare _ should stay a match wildcard, got:\n{rs}"
    );
}

// ── `::` survives an attribute body ─────────────────────────────────────────

#[test]
fn attribute_path_keeps_double_colon() {
    let (ok, rs, log) = compile(
        "@derive(Clone, serde::Deserialize)\nA := struct { n: int }\n",
        "attrpath",
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(
        rs.contains("serde::Deserialize"),
        "attribute body should keep `::`, got:\n{rs}"
    );
}

// ── `=> _` is a void early return ───────────────────────────────────────────

#[test]
fn void_early_return_in_loop_emits_bare_return() {
    let (ok, rs, log) = compile(
        "A := struct { n: int }\nf := (a::A, xs: @[int]) -> _ { for x in xs { a.n := a.n + 1\n  => _ } }\n",
        "voidret",
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(rs.contains("return;"), "got:\n{rs}");
    assert!(
        !rs.contains("return _"),
        "`_` is not an expression, got:\n{rs}"
    );
}

#[test]
fn valued_early_return_still_returns_its_value() {
    let (ok, rs, log) = compile(
        "f := (n: int) -> int { if (n > 0) { => 42 }\n  0 }\n",
        "valret",
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(rs.contains("return 42"), "got:\n{rs}");
}

// ── --extern references a module instead of inlining it ─────────────────────

const SHIM: &str =
    "pub struct Transform { pub y: f32 }\npub fn action(name: String) -> bool { false }\n";
const SRC: &str = "use engine\ng := (t: Transform) -> _ { if (action(\"jump\")) { print(t.y) } }\n";

#[test]
fn without_extern_a_sibling_rs_is_inlined() {
    let (ok, rs, log) = compile_with(SRC, "noextern", &[], &[("engine.rs", SHIM)]);
    assert!(ok, "expected success, got:\n{log}");
    assert!(
        rs.contains("pub fn action"),
        "should inline the shim, got:\n{rs}"
    );
    assert!(!rs.contains("use super::engine"), "got:\n{rs}");
}

#[test]
fn with_extern_the_module_is_referenced_not_inlined() {
    let (ok, rs, log) = compile_with(
        SRC,
        "extern",
        &["--extern", "engine"],
        &[("engine.rs", SHIM)],
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(
        rs.contains("use super::engine::*;"),
        "should reference the module, got:\n{rs}"
    );
    assert!(
        !rs.contains("pub fn action"),
        "should not inline the shim, got:\n{rs}"
    );
}

/// The pass-through `use engine;` codegen would otherwise emit must be gone,
/// or the generated module fails to compile.
#[test]
fn with_extern_no_stray_pass_through_use() {
    let (ok, rs, log) = compile_with(
        SRC,
        "nostray",
        &["--extern", "engine"],
        &[("engine.rs", SHIM)],
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(
        !rs.lines().any(|l| l.trim() == "use engine;"),
        "stray pass-through use, got:\n{rs}"
    );
}

/// --extern names a dependency, so an unrelated name keeps being inlined.
#[test]
fn extern_only_affects_the_named_dependency() {
    let (ok, rs, log) = compile_with(
        SRC,
        "othername",
        &["--extern", "somethingelse"],
        &[("engine.rs", SHIM)],
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(rs.contains("pub fn action"), "got:\n{rs}");
    assert!(!rs.contains("use super::engine"), "got:\n{rs}");
}

// ── --include: the host points the compiler at sources it owns ──────────────

/// Without a search path the dependency is simply not there.
#[test]
fn dependency_outside_the_input_directory_is_not_found() {
    let (ok, _rs, log) = compile_split("incnone", &[], ("engine.rs", SHIM));
    assert!(!ok, "expected failure, got success:\n{log}");
    assert!(log.contains("undefined reference"), "got:\n{log}");
}

/// --include DIR alone pulls the dependency in as an inline copy.
#[test]
fn include_pulls_the_dependency_inline() {
    let (ok, rs, log) = compile_split("incinline", &["--include", "@DEP"], ("engine.rs", SHIM));
    assert!(ok, "expected success, got:\n{log}");
    assert!(rs.contains("pub fn action"), "should inline, got:\n{rs}");
    assert!(!rs.contains("use super::engine"), "got:\n{rs}");
}

/// --include with --extern pulls it in as a reference instead.
#[test]
fn include_with_extern_pulls_the_dependency_as_an_import() {
    let (ok, rs, log) = compile_split(
        "incextern",
        &["--include", "@DEP", "--extern", "engine"],
        ("engine.rs", SHIM),
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(
        rs.contains("use super::engine::*;"),
        "should reference, got:\n{rs}"
    );
    assert!(
        !rs.contains("pub fn action"),
        "should not inline, got:\n{rs}"
    );
}

/// The input's own directory still wins over the search path.
#[test]
fn the_inputs_own_directory_takes_precedence() {
    let tmp = PathBuf::from(".tmp/homun_tests/v94_incprec");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("game")).unwrap();
    fs::create_dir_all(tmp.join("dep")).unwrap();
    fs::write(
        tmp.join("dep").join("engine.rs"),
        "pub fn action(n: String) -> bool { true }\n",
    )
    .unwrap();
    fs::write(
        tmp.join("game").join("engine.rs"),
        "pub struct Transform { pub y: f32 }\npub fn action(name: String) -> bool { false }\n// LOCAL\n",
    )
    .unwrap();
    let hom = tmp.join("game").join("input.hom");
    fs::write(&hom, SRC).unwrap();
    let rs = tmp.join("game").join("input.rs");

    let out = Command::new(env!("CARGO_BIN_EXE_homunc"))
        .args([
            "--module",
            "--include",
            tmp.join("dep").to_str().unwrap(),
            hom.to_str().unwrap(),
            "-o",
            rs.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let generated = fs::read_to_string(&rs).unwrap();
    assert!(
        generated.contains("// LOCAL"),
        "the sibling engine.rs should win, got:\n{generated}"
    );
}

// ── --emit-runtime is a usable fragment ─────────────────────────────────────

fn emit_runtime(extras: &[&str]) -> String {
    let mut args = vec!["--emit-runtime"];
    args.extend_from_slice(extras);
    let out = Command::new(env!("CARGO_BIN_EXE_homunc"))
        .args(&args)
        .output()
        .expect("failed to run homunc");
    assert!(out.status.success(), "--emit-runtime failed");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn duplicate_fn_names(src: &str) -> Vec<String> {
    let mut seen = Vec::new();
    let mut dupes = Vec::new();
    for line in src.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("pub fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if seen.contains(&name) {
                dupes.push(name);
            } else {
                seen.push(name);
            }
        }
    }
    dupes
}

/// The default set must be includable as-is: no duplicate definitions, no
/// undeclared crate dependency, no inner attributes.
#[test]
fn default_runtime_has_no_duplicate_definitions() {
    let rt = emit_runtime(&[]);
    let dupes = duplicate_fn_names(&rt);
    assert!(dupes.is_empty(), "duplicate definitions: {dupes:?}");
}

#[test]
fn default_runtime_needs_no_external_crate() {
    let rt = emit_runtime(&[]);
    assert!(
        !rt.lines().any(|l| l.trim_start().starts_with("use regex")),
        "the default runtime must not require the regex crate"
    );
}

#[test]
fn runtime_emits_no_inner_attributes() {
    let rt = emit_runtime(&[]);
    assert!(
        !rt.lines().any(|l| l.trim_start().starts_with("#![")),
        "a fragment to include! cannot carry inner attributes"
    );
}

#[test]
fn default_runtime_carries_the_macros_module_output_uses() {
    let rt = emit_runtime(&[]);
    assert!(rt.contains("macro_rules! len"), "len! is missing");
    assert!(rt.contains("trait HomunLen"), "HomunLen is missing");
}

/// Optional modules are opt-in, and asking for `re` brings its dependency.
#[test]
fn with_adds_an_optional_module() {
    let rt = emit_runtime(&["--with", "re"]);
    assert!(rt.lines().any(|l| l.trim_start().starts_with("use regex")));
}

// ── --runtime-path ──────────────────────────────────────────────────────────

#[test]
fn runtime_path_is_imported_by_module_output() {
    let (ok, rs, log) = compile_with(
        "use std\nf := (xs: @[int]) -> int { len(xs) }\n",
        "rtpath",
        &["--runtime-path", "super::runtime"],
        &[],
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(
        rs.contains("use super::runtime::*;"),
        "module output should import the runtime, got:\n{rs}"
    );
}

#[test]
fn without_runtime_path_nothing_is_imported() {
    let (ok, rs, log) = compile(
        "use std\nf := (xs: @[int]) -> int { len(xs) }\n",
        "nortpath",
    );
    assert!(ok, "expected success, got:\n{log}");
    assert!(!rs.contains("use super::runtime"), "got:\n{rs}");
}

/// Both prefixes are emitted, runtime first, when a game is built the way the
/// engine builds one.
#[test]
fn runtime_path_and_extern_compose() {
    let (ok, rs, log) = compile_with(
        SRC,
        "rtextern",
        &["--runtime-path", "super::runtime", "--extern", "engine"],
        &[("engine.rs", SHIM)],
    );
    assert!(ok, "expected success, got:\n{log}");
    let rt = rs.find("use super::runtime::*;").expect("runtime import");
    let ext = rs.find("use super::engine::*;").expect("extern import");
    assert!(rt < ext, "runtime should come first, got:\n{rs}");
}

// ── Fixed-size arrays index and measure like lists ──────────────────────────

const ARRAY_SHIM: &str =
    "#[derive(Clone, Copy, Debug, Default)]\npub struct Transform { pub translation: [f32; 3] }\n";

/// A script reads, index-assigns and measures a Rust `[f32; 3]` field, so a
/// host does not need a parallel script-side type just to reach it.
#[test]
fn array_fields_are_indexable_and_measurable() {
    let src = "use engine\n\
               read := (t: Transform) -> float { t.translation[1] }\n\
               bump := (t::Transform, d: float) -> _ { t.translation[1] := t.translation[1] + d }\n\
               axes := (t: Transform) -> int { len(t.translation) }\n";
    let (ok, rs, log) = compile_with(src, "arrayidx", &[], &[("engine.rs", ARRAY_SHIM)]);
    assert!(ok, "expected success, got:\n{log}");
    assert!(
        rs.contains("t.translation.homun_idx(1)"),
        "read, got:\n{rs}"
    );
    assert!(
        rs.contains("t.translation[1 as usize] ="),
        "index-assign, got:\n{rs}"
    );
    assert!(rs.contains("len!(t.translation)"), "len, got:\n{rs}");
}

/// The runtime must carry the impls that emission relies on.
#[test]
fn runtime_implements_index_and_len_for_arrays() {
    let rt = emit_runtime(&[]);
    assert!(
        rt.contains("HomunIndex<i32> for [T; N]"),
        "HomunIndex for arrays is missing"
    );
    assert!(
        rt.contains("HomunLen for [T; N]"),
        "HomunLen for arrays is missing"
    );
}
