# homunc — Homun → Rust Compiler

A text-to-text transpiler that compiles the **Homun** scripting language into **Rust** source code. Written entirely in Rust with zero external dependencies.

---

## Architecture

```
Source (.hom)
    │
    ▼
┌──────────┐
│ Resolver │  src/resolver.rs — multi-file dependency resolution (DFS)
└────┬─────┘    • 4-candidate search (mod.hom, .hom, mod.rs, .rs)
     │           • cycle detection (three-color algorithm)
     │           • .rs include!() expansion
     ▼
┌─────────┐
│  Lexer  │  src/lexer.rs   — tokenises Homun source into Vec<Token>
└────┬────┘
     │ Vec<Token>
     ▼
┌─────────┐
│  Parser │  src/parser.rs  — recursive-descent Pratt parser → AST
└────┬────┘
     │ Program (AST)
     ▼
┌──────────┐
│   Sema   │  src/sema.rs   — semantic analysis:
└────┬─────┘    • snake_case enforcement
     │           • undefined reference check
     │ Program   • (skips undef check when .rs deps present)
     ▼
┌──────────┐
│ Codegen  │  src/codegen.rs — walks AST, emits Rust text
└────┬─────┘
     │ Rust source (.rs)
     ▼
   rustc
```

---

## Build

```bash
# Requires Rust >= 1.70
cargo build --release
```

Version is set via `HOMUN_VERSION` environment variable (defaults to `git describe --tags`). In CI, the release workflow passes `--build-arg VERSION=<tag>` to Docker.

---

## Usage

```bash
# Compile to stdout
homunc examples/quicksort.hom

# Compile to file
homunc examples/fizzbuzz.hom -o output.rs

# Version / help
homunc -v
homunc --help

# Stdin (WASM playground uses this)
echo 'print("hello")' | homunc

# Then compile the Rust
rustc output.rs -o program
```

---

## Imports (`use`)

`use foo` resolves against **exactly one** of four candidates in the same directory as the source file:

| Priority | Path | Type | Action |
|---|---|---|---|
| 1 | `foo/mod.hom` | Homun folder | Compile recursively, inline Rust output |
| 2 | `foo.hom` | Homun file | Compile and inline |
| 3 | `foo/mod.rs` | Rust folder | Read, expand `include!()`, inline content |
| 4 | `foo.rs` | Rust file | Read and inline content |

- **0 matches** → pass through as Rust `use foo;`
- **1 match** → resolve and inline
- **2+ matches** → compile error (`Ambiguous import`)

### Uniqueness Rule

Only one form of `foo` may exist. If both `dog.rs` and `dog.hom` exist, or `dog/mod.hom` and `dog.rs`, the compiler emits an ambiguity error.

### Folder Namespaces

`foo/mod.hom` (or `foo/mod.rs`) resolves `use` statements relative to `foo/`:

```
opencv/
  mod.hom        // "use img" → resolves opencv/img.hom
  img.hom
  filter.hom
```

### Self-Contained Output

The compiled Rust output contains all dependencies inline — no `include!()` statements remain. For `.rs` dependencies, the resolver recursively expands any `include!("...")` lines before inlining.

### Dependency Resolution

- **Cycle detection** — three-color DFS algorithm; circular `use` chains → compile error
- **Deduplication** — each file compiled at most once (tracked by canonical path)
- **Topological order** — leaves compiled first, then files that depend on them

### WASM / Stdin Mode

When compiling from stdin (no filesystem), `use` statements pass through as `use foo;`. The WASM playground handles library inlining in JavaScript by loading `.rs` files from `examples/std/` and `examples/ext/` and replacing `use std;` / `use ext;` in the output.

---

## Runtime (`runtime/builtin.rs`)

The compiler always prepends `runtime/builtin.rs` to every output file via `include_str!()` at compiler build time. This provides:

