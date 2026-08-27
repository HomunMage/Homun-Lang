# Changelog

All notable changes to Homun-Lang are documented in this file.

Branches: `history` (spec drafts), `haskell` (Haskell compiler), `rust` (Rust rewrite, merged to `main`).

---

### v0.94 — 2026-08-27 — Rust-interop fixes and host-driven compilation

Every item was found by using Homun for real: a game engine's components and
systems moved into `.hom`, compiled to cdylibs the engine loads. `report.md`
records each with a repro. Four new flags let a host drive the compiler instead
of staging files next to every input.

- `--extern NAME`: `use NAME` references a module the consumer provides instead of inlining it, and the pass-through `use NAME;` is suppressed. Inlining copies a sibling `.rs` into every module that mentions it, so its types become a distinct type per module — the reason a host could not share one `Transform` across a game's systems. `skip_embed` generalised from hom-std to a named dependency
- `--include DIR` (repeatable): search these after the importing file's own directory, so a host compiles sources without copying their dependencies alongside each input. The file's own directory still wins
- `--runtime-path PATH`: emit `use PATH::*;` ahead of `--module` output, which called `len!` and `.homun_len()` but imported neither and failed with `no method named homun_len` even though the impl existed
- `--emit-runtime` now emits an includable fragment: `builtin` + `std` only, so no duplicate `is_alpha`/`is_digit`/`is_alnum`/`is_upper`, no undeclared `regex` dependency, no inner attributes. `re`/`heap`/`chars`/`str_ext`/`dict`/`path`/`fs` are opt-in with `--with NAME`
- `builtin.rs` / `std/mod.rs`: `HomunIndex` + `HomunLen` for `[T; N]`, so a script reads and measures a Rust `[f32; 3]` field directly. Index assignment was already correct, so only the read path failed — and a host had to define a parallel script-side type and convert per entity per frame
- `parser.hom`: braced and glob `use` paths pass through (`use engine::{A, B}` failed with `Expected identifier, got LBrace` despite being documented); `token_to_body_str` gained `DoubleColon`, so `@derive(Clone, serde::Deserialize)` no longer emits `serdeDeserialize`
- `lexer.hom`: `_` starts an identifier when the next character continues one, so `for _p in xs` parses; a bare `_` is still the wildcard
- `codegen.hom`: `=> _` emits `return;` rather than `return _;`, which rustc rejects
- new `tests/v94_fixes.rs`; 181 tests pass

Left open in `report.md`: string literals take `.to_string()` in argument
position, a list literal cannot target an array field, and a script function
whose name matches a runtime function collides silently.

---

### v0.93 — 2026-06-13 — `:=` is truly immutable (no reassignment)

`:=` now means an **immutable** binding, enforced end-to-end. Previously `:=` and `::=` were codegen-identical (both emitted `let mut` and silently allowed reassignment), so the immutability `:=` claimed was never enforced. Net diff: **+846 / −501** across 17 files.

- `codegen.hom`: `:=` → `let` (immutable), `::=` → `let mut` / reassignment; tuple binds `a, b :=` → `let (a, b)`, `a, b ::=` → mutable
- `sema.hom`: new immutability pass — rebinding or reassigning a `:=` name (via `:=`, `::=`, or `Assign`) is a compile error; `::=` reassigns a mutable. Runs always (independent of `.rs` deps); params are exempt (they compile to `mut`). No shadowing — a name visible from any enclosing scope cannot be re-`:=`-bound
- migrated every reassigned / in-place-mutated local in the compiler's own `.hom` sources + all `_site/examples` and `tests` from `:=` → `::=` (loop counters, accumulators, `push`/`heap_push` targets); self-host fixpoint holds (stage-2 ≡ stage-3 byte-identical)
- new `tests/immutability.rs` regression suite
- 153 tests pass, `cargo fmt` + `cargo clippy --release -- -D warnings` clean

---

### v0.92 — 2026-06-01 — AST robustness + kill `dep/` + indirect-recursion auto-Box

Made the AST fully positional, deleted the `dep/` directory by folding its helpers into `codegen_imp.rs`, and taught auto-Box to handle indirect recursion. Net diff: **+397 / −722** across 17 files.

