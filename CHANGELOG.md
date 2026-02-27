# Changelog

All notable changes to Homun-Lang are documented in this file.

Branches: `history` (spec drafts), `haskell` (Haskell compiler), `rust` (Rust rewrite, merged to `main`).

---

## homunc + rustc Era (v0.40+) 

### v0.40 — Starting Self-Host

- Added `--raw` flag to skip preamble (for compiling modules, not standalone programs)
- First `.hom` source file: `runtime/helpers.hom` with `use helpers_imp` pattern (mixed Homun + Rust)
- `build.rs` auto-compiles `.hom` → `.rs` if `homunc` is in PATH; falls back to checked-in `.rs`
- Dockerfiles bootstrap by downloading released `homunc` to compile `.hom` files before `cargo build`
- Merged `ext` library into `runtime/std/` (dict, stack, deque, io, char_at)
- Added practical std functions: `push`/`pop`/`remove` (vec), `insert`/`remove_key` (dict), `parse_int`/`parse_float` (str)
- Release asset filenames no longer include version (e.g., `homunc-linux-x86_64`)
- `cd.yml` downloads released WASM tarball instead of rebuilding
- Playground displays WASM compiler version via `version.txt`
- Playground removed all JS-side library loading — everything embedded in WASM compiler

## Rust Compiler Era (v0.30+)

### v0.34 — Embedded Standard Library

- Moved `std` library from `_site/examples/std/` to `runtime/std/` — now fully embedded in compiler binary via `include_str!()`, no `std/` folder needed on disk
- Resolver checks embedded runtime libraries when no filesystem match found — `use std` works without any files on disk
- WASM playground: removed std from JS lib loading (handled natively by embedded runtime), keeps ext only
- Added `.hooks/pre-commit` with `cargo fmt --check`

### v0.33 — Unified Namespace Resolution

- Unified `use` resolution: 4-candidate search (`foo/mod.hom`, `foo.hom`, `foo/mod.rs`, `foo.rs`) with strict uniqueness rule (ambiguity = compile error)
- Folder-as-namespace: `foo/mod.hom` or `foo/mod.rs` as entry points, sub-files resolve relative to folder
- Self-contained output: `.rs` dependencies recursively expanded (all `include!()` resolved and inlined)
- Removed special-casing of `use std` / `use ext` from codegen and sema — all imports follow the same resolution rule
- Embedded `runtime/builtin.rs` into compiler binary via `include_str!()` (always prepended to output)
- Added `build.rs` for git tag versioning (`homunc -v` shows version)
- Added `VERSION` build arg to Dockerfiles and release workflow for CI builds
- Reorganized `_site/examples/` into folder namespaces: `std/mod.rs` + sub-files, `ext/mod.rs` + sub-files
- Removed `_site/builtin.rs` and `_site/std.rs` (no longer needed)
- Updated `Compiler.md` and `README.md` with new import system docs

### v0.32 — Extended Library & WASM

- Added `ext` (extended library): `ext.rs` with `ext_str.rs`, `ext_math.rs`, `ext_collection.rs`, `ext_dict.rs`, `ext_stack.rs`, `ext_deque.rs`, `ext_io.rs`
- Added `_site/builtin.rs` and `_site/std.rs` for WASM playground runtime
- Added `Dockerfile.wasm` for WASM builds
- Simplified `sema.rs`: removed recursion/mutual-recursion detection, kept snake_case and undefined reference checks
- Codegen: added `use ext` → `include!("ext.rs")` support

### v0.31 — Multi-File Import

- Added `resolver.rs`: DFS dependency resolver with three-color cycle detection
- `use foo` resolves `foo.hom` in the same directory, compiles recursively, inlines in topo order
- Diamond deduplication: shared dependencies emitted exactly once
- Preamble emitted once regardless of file count
- Backward compatible: stdin/WASM unchanged; `use foo::bar` remains Rust pass-through; `use std` still emits `include!("std.rs")`

### v0.30 — Haskell to Rust Rewrite

- Rewrote entire compiler from Haskell to Rust
- Replaced `AST.hs`, `Lexer.hs`, `Parser.hs`, `Sema.hs`, `Codegen.hs`, `Main.hs` with `ast.rs`, `lexer.rs`, `parser.rs`, `sema.rs`, `codegen.rs`, `main.rs`
- Added `Cargo.toml`, `Dockerfile`, CI/CD workflows (`ci.yml`, `release.yml`)
- Added `Compiler.md` spec document
- Moved `builtin.rs` to `runtime/`
- Moved example `.hom` files and `std*.rs` to `_site/examples/`

---

## Haskell Compiler Era (v0.23–v0.29, `haskell` branch)

### v0.29 — File Reorganization

- Moved examples to `_site/examples/`
- Prepared directory layout for Rust rewrite

### v0.28 — Standard Library Extensions

- Added `std_collection.rs`, `std_math.rs`, `std_str.rs`
- Expanded `Compiler.md` with standard library docs

