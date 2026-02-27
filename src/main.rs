/// homunc — Homun to Rust compiler
///
/// Usage:
///   homunc <input.hom>           # prints Rust to stdout
///   homunc <input.hom> -o <out>  # writes to file
///   homunc --help
///
/// Pipeline:
///   Source text
///     -> Lexer   (lexer.rs)   -> Vec<Token>
///     -> Parser  (parser.rs)  -> Program (AST)
///     -> Sema    (sema.rs)    -> Checked Program
///     -> Codegen (codegen.rs) -> Rust source text
#[allow(dead_code)]
mod ast;
mod codegen;
#[allow(dead_code)]
mod lexer;
mod parser;
mod resolver;
mod sema;

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [flag] if flag == "--help" || flag == "-h" => {
            print_help();
        }
        [flag] if flag == "--version" || flag == "-v" => {
            println!("homunc {}", env!("HOMUN_VERSION"));
        }
        [] => {
            compile_from_stdin();
        }
        [src] => {
            compile_to_stdout(src);
        }
        [src, flag, out] if flag == "-o" => {
            compile_to_file(src, out);
        }
        _ => {
            eprintln!("Usage: homunc [input.hom] [-o output.rs]");
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("homunc {} — Homun to Rust compiler", env!("HOMUN_VERSION"));
    println!();
    println!("USAGE:");
    println!("  homunc <input.hom>            Compile and print Rust to stdout");
    println!("  homunc <input.hom> -o out.rs  Compile and write to file");
    println!("  homunc -v, --version          Show version");
    println!("  homunc -h, --help             Show this message");
    println!();
    println!("PIPELINE:");
    println!("  .hom source  ->  Lexer  ->  Parser  ->  Sema  ->  Codegen  ->  .rs");
    println!();
    println!("LANGUAGE FEATURES SUPPORTED:");
    println!("  * Variable bindings      x := 10");
    println!("  * Lambdas                fn := (a, b) -> {{ a + b }}");
    println!("  * Typed params           fn := (a: int, b: int) -> int {{ a + b }}");
    println!("  * Recursion              fib := (n) -> {{ if ... {{ fib(n-1) + fib(n-2) }} }}");
    println!("  * Pipe operator          list | filter(f) | map(g)");
    println!("  * Collections            @[], @{{}}, @()");
    println!("  * Pattern match          match x {{ ... }}");
    println!("  * if/else, for, while");
    println!("  * break => value         for ... do {{ break => val }}");
    println!("  * Structs & Enums");
    println!("  * String interpolation   \"Hello ${{name}}\"");
    println!("  * RON load/save");
}

/// Compile source text directly (used for stdin / WASM — no file resolution).
/// `use std` is handled via embedded runtime; other `use` statements pass through.
fn compile_source(source: &str) -> Result<String, String> {
    use std::collections::HashMap;
    let tokens = lexer::lex(source).map_err(|e| format!("Lex error: {}", e))?;
    let ast = parser::parse(tokens).map_err(|e| format!("Parse error: {}", e))?;
    sema::analyze_program_with_imports_skip_undef(&ast, &Default::default()).map_err(|errs| {
        let msgs: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        format!("Semantic errors:\n{}", msgs.join("\n"))
    })?;
    // Resolve embedded libraries (std) for use statements.
    let mut rs_content: HashMap<String, String> = HashMap::new();
    for stmt in &ast {
        if let ast::Stmt::Use(path) = stmt {
            if path.len() == 1 {
                if let Some(content) = embedded_rs(&path[0]) {
                    rs_content.insert(path[0].clone(), content);
                }
            }
        }
    }
    let code = codegen::codegen_program_with_resolved(&ast, &Default::default(), &rs_content);
    let rust_src = format!("{}{}", preamble(), code);
    Ok(rust_src)
}

/// Return embedded .rs content for official runtime libraries.
/// Files are embedded from `runtime/` at compiler build time.
pub fn embedded_rs(name: &str) -> Option<String> {
    match name {
        "std" => {
            let mod_rs: String = include_str!("../runtime/std/mod.rs")
                .lines()
                .filter(|l| !l.trim().starts_with("include!("))
                .collect::<Vec<_>>()
                .join("\n");
            Some(format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                mod_rs,
                include_str!("../runtime/std/str.rs"),
                include_str!("../runtime/std/math.rs"),
                include_str!("../runtime/std/collection.rs"),
                include_str!("../runtime/std/dict.rs"),
                include_str!("../runtime/std/stack.rs"),
                include_str!("../runtime/std/deque.rs"),
                include_str!("../runtime/std/io.rs"),
            ))
        }
        _ => None,
    }
}

/// Compile a .hom file, resolving multi-file `use` imports recursively.
fn compile_file(path: &Path) -> Result<String, String> {
    let resolved = resolver::resolve(path)?;
    let mut output = preamble();
    for (i, file) in resolved.files.iter().enumerate() {
        output.push_str(&file.rust_code);
        if i + 1 < resolved.files.len() {
            output.push('\n');
        }
    }
    Ok(output)
}

fn compile_from_stdin() {
    let mut src = String::new();
    io::stdin()
        .read_to_string(&mut src)
        .expect("Failed to read stdin");
    match compile_source(&src) {
        Ok(out) => print!("{}", out),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn compile_to_stdout(path: &str) {
    match compile_file(Path::new(path)) {
        Ok(out) => print!("{}", out),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn compile_to_file(src: &str, out: &str) {
    match compile_file(Path::new(src)) {
        Ok(code) => {
            fs::write(out, &code).unwrap_or_else(|e| {
                eprintln!("Cannot write {}: {}", out, e);
                process::exit(1);
            });
            println!("Compiled {} -> {}", src, out);
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn preamble() -> String {
    format!(
        "// Generated by homunc — Homun to Rust compiler\n\
         \n\
         #![allow(unused_variables, unused_mut, dead_code, unused_imports, unused_macros)]\n\
         #![allow(non_snake_case)]\n\
         \n\
         // ── builtin ────────────────────────────────────────────────\n\
         {}\n",
        include_str!("../runtime/builtin.rs")
    )
}