- AST 100% positional: `Lambda` named-field → 6-field positional; delete 10 Rust accessors/constructors (`src/ast.hom`)
- New `LValue` enum: `Stmt.Assign(LValue, Expr)` restricts lhs to valid targets
- `BindMut` now carries attrs `@[str]`, aligned with `Bind`
- Removed dead `LoadRon`/`SaveRon` variants (never constructed by parser)
- Killed `dep/` directory: `dep/codegen_helpers.rs` (−262) + `dep/mod.rs` (−16) deleted; content → new `src/codegen_imp.rs` (+122); `lib.rs` rewired to `use crate::ast::*`
- Auto-Box supports indirect recursion (all enum names registered upfront in pre-pass)
- Type-specific helpers → generics: `vec_extend<T>`, `vec_push<T>`, `vec_pop<T>`
- `.gitignore` cleanup (−124): collapse stale per-artifact entries
- Reverted the "register all enum names" pre-pass — it over-Boxed `Expr` fields in `Stmt` variants, breaking user code. Restored per-enum register/clear (v0.91 behavior)

---

### v0.91 — 2026-05-29 — `--import` stage 3: migrate ~20 lexer/parser/sema dispatch helpers from `_imp.rs` to `.hom` (Hom:Rs 1.92 → 2.65)

Stage 3 of the `--import` rollout. With `parser.hom` now consuming `--import src/lexer.hom`, lexer's `TokenKind` variants are visible cross-file — so token inspection, matching, and keyword/single-op dispatch can finally live in `.hom`. Net diff: **+551 / −613** across 9 files; Hom:Rs source ratio jumps from 1.92 to 2.65.

- `build.rs`: when compiling `parser.hom`, also pass `--import src/lexer.hom` so F9 match auto-deref + F12 construct auto-wrap work against `TokenKind` cross-file
- `lexer.hom`: new `ls_keyword_token` (20-arm `match` on keyword strings → `TokenKind.*` variants) and `ls_try_single_op` (22-arm operator/delimiter dispatch)
- `lexer_imp.rs`: deleted `ls_keyword`/`ls_keyword_token`/`ls_try_single_op` (−67 lines)
- `parser.hom`: gained 19 helpers — `ps_peek_kind`/`ps_peek_ident`/`ps_peek_int`/`ps_peek_float`/`ps_peek_bool`/`ps_peek_str`/`ps_peek_char`, `ps_check`/`ps_consume`/`ps_expect`/`ps_advance_ident`, `ps_collect_attr_body`/`ps_collect_outer_attr_body`/`token_to_body_str`, `split_block`, `push_name_expr_pair`/`push_expr_pair`/`new_name_expr_pairs`/`new_expr_pairs`
- `parser_imp.rs`: shrank from ~298 lines of token plumbing to a thin `ps_peek_token` + numeric-cast shims (`to_i64`/`to_f64`); kept `ps_same_line` (needs `parse_pos` thread-local)
- `sema_imp.rs`: collapsed to a 2-line `has_rs_dep` stub; `errs_empty`/`errs_one`/`errs_join` + `expr_is_lambda` migrated into `sema.hom` (or already live in `codegen_helpers`)
- `dep/codegen_helpers.rs`: deleted `is_str_expr`/`is_list_expr`/`expr_kind` (~109 lines) — discriminator dispatch via direct `match` is now the rule
- 147 tests pass, `cargo fmt --check` + `cargo clippy --release -- -D warnings` clean

---

### v0.90 — 2026-05-28 — Migration via `--import`: drop ~30 accessor helpers, direct `Expr.*` construction (stage 2)

Cashed in the v0.89 `--import` bootstrap by migrating the three remaining compiler modules to direct match destructures and direct enum construction. Net diff: **+142 / −625** across 6 files (~480 lines removed).

