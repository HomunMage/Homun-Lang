# Homun Script Reference


Homun is a scripting layer for a Rust-based ECS game engine. 
Homun gives game designers a lighter syntax for gameplay scripts without writing raw Rust. 
For performance-critical code and architecture, you write Rust directly. 
Homun is hemi-self-hosting script, never selfhost. It is a template-instantiation embed script. Every valid Homun program transpiles 1-to-1 to Rust. Thus, Hindley-Milner is handle by rust.

---

## Examples

Playground at : [https://homun.posetmage.com/Homun-Lang/](https://homun.posetmage.com/Homun-Lang/)

see _site/examples

```
// Valid Parentheses
is_valid := (s: str) -> bool {
  stack ::= @[]                  // ::= — pushed/popped below
  for ch in chars(s) {
    if (ch == "(" or ch == "[" or ch == "{") {
      stack ::= stack + @[ch]
    } else {
      if (len(stack) == 0) { => false }
      top := stack[-1]
      ok := (ch == ")" and top == "(")
        or (ch == "]" and top == "[")
        or (ch == "}" and top == "{")
      if (not ok) { => false }
      stack ::= stack[:-1]       // pop
    }
  }
  len(stack) == 0
}

// Quicksort — recursion + pipe
quicksort := (arr) -> {
  if (len(arr) <= 1) { => arr }
  pivot := arr[0]
  rest  := arr[1:]
  left  := rest | filter((x) -> { x <= pivot })
  right := rest | filter((x) -> { x > pivot })
  quicksort(left) + @[pivot] + quicksort(right)
}

// DFS — recursive
dfs := (graph, node, visited) -> {
  if (node in visited) { => visited }
  visited ::= visited + @[node]
  for neighbor in graph[node] {
    visited ::= dfs(graph, neighbor, visited)
  }
  visited
}


// Pattern Match
fizz_buzz := (n: int) -> @[str] {
  result ::= @[]
  for i in range(1, n+1) {
    value := match (i % 3, i % 5) {
      (0, 0) -> "FizzBuzz"
      (0, _) -> "Fizz"
      (_, 0) -> "Buzz"
      _         -> str(i)
    }
    result ::= result + @[value]
  }
  result
}

// Top K Frequent Words — regex + heap + dict membership
use std
use re
use heap

top_k_words := (words: @[str], k: int) -> @[str] {
  keys ::= @[]
  counts ::= @[]
  for w in words {
    if (len(w) == 0) { continue }
    if (not re_is_match("^[a-zA-Z]+$", w)) { continue }
    found ::= false
    j ::= 0
    while (j < len(keys)) {
      if (keys[j] == w) {
        counts[j] := counts[j] + 1     // index assign — counts is ::= above
        found ::= true
      }
      j ::= j + 1
    }
    if (not found) {
      keys ::= keys + @[w]
      counts ::= counts + @[1]
    }
  }
  h ::= heap_new()
  i ::= 0
  while (i < len(keys)) {
    heap_push(h, 0 - counts[i], keys[i])
    i ::= i + 1
  }
  result ::= @[]
  i ::= 0
  while (i < k and not heap_is_empty(h)) {
    match heap_pop(h) {
      Some((_, word)) -> { result ::= result + @[word] }
      none -> {}
    }
    i ::= i + 1
  }
  result
}
```

---

## Basics

```
// Comments
// single-line comment
/* multi-line comment */

// Imports
use std                                // standard library
use engine::physics::{Vec2, RigidBody} // Rust pass-through

// Variables — := immutable, ::= mutable (no let/var/const)
x      := 10              // immutable (Rust: let)
name   := "hero"
speed  := float(3.14)
hp     := int(100)         // explicit type via constructor
// x := 20                 // ❌ compile error: ':=' cannot reassign an immutable binding

// Mutable variables — ::= emits let mut, allows reassignment
counter ::= 0
counter ::= counter + 1    // reassignment (Rust: counter = counter + 1)
// A value mutated in place (push, heap_push, set_add, field/index assign…)
// also needs ::= — it is being changed.

// String interpolation — any expression inside ${}
greeting := "Hello, ${name}! HP: ${hp * 2}"
```

### Primitive Types

| Type | Example | Notes |
|---|---|---|
| `int` | `42`, `int(42)` | 32-bit signed |
| `float` | `3.14`, `float(3.14)` | 32-bit |
| `bool` | `true`, `false` | |
| `str` | `"hello"` | UTF-8, `${}` interpolation |
| `char` | `'a'`, `'\n'` | Single character (single quotes) |
| `none` | `none` | Missing value. Use `match` to handle. |

`-> _` is the void return (Rust `()`). `none` is absence (Rust `Option::None`). They are different.

---

## Operators

No bare `=` exists. `:=` and `::=` bind, `==` compares. No ambiguity.

| Operator | Meaning |
|---|---|
| `:=` | Bind (immutable — Rust `let`). **Cannot reassign** an existing binding — use `::=` or a new name. |
| `::=` | Bind / reassign (mutable — Rust `let mut`) |
| `==`, `!=`, `<`, `>`, `<=`, `>=` | Comparison |
| `and`, `or`, `not` | Boolean (Python-style) |
| `in` | Membership (lists, sets, dict keys). Negate: `not x in s` |
| `+`, `-`, `*`, `/`, `%` | Arithmetic |
| `\|` | Pipe — `x \| f(args)` desugars to `f(x, args)` |
| `.` | Field access / lambda-field call |
| `@<x>` | Outer attribute — emits Rust `#[<x>]` above next decl (see [Attributes](#attributes-)) |
| `@!<x>` | Inner attribute — emits Rust `#![<x>]` at file top |
| `@[..]` / `@{..}` | Collection literal (Vec / dict / set) |

---

## Type Handling & Rust Delegation

Homun is a **template-instantiation language**, not a Hindley-Milner or traditional type-inference language. Its core job is to **compile into Rust**, and Rust handles `<T, U>` generics and monomorphization.

```
// Homun source code
apply := (f, x) -> { f(x) }

// Compiled to Rust code text
fn apply<T, U>(f: impl Fn(T) -> U, x: T) -> U {
    f(x)
}
```

> 💡 Note: Homun itself does **not** perform high-level type inference. It simply transpiles Homun programs into Rust, and **all generic `<T, U>` resolution and monomorphization are handled by Rust**.

---

## Lambdas

Every callable is a lambda. Braces always required. Last expression is the return value.

```
double    := (x) -> { x * 2 }                       // fully inferred
add       := (a, b) -> { a + b }                     // inferred from usage
add_typed := (a: int, b: int) -> int { a + b }      // explicit (optional)
log_event := (msg) -> _ { print(msg) }               // void (-> _)
tick      := () -> _ { update() }                    // no args

// Mutable reference params — :: in param position
increment := (c::Counter) -> _ { c.value := c.value + 1 }
// compiles to: fn increment(c: &mut Counter) { c.value = c.value + 1; }
// call sites auto-pass &mut: increment(my_counter)

// Param defaults — := immutable default, ::= mutable default
greet := (name:=str("world")) -> str { "Hello, ${name}!" }
greet()          // "Hello, world!"
greet("Alice")   // "Hello, Alice!"
```

### Parameter Forms

| Syntax | Meaning |
|---|---|
| `name:Type` | immutable param, no default |
| `name::Type` | mutable param (&mut), no default |
| `name:=expr` | immutable param, with default |
| `name::=expr` | mutable param (&mut), with default |

Return type goes between `->` and `{`: `-> {` inferred, `-> Type {` explicit, `-> _ {` void.

### Polymorphism

No special syntax. The compiler infers from call-site usage:

```
identity := (x) -> { x }
identity(42)       // int version
identity("hello")  // str version
```

### Recursion

Auto-detected — hidden `rec` mind.

```
fib := (n) -> { if (n <= 1) { n } else { fib(n-1) + fib(n-2) } }
```

### Lambdas as Values

```
transform := (x) -> { x * 2 }
doubled   := @[1, 2, 3] | map(transform)
```

---

## Pipe `|`

Pipes left-hand value as first argument into right-hand call. Same-line or multi-line.

```
result := @[1, 2, 3, 4, 5]
  | filter((x) -> { x > 2 })
  | map((x) -> { x * 10 })
  | reduce((a, b) -> { a + b })

// desugars to: reduce(map(filter(list, f), g), h)
```

`.` is always field access. `|` is always pipe. No overlap.

```
p.hp              // field read
e.on_tick()       // call lambda stored in field
p.hp | clamp(0, 100)   // pipe into function
```

---

## Collections

All prefixed with `@`. Bracket shape determines kind.

```
items  := @[1, 2, 3]                  // list (ordered, duplicates ok)
scores := @{"alice": 100, "bob": 80}  // dict (key-value)
flags  := @{"fire", "ice", "poison"}  // set (unique, unordered)
empty  := @[]                          // type from first use
```

Struct destructuring with rename:

```
Point := struct { x: int, y: int }
p := Point { x: 3, y: 4 }
{x: a, y: b} := p       // a == 3, b == 4
```

Dict access returns `none` on missing key:

```
val := scores["alice"]     // value or none
scores["bob"] := 99        // update or insert

match scores["unknown"] {
  none -> print("not found")
  val  -> print("score: ${val}")
}
```

---

## Slicing and Indexing

0-based. Python-style slicing: `[start:end:step]`, start inclusive, end exclusive.

```
x := @[10, 20, 30, 40, 50]
x[0]         // 10
x[-1]        // 50 (from end)
x[1:4]       // [20, 30, 40]
x[::2]       // [10, 30, 50]
x[::-1]      // reversed
```

### `range`

```
range(5)            // 0, 1, 2, 3, 4
range(3, 7)         // 3, 4, 5, 6
range(1, 10, 2)     // 1, 3, 5, 7, 9
range(10, 0, -1)    // countdown
```

---

## Control Flow

Condition in parens, then `{`. Bare `else` before `{`.

### `if` / `else`

No `elif`. Use `match` for multi-branch.

```
if (hp <= 0) { die() } else { recover() }

// multi-branch
match true {
  _ if hp <= 0  -> die()
  _ if hp < 20  -> warn()
  _             -> recover()
}
```

Boolean operators: `and`, `or`, `not`, `in`.

### `match`

Expression — result can be assigned. `_` is wildcard. Compiler warns on non-exhaustive matches.

```
dmg := match element {
  Element.Fire(power) -> power * 2
  Element.Ice(power)  -> power * 1.5
  _                   -> 0
}

match find_target(pos) {
  none   -> idle()
  target -> attack(target)
}

// Option matching with Some
match heap_pop(h) {
  Some((priority, item)) -> print("got: ${item}")
  none -> print("empty")
}
```

---

## Loops

```
for i in range(10) { print(i) }
for item in inventory { use(item) }

enemies ::= 10
while (enemies > 0) { enemies ::= enemies - 1 }
```

`break` exits loop. `continue` skips iteration. 

## Early Return: `=> value`

`=> value` returns early from the enclosing function/block.

There is no `return` keyword. The last expression in a block is its value. `=> value` is only for **early** exit — using a separate keyword return makes confusion with the implicit final-expression return.

```
// Two Sum
use dict
two_sum := (nums: @[int], target) -> @[int] {
  seen ::= @{}                       // value -> index (mutated, so ::=)
  for i in range(len(nums)) {
    comp := target - nums[i]
    if (comp in seen) { => @[seen[comp], i] }
    dict_insert(seen, nums[i], i)
  }
  @[]
}
```

`=> _` returns void early.

---

## Destructuring

Multiple bindings on left side of `:=` or `::=`. Right side fully evaluated first.

```
a, b    := b, a              // bind fresh a, b from a tuple (immutable)
_, b    := get_pair()        // discard first
x, y    := y, x + y          // Fibonacci step

a, b    ::= b, a             // reassign existing a, b (swap) — requires ::=

{ name, hp, _ } := player    // struct destructure, skip speed
{ x, _, z } := pos           // skip y
```

`_` discards — no binding created.

---

## Error Handling

No try/catch, no exceptions.

Use `none` + `match` for expected absence. For complex error states, model as enum variants.

`Result` + `?` for error propagation:

```
parse := (src: str) -> Result {
  if (len(src) == 0) { Err("empty") }
  else { Ok(build_ast(src)) }
}

// ? propagates errors (returns Err early)
node := parse_node(cursor)?
graph := build_graph(nodes)?
```

In ECS context, the Rust engine wraps scripts in an error boundary — runtime failures are caught, logged, and the game keeps running.

---

## Structs

No classes. Data = structs. Behavior = lambdas that accept structs.

```
Player := struct { name: str, hp: int, speed: float }
p := Player { name: "Aria", hp: 100, speed: 3.5 }
p.hp := p.hp - 10           // field mutation

pos := { x: 1.0, y: 2.0 }  // anonymous struct

// For in-place mutation via functions, use :: in params:
Counter := struct { value: int }
c ::= Counter { value: 0 }           // mutable binding
add_n := (c::Counter, n: int) -> _ { c.value := c.value + n }
add_n(c, 5)                           // c.value is now 5
```

**Data structs** (no lambda fields) are auto-serializable to RON. **Behavior structs** (has lambda
fields) are not. The compiler infers which kind from field types — you never declare it.

---

## Enums

Closed set of named variants, optionally carrying data.

```
Direction := enum { North, South, East, West }
Element   := enum { Fire(int), Ice(int), Neutral }

result := match dir {
  Direction.North -> move(0, 1)
  Direction.South -> move(0, -1)
  _               -> idle()
}
```

`match` is exhaustive — compiler warns if not all variants are covered and no `_` exists.

---

## Attributes (`@`)

`@<content>` and `@!<content>` are 1-to-1 passthroughs to Rust attributes. Homun does not interpret the body — copy-paste from Rust docs and crate READMEs works as-is.

| Syntax | Emits | Attaches to |
|---|---|---|
| `@<x>` | `#[<x>]` | next struct / enum / top-level fn binding |
| `@!<x>` | `#![<x>]` | the file (must appear at file top, before any decl) |
| `@[..]` / `@{..}` | unchanged | collection literals (parser disambiguates by position) |

```
@derive(Clone, Debug, PartialEq)
Player := struct { name: str, hp: int }

@inline
@must_use
sq := (x: int) -> int { x * x }

@cfg(any(unix, target_os = "macos"))
unix_only := () -> _ { print("posix") }

@!allow(dead_code)        // applies to whole compiled module
@!no_std
```

After `@` / `@!` the body is a Rust-token-stream passthrough zone — bare `=`, `::`, suffixed numerics, raw strings all flow through untouched. Only `()`, `[]`, `{}` must stay balanced; the body otherwise ends at end-of-line.

Multiple attrs stack in source order. Attribute names are not validated by Homun — `rustc` reports unknown attributes.

Out of scope (v0.81): attributes on local `let`, expressions, fn params, struct fields, enum variants. They are top-level decl-only.

---

## RON Integration

Data structs round-trip to/from RON automatically. `as Type` is required on load and is
compile-time validated. `as` is reserved exclusively for this — not a general cast.

```
level := load_ron("levels/world1.ron") as Map
save_ron(config, "config.ron")

// Homun struct literal ↔ RON (only @ prefix differs)
template := Enemy { name: "Goblin", hp: 30, loot: @["gold_coin"] }
// RON: Enemy(name: "Goblin", hp: 30, loot: ["gold_coin"])
```

Only data structs are RON-compatible. `save_ron` on a behavior struct is a compile error.

---

## Imports (`use`)

`use foo` resolves to exactly **one** of four candidates (relative to the current file's directory):

| Candidate | Type | What happens |
|---|---|---|
| `foo/mod.hom` | Homun folder | Compile `mod.hom` recursively; it can `use` siblings |
| `foo.hom` | Homun file | Compile and inline |
| `foo/mod.rs` | Rust folder | Read, expand, and inline |
| `foo.rs` | Rust file | Read, expand, and inline |

**Rules:**
- **0 matches** — falls through to Rust `use` (e.g. `use engine::physics`)
- **1 match** — resolved and inlined into the output
- **2+ matches** — compile error (ambiguous)

Only one form of `foo` may exist. If both `dog.hom` and `dog.rs` exist, or `dog/mod.hom` and `dog.rs`, the compiler errors.

### Folder namespaces

A folder acts as a namespace when it contains `mod.hom` (or `mod.rs`):

```
opencv/
  mod.hom          // entry point — use img
  img.hom          // exports image functions
  filter.hom       // exports filter functions
```

From outside: `use opencv` resolves to `opencv/mod.hom`, which pulls in siblings via normal `use`.

### Standard library (embedded in `homunc`)

All runtime libraries are embedded in the `homunc` binary. No external files or submodules needed.

```
use std    // provides: range, len, filter, map, reduce, split, join, abs, min, max, ...
use re     // regex: re_match, re_is_match
use heap   // priority queue: heap_new, heap_push, heap_pop, heap_len, heap_is_empty
use chars  // char utils: is_alpha, is_digit, is_alnum, is_whitespace, is_newline
use set    // set mutation: set_new, set_add, set_remove, set_clear (see llm.txt for full spec)
use fs     // filesystem: fs_read, fs_write, fs_exists, fs_is_dir
use path   // path utils: path_join, path_parent, path_canonicalize, path_strip_prefix
```

**Standalone usage** — just compile and run:

```bash
homunc main.hom -o main.rs
rustc main.rs -o main        # self-contained, no extra files needed
```

**Multi-module Cargo projects** — extract the runtime from `homunc`, then compile each module:

```bash
homunc --emit-runtime > src/runtime.rs
homunc --module src/types.hom -o out/types.rs
```

---

## Testing

Tests live in `tests/`:

```bash
cargo test                    # run all tests
cargo test --test examples    # run _site/examples/*.hom integration tests
cargo test --test hom_std     # run hom-std runtime tests
```

- `tests/examples.rs` — compiles and runs every `_site/examples/*.hom` file
- `tests/hom_std.rs` — compiles and runs hom-std test `.hom` files
- `tests/std-tests/` — unit tests for hom-std runtime modules

---

## Compiler Behavior

```
identity := (x) -> { x }

f := identity
a := f(1)
b := f("hi")   // ❌ compile error
g := identity
c := g("hi")   // success

nums := @[int](1,2,3)
empty := @[]              // ❌ compile error, never used


```

---

## ECS Design with Homun

Homun achieves ECS patterns without traits, generics, derive, or impl blocks — features that Rust requires but a scripting layer does not.

### The Problem

Rust ECS frameworks (Bevy, hecs, legion) rely on five language features:

| Rust Feature | ECS Purpose |
|---|---|
| `&mut` borrowing | Mutate components in-place |
| `trait` | Mark types as `Component`, `System`, `Resource` |
| `impl` blocks | Attach methods to structs |
| Generics `<T>` | Type-safe queries `Query<(&Pos, &mut Vel)>` |
| `#[derive(...)]` | Auto-generate trait implementations |

### How Homun Covers All Five

**1. Borrowing — `::=` operator (v0.63)**

```
// ::= gives &mut semantics, := keeps clone
move := (pos::Position, vel: Velocity, dt: float) -> _ {
  pos.x := pos.x + vel.dx * dt    // mutates in place
}
```

**2. Traits — struct declaration IS the trait**

A struct with specific fields acts as a contract. Any function that accepts it requires those fields to exist — duck typing enforced by `rustc`.

```
// This struct definition IS the "Movable trait"
Movable := struct { x: float, y: float, move: (float) -> _ }

// Any struct matching the shape works
Player := struct { x: float, y: float, move: (float) -> _, hp: int }
```

**3. Impl blocks — lambda fields in structs**

No need for `impl Foo for Bar`. Put the function directly in the struct.

```
p := Player {
  x: 0.0, y: 0.0,
  move: (dt) -> _ { print("moving ${dt}") },
  hp: 100
}
p.move(0.16)    // call like a method
```

**4. Generics — untyped params are generic**

```
identity := (x) -> { x }
a := identity(42)       // i32 version
b := identity("hello")  // String version
// Homun emits fn identity<T>(x: T) -> T, rustc monomorphizes
```

**5. Derive — generic functions replace it**

```
to_string := (x) -> str { "${x}" }   // works on any Display type
clone := (x) -> { x }                // works on any Clone type
// No #[derive] needed — the function IS the derive
```

### Putting It Together — ECS in Homun

```
// Components — just data
Position := struct { x: float, y: float }
Velocity := struct { dx: float, dy: float }
Health   := struct { hp: int, max_hp: int }

// Systems — just functions with ::= for mutation
move_system := (pos::Position, vel: Velocity, dt: float) -> _ {
  pos.x := pos.x + vel.dx * dt
  pos.y := pos.y + vel.dy * dt
}

damage_system := (health::Health, amount: int) -> _ {
  health.hp := max(0, health.hp - amount)
}

// The Rust engine handles entity storage and system scheduling.
// Homun scripts define components (data) and systems (logic).
```