### v0.27 — Playground Refinements

- Refined WASM playground UI and `Main.hs`

### v0.26 — WASM Playground

- Added web-based playground (`_site/index.html`)
- Added CD workflow and `Dockerfile.wasm`

### v0.25 — Runtime Split

- Split runtime into `builtin.rs` + `std.rs`
- Added `use std` support in AST/Parser/Codegen
- Added `.gitignore`
- Added `binary_search.hom`, `fib.hom`, `pipeline.hom`, `two_sum.hom` examples

### v0.24 — Runtime Extraction

- Extracted runtime into `std.rs` (110 lines)
- Added `binary_search.hom`, `pipeline.hom`, `two_sum.hom` examples

### v0.23 — Initial Haskell Compiler

- Full pipeline: `Lexer.hs` -> `Parser.hs` -> `Sema.hs` -> `Codegen.hs` targeting Rust output
- `AST.hs` with structs, enums, lambdas, pattern matching, collections
- `Main.hs` with file/stdin compilation
- Major parser rewrite and improved codegen
- Added `dfs.hom`, `fib.hom` examples; renamed `src/README.md` to `Compiler.md`
- Example programs: `fizzbuzz.hom`, `quicksort.hom`

---

## Spec Design Era (v0.1–v0.22, `main` README + `history` branch)

### v0.22 — Text-to-Text Clarification

- Clarified that Homun compiles text-to-text (not to Rust AST or binary)
- README wording updates

### v0.21 — Examples-First Rewrite

- Major README rewrite: examples-first documentation
- Replaced `steps()` with `range()` as standard range function
- Simplified type handling — Homun delegates all generics/monomorphization to Rust
- Code examples: Valid Parentheses, Quicksort, DFS, FizzBuzz, Binary Search, Two Sum

### v0.20 — Compact Spec

- Examples-first format with `range()` function
- Dropped verbose prose sections

### v0.19 — Use & Slicing

- Added `use` for Rust library imports
- Python-style `[start:end)` slicing

### v0.18 — 0-Based Refinements

- Further refinements to 0-based indexing spec

### v0.17 — 0-Based Refinements

- Refinements to 0-based indexing spec

### v0.16 — 0-Based Indexing

- Switched from 1-based to 0-based indexing

### v0.15 — Reference Doc Rewrite

- Major rewrite to polished reference-doc style ("Homun Language Reference")

### v0.14 — Range Function

- Renamed `upto()` / `from()` to `steps(start, end)` with optional step argument
- `steps` supports negative step for countdown ranges

### v0.13 — Pipe `|` & Arrow Lambda

- Changed pipe operator from `.` (newline-sensitive) to `|` (explicit, no whitespace rules)
- Lambda syntax changed from `|params| { body }` to `(params) -> { body }`
- Introduced `-> _` as void return marker (replacing `-> ()`)
- Braces always required around lambda bodies

### v0.12 — Pipe Disambiguation

- Same-line `.` is field access, new-line `.` is pipe call
- Clarified `none` is only a value, not a type annotation — `-> ()` is the sole void return form

### v0.11.2 — `@` Collection Prefix

- `@[]` list, `@{}` dict, `@()` set
- `[` only for indexing, `{` only for blocks

### v0.11 — Named "Homun"

- Language named "Homun"
- Documented intentional design decisions (`:=` only, `.` pipe, 1-based indexing)
- Added RON load/save support, recursion detection, comments (`//`, `/* */`)

### v0.10 — Initial Spec

- First complete language reference published as `README.md`
- Core: `:=` binding, snake_case enforcement, `@` collections, 1-based indexing, pattern matching, for/while, string interpolation
- Philosophy: no classes, no functions (only lambdas), no semicolons, no bare `=`

### v0.9 — `${}` Interpolation

- String interpolation changed from `{}` to `${}`

### v0.8 — `|params|` Syntax & `@` Comprehensions

- Lambda syntax changed to `|params| -> Type {}`
- `@` for comprehensions
- `{}` string interpolation

### v0.7 — `||` Lambda & `.` Pipe

- Lambda syntax changed to `|| -> Type {}`
- `.` serves as pipe operator

### v0.6 — `\()` Lambda

- Lambda syntax changed to `\() -> Type {}`

### v0.5.5 — UFCS `.`

- Replaced `|>` with UFCS `.` for composition

### v0.5 — Reorganization

- Reorganized spec sections (A/B/C structure)

### v0.4 — `|>` Pipe Operator

- Added `|>` pipe operator
- `.` restricted to field access only

### v0.3 — Arrow Lambda & `==`

- Lambda syntax changed to `() -> Type {}`
- Equality operator fixed to `==` only (removed bare `=` for equality)

### v0.2 — Spec Draft

- Same core as v0.1

### v0.1 — Initial Draft

- Unnamed spec: `lambda()` syntax, `=` for equality, `mut` keyword, 1-based indexing
