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
use homunc::{ast, codegen_hom, embedded_rs, lexer, parser, resolver, sema_hom};

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    // Check for --raw flag (skip preamble, for module compilation)
    let raw = args.iter().any(|a| a == "--raw");
    // Check for --module flag (skip preamble + don't embed runtime libs)
    let module = args.iter().any(|a| a == "--module");
    let raw = raw || module; // --module implies --raw
    let args: Vec<&str> = args
        .iter()
        .filter(|a| *a != "--raw" && *a != "--module")
        .map(|s| s.as_str())
        .collect();
    match args.as_slice() {
        [flag] if *flag == "--help" || *flag == "-h" => {
            print_help();
        }
        [flag] if *flag == "--version" || *flag == "-v" => {
            println!("homunc {}", env!("HOMUN_VERSION"));
        }
        [] => {
            compile_from_stdin_with(raw);
        }
        [src] => {
            compile_to_stdout_with(src, raw, module);
        }
        [src, flag, out] if *flag == "-o" => {
            compile_to_file_with(src, out, raw, module);
        }
        _ => {
            eprintln!("Usage: homunc [--raw|--module] [input.hom] [-o output.rs]");
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
fn compile_source(source: &str, raw: bool) -> Result<String, String> {
    use std::collections::HashMap;
    let tokens = lexer::lex(source).map_err(|e| format!("Lex error: {}", e))?;
    let ast = parser::parse(tokens).map_err(|e| format!("Parse error: {}", e))?;
    let sema_errs = sema_hom::sema_analyze_skip_undef(ast.clone(), Vec::new());
    if !sema_errs.is_empty() {
        return Err(format!("Semantic errors:\n{}", sema_errs.join("\n")));
    }
    // Resolve embedded libraries (std) for use statements.
    let mut rs_content: HashMap<String, String> = HashMap::new();
    for stmt in &ast {
        if let ast::Stmt::Use(path) = stmt
            && path.len() == 1
            && let Some(content) = embedded_rs(&path[0])
        {
            rs_content.insert(path[0].clone(), content);
        }
    }
    let code = codegen_hom::codegen_program_with_resolved(ast, Default::default(), rs_content);
    let prefix = if raw { String::new() } else { preamble() };
    Ok(format!("{}{}", prefix, code))
}

/// Compile a .hom file, resolving multi-file `use` imports recursively.
fn compile_file(path: &Path, raw: bool, module: bool) -> Result<String, String> {
    let resolved = if module {
        resolver::resolve_module(path)?
    } else {
        resolver::resolve(path)?
    };
    let mut output = if raw { String::new() } else { preamble() };
    for (i, file) in resolved.files.iter().enumerate() {
        output.push_str(&file.rust_code);
        if i + 1 < resolved.files.len() {
            output.push('\n');
        }
    }
    Ok(output)
}

fn compile_from_stdin_with(raw: bool) {
    let mut src = String::new();
    io::stdin()
        .read_to_string(&mut src)
        .expect("Failed to read stdin");
    match compile_source(&src, raw) {
        Ok(out) => print!("{}", out),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn compile_to_stdout_with(path: &str, raw: bool, module: bool) {
    match compile_file(Path::new(path), raw, module) {
        Ok(out) => print!("{}", out),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn compile_to_file_with(src: &str, out: &str, raw: bool, module: bool) {
    match compile_file(Path::new(src), raw, module) {
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
        include_str!("hom/builtin.rs")
    )
}
