# homunc — Homun → Rust Compiler

A text-to-text transpiler that compiles the **Homun** scripting language into **Rust** source code. The compiler is hemi-self-hosting — ~30% written in Homun (`.hom`) and ~70% in Rust (`_imp.rs` partners + `dep/`) — with a single external dependency (`regex`, used by the embedded `re` runtime library).

---

## Architecture

```
Source (.hom)
    │
    ▼
┌──────────┐
│ Resolver │  src/resolver.hom — multi-file dependency resolution (DFS)
└────┬─────┘    • 4-candidate search (mod.hom, .hom, mod.rs, .rs)
     │           • cycle detection (three-color algorithm)
     │           • embedded runtime fallback (no hom/ folder needed)
     ▼
┌─────────┐
│  Lexer  │  src/lexer.hom  — tokenises Homun source into Vec<Token>
└────┬────┘
     │ Vec<Token>
     ▼
┌─────────┐
│  Parser │  src/parser.hom — recursive-descent Pratt parser → AST
└────┬────┘
     │ Program (AST)
     ▼
┌──────────┐
│   Sema   │  src/sema.hom  — semantic analysis:
└────┬─────┘    • snake_case enforcement
     │           • undefined reference check
     │ Program   • (skips undef check when .rs deps present)
     ▼
┌──────────┐
│ Codegen  │  src/codegen.hom — walks AST, emits Rust text
└────┬─────┘
     │ Rust source (.rs)
     ▼
   rustc
```

---

## Build

```bash
# Requires Rust >= 1.85 (edition 2024)
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

# Emit embedded runtime (for multi-module Cargo projects)
homunc --emit-runtime > src/runtime.rs

# Compile a single module without prepending the runtime preamble
# (used by build.rs when compiling the compiler's own .hom sources)
homunc --module src/types.hom -o out/types.rs

# Pre-register an external .hom's enum variant types so cross-file Box
# auto-wrap/auto-deref work during compilation (compiler-bootstrap flag)
homunc --module --import src/ast.hom src/parser.hom -o out/parser.rs

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

## Runtime (embedded in `homunc`)

All runtime libraries are embedded in the `homunc` binary at compile time. No external files or submodules needed.

The compiler prepends `builtin.rs` to every output file. This provides:

| Helper | Description |
|---|---|
| `homun_slice(v, start, end, step)` | Python-style negative-index slicing |
| `homun_concat(a, b)` | List concatenation |
| `homun_in!(val, collection)` | Membership test |
| `homun_idx(idx)` | Indexing trait |
| `str_of(x)` | Convert anything Display to String |
| `dict![]`, `set![]`, `slice![]` | Collection construction macros |

The `std` library is also embedded and inlined when user code writes `use std`. It provides additional helpers (range, len, filter, map, reduce, string/math/collection utilities).

For multi-module Cargo projects, use `homunc --emit-runtime > src/runtime.rs` to extract the full runtime, then compile each `.hom` module with `homunc --module` (which strips runtime embedding).

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
| `@{"x","y"}` | `HashSet::from(["x","y"])` |
| `list \| filter(f) \| map(g)` | `map(filter(list, f), g)` |
| `if (c) { x } else { y }` | `if c { x } else { y }` |
| `match x { pat -> body }` | `match x { pat => body, }` |
| `for i in range(n) { }` | `for i in (0..n) { }` |
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
| `char` | `char` (single quotes: `'a'`, `'\n'`) |
| `none` | `Option<_>` |
| `@[T]` | `Vec<T>` |
| `@{K:V}` | `HashMap<K, V>` |
| `@{T}` | `HashSet<T>` |
| `_` (void) | `()` |
| unknown | `_` (let Rust infer) |

---

## File Structure

```
Homun-Lang/
├── Cargo.toml
├── Compiler-Design.md
├── Dockerfile           — cross-compilation (linux x86_64, aarch64, windows)
├── Dockerfile.test      — test image
├── Dockerfile.wasm      — WASM build (wasm32-wasip1)
├── hom-std/             — runtime library source (embedded in homunc at build time)
│   ├── builtin.rs       — macros (range!, len!, filter!, map!, dict!, set!, slice!)
│   ├── std/             — standard library (str, math, collection, dict, stack, deque, io)
│   ├── re.rs            — regex helpers
│   ├── heap.rs          — priority queue
│   ├── chars.rs         — character classification
│   ├── str_ext.rs       — string utilities
│   ├── dict.rs          — HashMap helpers
│   ├── set.rs           — set mutation helpers
│   ├── fs.rs            — filesystem helpers
│   └── path.rs          — path manipulation helpers
├── src/
│   ├── build.rs         — bootstraps .hom compilation, generates runtime.rs
│   ├── lib.rs           — wires compiled .hom modules + embedded runtime
│   ├── main_entry.rs    — committed bin shim → main_hom::main() (v0.88)
│   ├── ast.hom          — AST types (self-hosted, v0.84)
│   ├── main.hom         — CLI entry point (self-hosted)
│   ├── lexer.hom        — tokeniser (self-hosted)
│   ├── parser.hom       — recursive-descent parser (self-hosted)
│   ├── resolver.hom     — multi-file dependency resolution (self-hosted)
│   ├── sema.hom         — semantic analysis (self-hosted)
│   ├── codegen.hom      — Rust code emitter (self-hosted)
│   ├── scope.hom        — scope management (self-hosted)
│   ├── scope_imp.rs     — Rust helpers for scope.hom
│   ├── *_imp.rs         — Rust helpers for each .hom module
│   └── dep/             — shared Rust helpers (codegen_helpers, mod). Note: accessor layer removed v0.84; scope migrated to src/scope.hom v0.83.
├── tests/
│   ├── examples.rs      — compiles + runs _site/examples/*.hom
│   ├── hom_std.rs       — compiles + runs runtime test .hom files
│   └── std-tests/       — unit tests for hom-std runtime modules
└── _site/
    ├── index.html       — WASM playground
    ├── llm.txt          — language spec for LLMs
    └── examples/        — example .hom programs
```

## hom is learn from

| Language | What Homun Took |
|---|---|
| **Rust** | Compilation target, type system, ECS patterns, RON, error philosophy |
| **Svelte** | Compile-time transformation philosophy, "disappear at build time" |
| **Python** | `and/or/not/in`, slicing, `range`, `for x in y`, readable ergonomics |
| **Haskell/ML** | Lambdas-first, last-expression return, `match` exhaustiveness, map/filter/reduce |
| **Elixir/F#** | Pipe operator `\|`, function composition style |
| **Go** | No classes, errors as values, one obvious way, anti-ceremony |
| **C++** | `:=` binding operator — inspired by `auto` type deduction but without the keyword |
| **Lua** | Game scripting niche, thin layer over native engine, anonymous structs |
| **GDScript** | Designer-friendly motivation, hide systems complexity |
| **OCaml** | Auto-detected recursion (hidden `rec`) |


## hemi-self-host

homunc source cod is 30% .hom + 70% .rs . hom will not become fully self-hosting.
