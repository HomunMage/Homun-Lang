/// Regression tests for the `:=` immutability rule (v0.93).
///
/// A `:=` binding is immutable: reassigning it (via `:=`, `::=`, an `Assign`,
/// or in-place mutation) is a compile error. `::=` declares a mutable binding.
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Compile a Homun snippet; return (success, combined stdout+stderr).
fn compile(src: &str, name: &str) -> (bool, String) {
    let tmp = PathBuf::from(".tmp/homun_tests");
    fs::create_dir_all(&tmp).unwrap();
    let hom = tmp.join(format!("immut_{name}.hom"));
    let rs = tmp.join(format!("immut_{name}.rs"));
    fs::write(&hom, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_homunc"))
        .args([hom.to_str().unwrap(), "-o", rs.to_str().unwrap()])
        .output()
        .expect("failed to run homunc");
    let mut log = String::from_utf8_lossy(&out.stdout).into_owned();
    log.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), log)
}

#[test]
fn reassigning_immutable_with_colon_eq_is_error() {
    let (ok, log) = compile(
        "main := () -> _ {\n  x := 10\n  x := 20\n  print(x)\n}\n",
        "reassign",
    );
    assert!(!ok, "expected compile error, got success");
    assert!(
        log.contains("immutable"),
        "expected immutability error, got:\n{log}"
    );
}

#[test]
fn reassigning_immutable_with_colon_colon_eq_is_error() {
    let (ok, log) = compile(
        "main := () -> _ {\n  x := 10\n  x ::= 20\n  print(x)\n}\n",
        "mixmut",
    );
    assert!(!ok, "expected compile error, got success");
    assert!(
        log.contains("immutable"),
        "expected immutability error, got:\n{log}"
    );
}

#[test]
fn mutating_immutable_field_is_error() {
    let src = "P := struct { hp: int }\nmain := () -> _ {\n  p := P { hp: 1 }\n  p.hp := 2\n}\n";
    let (ok, log) = compile(src, "field");
    assert!(!ok, "expected compile error, got success");
    assert!(
        log.contains("immutable"),
        "expected immutability error, got:\n{log}"
    );
}

#[test]
fn mutable_binding_reassignment_compiles() {
    let (ok, log) = compile(
        "main := () -> _ {\n  c ::= 0\n  c ::= c + 1\n  print(c)\n}\n",
        "ok_mut",
    );
    assert!(ok, "expected success, got:\n{log}");
}

#[test]
fn loop_accumulator_with_colon_colon_eq_compiles() {
    let src = "main := () -> _ {\n  total ::= 0\n  i ::= 0\n  while (i < 5) { total ::= total + i  i ::= i + 1 }\n  print(total)\n}\n";
    let (ok, log) = compile(src, "ok_loop");
    assert!(ok, "expected success, got:\n{log}");
}

#[test]
fn fresh_immutable_bindings_compile() {
    let (ok, log) = compile(
        "main := () -> _ {\n  x := 1\n  y := x + 1\n  print(y)\n}\n",
        "ok_fresh",
    );
    assert!(ok, "expected success, got:\n{log}");
}