- `build.rs`: now passes `--import src/ast.hom` when compiling every other `.hom` file, so codegen knows `Expr`/`TypeExpr` variant field types cross-file (F9 match auto-deref + F12 construct auto-wrap)
- `parser.hom`: replaced ~25 `mk_expr_*` / `mk_type_*` / `some_expr` wrapper calls with direct `Expr.Field(base, name)` / `Expr.BinOp(BinOp.Eq, lhs, rhs)` / `Some(...)` literals — F12 emits `Box::new` at construct sites automatically
- `parser_imp.rs`: deleted 23 `mk_expr_*` / `mk_type_*` / `some_*` / `none_*` constructor helpers (−221 lines)
- `codegen.hom` + `sema.hom`: replaced ~25 `expr_*` accessor calls (`expr_field_expr`, `expr_binop_lhs`, `expr_for_var`, `expr_block_stmts`, …) with direct positional bindings in match arms
- `dep/codegen_helpers.rs`: deleted ~30 zero-caller `expr_*` / `type_*` accessor helpers (−245 lines); kept `expr_is_lambda` for the Lambda early-out (named fields can't be positionally bound)
- 147 tests pass, `cargo fmt` + `cargo clippy --release -- -D warnings` clean

---

### v0.89 — 2026-05-28 — `--import` flag for cross-file enum awareness (stage 1)

Stage-1 bootstrap: ship the `--import` infrastructure compiling cleanly from the v0.88 bootstrap. Stage 2 (v0.90) will use the new homunc to migrate `parser.hom`/`codegen.hom`/`sema.hom` from `mk_expr_*` / `expr_*` accessors to direct `Expr.*` construction and direct match destructures.

- `--import <file.hom>` CLI flag: pre-parses an external `.hom` and registers its enum variant field types into the codegen registry, so F9 (match auto-deref) and F12 (construct auto-wrap) work cross-file
- `main.hom`: `collect_imports` / `filter_imports` / `process_imports` consume `--import` pairs from argv and run lex → parse → `register_external_types` per file
- `main_imp.rs`: new `read_file` (exits on I/O error) + Rust shim for `register_external_types`
- `codegen.hom`: new `register_external_types(stmts)` walks `Stmt.EnumDef` and calls the existing `register_variant_field_types` registry
- 147 tests pass, `cargo fmt` + `cargo clippy --release -- -D warnings` clean

---

### v0.88 — 2026-05-07 — F12 construct-side Box auto-wrap, F9 cross-file fix, `gen/` → `src/`

- **F12** construct-side `Box<T>` auto-wrap for self-recursive enum variants (mirrors F9's match-side rebinds); reuses the `variant_field_types` registry
- Migrated 12 accessor sites in `codegen.hom`/`sema.hom` to direct positional bindings (F9 was already cross-file safe)
- Bin entry shim moved from generated `gen/main_entry.rs` to committed `src/main_entry.rs`; `build.rs::generate_bin_entry()` deleted
- 147 tests pass

---

### v0.87 — 2026-05-06 — Audit follow-ups (9-ticket bot batch)

- `codegen.hom` dedupe: extracted `cg_bind_common`/`cg_top_bind`, `cg_str_or_expr`, `infer_const_ty`; killed ~12 `expr_kind` callsites
- `hom-std/heap.rs`: dropped `Rc<RefCell<BinaryHeap>>` for plain `BinaryHeap` (registered as `&mut Heap` callee)
- `_imp.rs` shrink: migrated 11 pure-logic helpers from `parser_imp.rs`/`lexer_imp.rs` to `.hom`; added `is_upper` to `chars`
- `lib.rs`: collapsed 6 test `mod` blocks via `embed_test_mod!` macro; removed 4 obsolete `#![allow(...)]` lints
- CI fix: `gen/main_entry.rs` shim tracked so `cargo fmt --check` works on fresh clone
- Net diff +220 / −374 across 19 files; bot shipped 9/9 tickets in ~1h29min

---

### v0.86 — 2026-05-05 — `chars` consolidation, kill `src/main.rs`, hoist `hom-std` tests

- `hom-std/chars.rs`: consolidated single-char predicates from `lexer_imp.rs` (`is_alpha`/`is_digit`/`is_alnum`/`is_whitespace`/`is_newline`); `lexer.hom` now `use chars`
- `src/main.rs` deleted; `build.rs` writes a 3-line shim to `gen/main_entry.rs`, entry lives entirely in `main.hom` + `main_imp.rs`
- 623 lines of inline `#[cfg(test)]` blocks in `hom-std/` hoisted to `tests/std-tests/`; new `test_set.rs`/`test_fs.rs`/`test_path.rs`
- 175 tests pass; deeper migration tickets (R4c parser_imp, R6 resolver) hit 900s worker timeout, carried to v0.87

---

### v0.85 — 2026-05-05 — Direct-match `Expr`/`TypeExpr` dispatch, set/dict mutation stdlib

- `sema.hom` + `codegen.hom`: `expr_kind`/`type_kind` discriminator calls replaced with direct `match` arms; Box-bearing arms kept accessor calls (pre-F9-cross-file)
- `dep/codegen_helpers.rs`: deleted 6 zero-caller `expr_*` accessors; added `register_variant_field_types`/`variant_field_types_get` registry for cross-file Box auto-deref
- New `hom-std/set.rs` (`set_new`/`set_add`/`set_remove`/`set_clear`) + `dict_insert`/`dict_remove`/`dict_clear` — all `&mut`-registered
- New fixtures: `box_match.hom`, `cross_file_box_match/`, `char_builtins/`, `set_dict_mut/`
- 199 tests pass

---

### v0.84 — 2026-04-30 — `ast.rs` → `ast.hom` + delete `dep/ast_access.rs`

Cashed in v0.82's features (multi-payload variants, or-patterns, `@derive` auto-Box) on the two highest-leverage targets v0.83 deferred. Hom:Rs ratio: **0.77 → 1.18 → 1.45**.

- `src/ast.rs` (207 lines) → `src/ast.hom` (~80 lines): 12 AST types with `@derive(Clone, Debug)`; auto-Box through `Option<T>`/`Tuple`
- `resolver.hom`/`sema.hom`/`codegen.hom`: replaced `stmt_*`/`pat_*`/`arm_*`/`param_*`/`fielddef_*`/`variantdef_*` accessors with direct `match` + field access; `expr_*`/`type_*` accessors kept for Box-deref
- `src/dep/ast_access.rs` (964 lines) **deleted**; 53 needed Box-deref helpers moved into `codegen_helpers.rs`
- Net: **−409 lines** of `dep/` Rust; 182 tests pass; 5 sub-tickets shipped by claude-bot in ~85 min

---

### v0.83 — 2026-04-29 — Hom:Rs ratio rebalanced via 10 self-host reductions

Applied v0.82's language additions (multi-payload variants, or-patterns, `@derive`, `@thread_local`, `path`/`fs` stdlib, explicit generics) to shrink `_imp.rs` and `dep/` shims.

- Lexer: `Pos`/`Token`/`TokenKind` migrated to `lexer.hom`; 8 `make_token_*` constructors deleted
- Parser: ~60 trivial `mk_*` AST constructors inlined in `parser.hom`; state (`parse_pos`/`parse_err`/`gensym_counter`) made `@thread_local`
- Resolver: `Found`/`ResolverState`/`ResolvedFile` migrated to `resolver.hom`; `read_file`/`path_*` wrappers replaced with stdlib
- Codegen: 8 helpers (`parse_interp`, `escape_str`, `codegen_string`, `codegen_type`, …) moved to `codegen.hom`; fn-signature registry split (logic in `.hom`, state in `_imp.rs`)
- Scope: `dep/scope.rs` (64 lines) → `src/scope.hom` (23 lines) + thin `scope_imp.rs`
- Main pipeline: `compile_source`/`compile_file` migrated to `main.hom`
- Net ~**−1,000 lines** of `.rs`; 182 tests pass
- Deferred: `src/ast.rs` → `src/ast.hom` (blocked on `codegen_type_variant_field` Option-wrapping; landed in v0.84)

---

### v0.82 — 2026-04-29 — Explicit generics syntax `<T: Trait>`

- Added `<T: Trait + Trait, U: Trait>(params) -> ret { body }` lambda syntax for explicit generic constraints
- Parser: `parse_atom` handles `Lt` token to enter `parse_generic_lambda`; `parse_generic_param_list` builds pre-rendered constraint strings (e.g. `"T: Hash + Eq"`)
- AST: `Expr::Lambda` gains `generics: Vec<Name>` field (pre-rendered constraint strings; empty = implicit inference as before)
- Codegen: `cg_top_fn` uses explicit generics when provided, falls back to `infer_generics` for untyped params otherwise
- Sema: mixing explicit `<T:...>` with implicit untyped params in the same fn is now a compile error
- Existing untyped-params → `<T: Clone>` inference unchanged

---

### v0.81 — 2026-04-27 — `@attr` syntax for Rust attribute passthrough

- Added `@<content>` outer attribute → emits Rust `#[<content>]` above the next struct, enum, or top-level fn binding
- Added `@!<content>` inner attribute → emits Rust `#![<content>]` at output top (file-level)
- Pure token-stream passthrough — no Homun-side interpretation; copy-paste from Rust docs works (`@derive(Clone, Debug)`, `@cfg(any(unix, target_os = "macos"))`, `@inline`, `@!allow(dead_code)`)
- Lexer: `!` added as `Bang` token; `@` unchanged; collection literals `@[..]` / `@{..}` keep working (parser disambiguates by position)
- AST: `Stmt::StructDef` / `Stmt::EnumDef` / `Stmt::Bind` carry `attrs: Vec<String>`; new `Stmt::InnerAttr(String)` variant
- Parser: `parse_program` drains `@` / `@!` before each decl; `@!` errors if it appears after the first non-attr decl
- Codegen: `emit_attrs()` helper prints attrs above struct/enum/fn; inner attrs collected and emitted as `#![..]` at output top
- Examples: `attr_derive.hom`, `attr_cfg.hom` — registered in `tests/examples.rs`

---

### v0.80 — 2026-03-29 — Examples, dedup use lines, IO docs

- Added `top_k_words.hom` example — Top K Frequent Words using std + re + heap
- Added `test_io.hom` example — IO integration test (write_file, read_file)
- Added `dedup_use_lines()` — fixes duplicate `use` imports when multiple embedded libs are inlined
- Updated README.md, llm.txt with new examples and IO API docs

---

### v0.79 — 2026-03-28 — Embedded runtime, remove hom submodule

- Removed `src/hom` git submodule — runtime files now in `hom-std/` as regular tracked files
- Removed `.gitmodules` — no submodule dependencies
- Runtime tests moved from `hom-std/tests/` to `tests/std-tests/`
- Added `--emit-runtime` flag — prints full embedded runtime (builtin + std + re + heap + chars + str_ext + dict) to stdout
- Multi-module Cargo projects no longer need `hom/` submodule: `homunc --emit-runtime > src/runtime.rs`
- Updated `build.rs` and `lib.rs` paths from `src/hom/` to `hom-std/`
- Added `top_k_words.hom` example — Top K Frequent Words using std + re + heap
- Added `test_io.hom` example — IO integration test (write_file, read_file)
- Added `dedup_use_lines()` — fixes duplicate `use` imports when multiple embedded libs are inlined
- Updated README.md, llm.txt with new examples and IO API docs

---

### v0.78 — 2026-03-21 — Clone elision in hom-std, extract tests, heap &mut

- `src/hom/` tests extracted from inline `#[cfg(test)]` blocks into `src/hom/tests/test_*.rs`
- `heap_push/pop/len/is_empty` signatures changed from owned `Heap` to `&mut Heap`
- `count()` and `unique()` in `collection.rs`: defer `.cloned()` to after filter — one clone instead of two
- `filter!` macro in `mod.rs`: same pattern — `filter` then `.cloned().collect()`
- Registered heap builtins in `register_known_dep_fns()` so codegen emits `&mut` for `.hom` callers

---

### v0.77 — 2026-03-19 — Tuple patterns require `()`, removed bare comma patterns

- Match tuple patterns now require explicit parentheses: `(0, _, _) ->` instead of `0, _, _ ->`
- Aligns with Rust tuple pattern syntax
- Removed `parse_more_pats` from parser — tuple patterns handled by `parse_pat` `(` branch

---

### v0.76 — 2026-03-19 — Set syntax `@{a, b}`, struct destructuring, removed `@()`

- Set literals now use `@{a, b, c}` (curly braces) instead of `@(a, b, c)` (parens)
- `@{k: v}` is dict (has colons), `@{a, b}` is set (no colons) — parser auto-detects
- `@()` removed — no longer accepted
- `@{}` empty → empty dict
- Added struct rename destructuring: `{x: a, y: b} := pos` desugars to field access bindings
- Updated README, llm.txt, Compiler.md, main.hom help text

---

### v0.75 — 2026-03-18 — Self-host parser + `::` params + CI overhaul

- **Parser self-hosted**: replaced 1090-line `parser.rs` with `parser.hom` + `parser_imp.rs` (thread-local state, error propagation via no-op pattern)
- **`::` params in lexer.hom**: 6 helper functions (`ls_skip_line_comment`, `ls_skip_block_comment`, `ls_read_string`, `ls_read_char_lit`, `ls_read_number`, `ls_read_ident`) use `state::LexState` — mutate in place, return only the payload
- **`::` params in resolver.hom**: `resolve_file(rs::ResolverState, ...)` — `&mut` instead of `Rc<RefCell<>>`
- `resolver_imp.rs`: ResolverState is now plain struct — removed `Rc<RefCell<>>` wrapper
- `dep/scope.rs`: Scope is now plain `HashSet<String>` — removed `Rc<RefCell<>>` wrapper
- `codegen.hom`: `::` param reassignment emits `*name = rhs` (deref)
- `codegen_helpers.rs`: added `set_current_mut_ref_params` / `is_mut_ref_param` for `::` deref tracking
- **CI**: Docker-based test workflow (`Dockerfile.test`) — no stale cache issues
- **test_hom_std_re**: un-ignored, compiles via temp Cargo project with `regex` dep
- **Bootstrap fix**: v0.75.4 two-stage release to break `::` deref chicken-and-egg cycle

### v0.74 — 2026-03-18 — Scope functional pattern + codegen refactor

- `scope_insert` returns modified Scope (functional pattern) instead of mutating shared ref
- `sema.hom`: all scope_insert calls capture return value
- `codegen.hom`: `cg_stmts` returns `(lines, scope)` tuple, scope tracking moved from `cg_stmt`

---

### v0.73 — 2026-03-18 — Fix Tuple Destructuring Reassignment

- BindPat/BindPatMut codegen now checks if ALL names in a tuple pattern are already in scope
- If all exist: emits `(a, b) = rhs;` (reassignment) instead of `let (mut a, mut b) = rhs;` (shadow)
- Fixes incorrect variable shadowing when re-destructuring into existing names

---

### v0.72 — 2026-03-18 — Self-Hosted Source Migration to `->` Match Arms

- Converted all `src/*.hom` match arms from `=>` to `->` (codegen, sema, lexer, resolver, main)
- Parser now requires `->` (Arrow) only — `=>` (FatArrow) no longer accepted in match arms
- Early return `=> expr` syntax unchanged

---

### v0.71 — 2026-03-17 — Match Arm Arrow Syntax (`=>` → `->`)

- Parser now accepts both `->` (Arrow) and `=>` (FatArrow) in match arms — full backward compatibility
- All `_site/examples/*.hom` match arms updated to `->` syntax
- README.md and `_site/llm.txt` documentation examples updated to `->` syntax

---

### v0.70 — Self-Hosted Source Upgrade & Old-Syntax Removal

- Converted all `src/*.hom` to new syntax: removed `do` from `if`/`for`/`while`, `break => val` → `=> val`, `param ::= Type` → `param::Type`
- Removed old-syntax compatibility from parser: `do` token no longer accepted after control flow, `break =>` no longer parsed, `name ::= Type` param form removed
- All 5 self-hosted modules (`lexer`, `codegen`, `sema`, `resolver`, `main`) now use new syntax exclusively

---

### v0.69 — Syntax Overhaul: EarlyReturn, Optional `do`, New Param Forms

- Added `EarlyReturn(Box<Expr>)` to AST and `=> expr` early-return syntax
- Added `DoubleColon` (`::`) token; `::param` form for mutable params
- Made `do` keyword optional in `if`/`for`/`while` control flow
- Parser accepts 4 param forms: `:` (typed), `::` (mutable), `:=` (default), `::=` (mutable+default)
- Codegen emits `return <val>` for EarlyReturn, `break` for Break
- Param defaults: missing trailing args filled from default exprs at call site
- Sema updated to check EarlyReturn nodes
- All `_site/examples/*.hom` converted to new syntax (no `do`, `=> val` early return)

---

### v0.63 Mutable Assign Operator

Added MutAssign operator ::= to Homun
Mutable params emit ref-mut T sig, call sites emit ref-mut arg
build.rs detects imp.rs changes, re-triggers hom recompilation

---

### v0.62 — Full Self-Hosting (Lexer, Resolver, Main)

- Ported lexer to Homun: `src/lexer.hom` + `src/lexer_imp.rs` (LexState helpers, token constructors, char-testing)
- Ported resolver to Homun: `src/resolver.hom` + `src/resolver_imp.rs` (DFS three-color algorithm, I/O wrappers, pipeline helpers)
- Ported main to Homun: `src/main.hom` + `src/main_imp.rs` (CLI arg parsing, compile pipelines, stdin/file/stdout modes)
- All 5 compiler modules now self-hosted in `.hom`: codegen, sema, lexer, resolver, main (only parser.rs and ast.rs remain in Rust)
- `src/main.rs` reduced to thin wrapper calling `homunc::main_hom::main()`
- Deleted `runtime/` directory; moved test `.hom` files to `_site/examples/`

---

### v0.61 — Char Type & Naming Conventions

- Added `char` type: `'x'` literals map to Rust `char`, supports escapes (`'\n'`, `'\t'`, `'\\'`, `'\0'`)
- Char in all compiler stages: lexer (`TokenKind::Char`), AST (`Expr::Char`), parser, codegen, type inference
- Type annotation `char` maps to Rust `char`; top-level char bindings infer `pub const X: char`
- Renamed `_dep.rs` → `_imp.rs` convention (`codegen_imp.rs`, `sema_imp.rs`)
- Fixed CI: added missing `src/hom` submodule pointer, converted `_site/examples/hom` from symlink to submodule

---

### v0.60 — Self-Hosting Compiler

- Rewrote `codegen.rs` and `sema.rs` in Homun (`src/codegen.hom`, `src/sema.hom`) — compiler now compiles itself
- `build.rs` bootstraps: downloads released `homunc` binary, compiles `.hom` sources to `.rs` into `OUT_DIR`
- Added `src/dep/` Rust helper modules (`ast_access.rs`, `codegen_helpers.rs`, `scope.rs`) used by `.hom` codegen/sema via `use` imports
- Added `src/lib.rs` exposing compiler as a library crate
- Moved hom-std submodule from `hom/` to `src/hom/` for cleaner Cargo layout
- Cargo edition upgraded to 2024

---

### v0.51 — Const Type Inference & CI Submodule Support

**Codegen:**
- Top-level bindings now infer const types: `name := "foo"` emits `pub const NAME: &str` instead of `pub const NAME: _`
- Supports `&str`, `i32`, `f32`, `bool`; other expressions fall back to `_`

**CI/CD:**
- All workflows (ci, release, cd) now use `submodules: recursive` for `actions/checkout`
- CD auto-triggers after "Build and Release" succeeds

---

### v0.50 — hom-std Submodule, --module Flag & Integration Tests

**Runtime refactor:**
- Extracted runtime libraries into homun-std git submodule at `hom/`
- Compiler now reads runtime from `hom/` submodule (`include_str!("../hom/...")`) instead of `runtime/`
- `use std`, `use re`, `use heap` etc. — compiler resolves to `hom/std/`, `hom/re.rs`, `hom/heap.rs`
- Standalone usage unchanged — runtime embedded in `homunc` binary, no `hom/` needed on disk

**Multi-module Cargo support:**
- Added `--module` flag — skips preamble and runtime embedding for multi-module Cargo projects
- Projects add `homun-std` as submodule, `build.rs` concatenates `hom/*.rs` into shared `runtime.rs`
- Added `PartialEq` to all derived traits (structs and enums)

**Testing:**
- Added `tests/examples.rs` — compiles and runs all `_site/examples/*.hom` (7 tests)
- Added `tests/hom_std.rs` — compiles and runs runtime test files (chars, heap)
- 24 total tests (15 unit + 7 examples + 2 hom-std)

**Submodule setup:**
- `hom/` — homun-std runtime (builtin, std, re, heap, chars, str_ext, dict)
- `_site/examples/hom/` — same submodule, so examples can `use std` via resolver

---

### v0.43 — Codegen Fixes, Parser Hardening & Examples

**Codegen fixes:**
- Enum variant access emits `::` instead of `.` — `Direction.TD` → `Direction::TD`
- Enum variant match patterns emit `::` — `Direction.LR =>` works correctly
- String concatenation: `"str" + expr` wraps with `.to_string()` / `&` for Rust compatibility

**Parser improvements:**
- Tuple return types `(int, int)` now parsed correctly in function signatures
- Same-line guard for `[` — prevents `expr\n[list]` from being parsed as indexing

**Simplification:**
- Removed snake_case naming enforcement from semantic analysis — any naming style now allowed

**Examples:**
- Added 5 end-to-end examples in `examples/`:
  - `enum_variant.hom` — enum definitions, variant access, match patterns
  - `struct_basic.hom` — struct definition, construction, field access
  - `match_pattern.hom` — match with literals, enums, Ok/Err constructors
  - `tuple_destructure.hom` — tuple bind, wildcard `_`, multi-return
  - `collection.hom` — `@[]` lists, for loops, filter/map with pipe `|`

### v0.41 — Pattern Matching, Result/Option & Runtime Libraries

**Language features (part-a):**
- `?` postfix TryUnwrap operator for Result/Option propagation
- `Ok(x)` / `Err(msg)` / `Some(x)` recognized as sema builtins
- `TypeExpr::Generic` parsing and codegen (e.g., `Result<i32, String>`)
- Tuple destructuring bind in let statements
- Tuple patterns in match arms
- Nested constructor patterns in match arms (`Ok(x)`, `Err(msg)`, `Some(x)`)
- Mutable nested indexing via `Stmt::Assign` for lvalue assignment
- `.to_string()` fix for `Str` arguments

**Runtime libraries (part-b):**
- `heap.rs` — priority queue wrapping `BinaryHeap` with `Reverse`; `heap_push` accepts `&str`; `heap_is_empty`, `heap_pop` returns `Option`
- `re.rs` — regex pattern matching with thread-local caching (`re_match`, `re_is_match`)
- `chars.rs` — character classification (`is_alpha`, `is_alnum`, `is_digit`, `is_ws`)
- `str_ext.rs` — `str_repeat`, `str_pad_center`
- `dict.rs` — `dict_from_pairs`, `dict_zip`, `dict_clone` (HashMap helpers)

**Testing & docs:**
- Integration tests: multi-file `.hom` using `?`, `Ok`/`Err`, tuple destruct, nested match, nested index
- `.hom` integration tests for chars, re, and heap libraries
- Updated `llm.txt` with new runtime library documentation

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

---
**homunc + rustc Era (v0.40+)**
---

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
**Rust Compiler Era (v0.30+)**
---

### v0.29 — File Reorganization

- Moved examples to `_site/examples/`
- Prepared directory layout for Rust rewrite

### v0.28 — Standard Library Extensions

- Added `std_collection.rs`, `std_math.rs`, `std_str.rs`
- Expanded `Compiler.md` with standard library docs

### v0.27 — Playground Refinements

- Refined WASM playground UI and `Main.hs`


---
**Haskell Compiler Era (v0.23–v0.29, `haskell` branch)**
---

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
