/// Integration tests — compile and run every _site/examples/*.hom file.
/// Each .hom is compiled via homunc (resolver + codegen), then rustc, then executed.
use std::fs;
use std::path::Path;
use std::process::Command;

fn compile_and_run(hom_path: &Path) -> String {
    let tmp = std::env::temp_dir().join("homun_tests");
    fs::create_dir_all(&tmp).unwrap();

    let stem = hom_path.file_stem().unwrap().to_string_lossy();

    // Compile .hom → .rs via cargo run
    let rs_path = tmp.join(format!("{}.rs", stem));
    let compile_hom = Command::new(env!("CARGO_BIN_EXE_homunc"))
        .args([hom_path.to_str().unwrap(), "-o", rs_path.to_str().unwrap()])
        .output()
        .expect("failed to run homunc");

    assert!(
        compile_hom.status.success(),
        "{}: homunc failed:\n{}",
        stem,
        String::from_utf8_lossy(&compile_hom.stderr)
    );

    // Compile .rs → binary via rustc
    let bin_path = tmp.join(format!("{}_bin", stem));
    let compile_rs = Command::new("rustc")
        .args([rs_path.to_str().unwrap(), "-o", bin_path.to_str().unwrap()])
        .output()
        .expect("rustc not found");

    if !compile_rs.status.success() {
        let src = fs::read_to_string(&rs_path).unwrap_or_default();
        panic!(
            "{}: rustc failed:\n{}\n\nGenerated .rs:\n{}",
            stem,
            String::from_utf8_lossy(&compile_rs.stderr),
            src
        );
    }

    // Run binary
    let run = Command::new(&bin_path)
        .output()
        .expect("failed to run binary");

    assert!(
        run.status.success(),
        "{}: binary exited with error:\n{}",
        stem,
        String::from_utf8_lossy(&run.stderr)
    );

    String::from_utf8(run.stdout).unwrap()
}

#[test]
fn test_example_fib() {
    let out = compile_and_run(Path::new("_site/examples/fib.hom"));
    assert!(out.contains("fib(0) = 0"));
    assert!(out.contains("fib(9) = 34"));
}

#[test]
fn test_example_fizzbuzz() {
    let out = compile_and_run(Path::new("_site/examples/fizzbuzz.hom"));
    assert!(out.contains("FizzBuzz"));
    assert!(out.contains("Hello, Aria!"));
}

#[test]
fn test_example_binary_search() {
    let out = compile_and_run(Path::new("_site/examples/binary_search.hom"));
    assert!(out.contains("index 3"));
    assert!(out.contains("index -1"));
}

#[test]
fn test_example_dfs() {
    let out = compile_and_run(Path::new("_site/examples/dfs.hom"));
    assert!(out.contains("DFS traversal:"));
}

#[test]
fn test_example_pipeline() {
    let out = compile_and_run(Path::new("_site/examples/pipeline.hom"));
    assert!(out.contains("220"));
}

#[test]
fn test_example_quicksort() {
    let out = compile_and_run(Path::new("_site/examples/quicksort.hom"));
    assert!(out.contains("[1, 2, 3, 4, 5, 6, 7, 8, 9]"));
}

#[test]
fn test_example_two_sum() {
    let out = compile_and_run(Path::new("_site/examples/two_sum.hom"));
    assert!(out.contains("Two Sum:"));
}