| Helper | Description |
|---|---|
| `homun_slice(v, start, end, step)` | Python-style negative-index slicing |
| `homun_concat(a, b)` | List concatenation |
| `homun_in!(val, collection)` | Membership test |
| `homun_idx(idx)` | Indexing trait |
| `str_of(x)` | Convert anything Display to String |
| `dict![]`, `set![]`, `slice![]` | Collection construction macros |

The `std` library (`runtime/std/`) is also embedded in the compiler binary via `include_str!()` and inlined when user code writes `use std`. It provides additional helpers (range, len, filter, map, reduce, string/math/collection utilities). Users do not need a `std/` folder on disk — the resolver checks embedded libraries automatically.

---

## Language Coverage

| Homun Feature | Rust Output |
|---|---|
| `x := 10` | `let mut x = 10;` |
| `fn := (a, b) -> { a + b }` | `pub fn fn<T,U>(a: T, b: U) -> _ { a + b }` |
| `fn := (a: int) -> int { ... }` | `pub fn fn(a: i32) -> i32 { ... }` |
| `fib := (n) -> { fib(n-1) }` | recursive fn via `fn fib<T>(...) -> _` |
| `@[1,2,3]` | `vec![1, 2, 3]` |
| `@{"a": 1}` | `HashMap::from([("a", 1)])` |
| `@("x","y")` | `HashSet::from(["x","y"])` |
| `list \| filter(f) \| map(g)` | `map(filter(list, f), g)` |
| `if (c) do { x } else { y }` | `if c { x } else { y }` |
| `match x { pat => body }` | `match x { pat => body, }` |
| `for i in range(n) do { }` | `for i in (0..n) { }` |
| `break => value` | `break value` |
| `"Hello ${name}"` | `format!("Hello {}", name)` |
| `Player := struct { hp: int }` | `pub struct Player { pub hp: i32, }` |
| `Direction := enum { North }` | `pub enum Direction { North, }` |
| `load_ron("f.ron") as Map` | `ron::from_str::<Map>(...)` |
| `save_ron(data, "f.ron")` | `std::fs::write(...)` |

---

## Semantic Checks

The `Sema` pass enforces Homun's rules **before** codegen:

1. **snake_case** — all variable and lambda names must be `snake_case`
2. **Undefined references** — references to names not yet defined → compile error (skipped when `.rs` deps are present since sema can't introspect Rust files)

---

## Homun Type → Rust Type Mapping

| Homun | Rust |
|---|---|
| `int` | `i32` |
| `float` | `f32` |
| `bool` | `bool` |
| `str` | `String` |
| `none` | `Option<_>` |
| `@[T]` | `Vec<T>` |
| `@{K:V}` | `HashMap<K, V>` |
| `@(T)` | `HashSet<T>` |
| `_` (void) | `()` |
| unknown | `_` (let Rust infer) |

---

## File Structure

```
Homun-Lang/
├── Cargo.toml
├── build.rs             — sets HOMUN_VERSION from git tag or env var
├── Compiler.md
├── Dockerfile           — cross-compilation (linux x86_64, aarch64, windows)
├── Dockerfile.wasm      — WASM build (wasm32-wasi)
├── runtime/
│   ├── builtin.rs       — runtime helpers (embedded in compiler output)
│   └── std/             — standard library (embedded in compiler binary)
│       ├── mod.rs
│       ├── str.rs
│       ├── math.rs
│       └── collection.rs
├── src/
│   ├── main.rs          — CLI entry point, preamble, stdin/file compilation
│   ├── lexer.rs         — tokeniser
│   ├── ast.rs           — abstract syntax tree types
│   ├── parser.rs        — recursive-descent parser
│   ├── resolver.rs      — multi-file dependency resolution
│   ├── sema.rs          — semantic analysis
│   └── codegen.rs       — Rust code emitter
└── _site/
    ├── index.html       — WASM playground
    └── examples/
        ├── *.hom        — example Homun programs
        └── ext/         — extended library (.rs)
            ├── mod.rs
            ├── str.rs
            ├── math.rs
            ├── collection.rs
            ├── dict.rs
            ├── stack.rs
            ├── deque.rs
            └── io.rs
```
