# Homun Script Reference

Homun is a scripting layer for a Rust-based ECS game engine. Every valid Homun program transpiles 1-to-1 to Rust. 
For performance-critical code and architecture, you write Rust directly. 
Inspired by Svelte, Homun gives game designers a lighter syntax for gameplay scripts without writing raw Rust. 
Homun is not a language. It is a template-instantiation embed script. Thus, Hindley-Milner is handle by rust.

---

## Examples

see _site/examples

```
// Valid Parentheses
is_valid := (s) -> {
  stack := @[]
  pairs := @{")" : "(", "]" : "[", "}" : "{"}
  for ch in s do {
    if (ch in "([{") do {
      stack := stack + @[ch]
    } else {
      if (len(stack) == 0 or stack[-1] != pairs[ch]) do { break => false }
      stack := stack[:-1]
    }
  }
  len(stack) == 0
}

// Quicksort — recursion + pipe
quicksort := (arr) -> {
  if (len(arr) <= 1) do { break => arr }
  pivot := arr[0]
  rest  := arr[1:]
  left  := rest | filter((x) -> { x <= pivot })
  right := rest | filter((x) -> { x > pivot })
  quicksort(left) + @[pivot] + quicksort(right)
}

// DFS — recursive
dfs := (graph, node, visited) -> {
  if (node in visited) do { break => visited }
  visited := visited + @[node]
  for neighbor in graph[node] do {
    visited := dfs(graph, neighbor, visited)
  }
  visited
}


// Pattern Match
fizz_buzz := (n: int) -> @[str] {
  result := @[]
  for i in range(1, n+1) do {
    value := match (i % 15, i % 3, i % 5) {
      0, _, _ => "FizzBuzz"
      _, 0, _ => "Fizz"
      _, _, 0 => "Buzz"
      _         => str(i)
    }
    result := result + @[value]
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

// Mutable variables — ::= emits let mut, allows reassignment
counter ::= 0
counter ::= counter + 1    // reassignment (Rust: counter = counter + 1)

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
| `none` | `none` | Missing value. Use `match` to handle. |

`-> _` is the void return (Rust `()`). `none` is absence (Rust `Option::None`). They are different.

---

## Operators

No bare `=` exists. `:=` and `::=` bind, `==` compares. No ambiguity.

| Operator | Meaning |
|---|---|
| `:=` | Bind / rebind (immutable — Rust `let`) |
| `::=` | Bind / rebind (mutable — Rust `let mut`) |
| `==`, `!=`, `<`, `>`, `<=`, `>=` | Comparison |
| `and`, `or`, `not` | Boolean (Python-style) |
| `in` | Membership (lists, sets, dict keys). Negate: `not x in s` |
| `+`, `-`, `*`, `/`, `%` | Arithmetic |
| `\|` | Pipe — `x \| f(args)` desugars to `f(x, args)` |
| `.` | Field access / lambda-field call |

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

// Mutable reference params — ::= in param position
increment := (c ::= Counter) -> _ { c.value := c.value + 1 }
// compiles to: fn increment(c: &mut Counter) { c.value = c.value + 1; }
// call sites auto-pass &mut: increment(my_counter)
```

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
fib := (n) -> { if (n <= 1) do { n } else { fib(n-1) + fib(n-2) } }
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
flags  := @("fire", "ice", "poison")  // set (unique, unordered)
empty  := @[]                          // type from first use
```

Dict access returns `none` on missing key:

```
val := scores["alice"]     // value or none
scores["bob"] := 99        // update or insert

match scores["unknown"] {
  none => print("not found")
  val  => print("score: ${val}")
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

Condition blocks use `do` before `{`. Bare `else` has no `do`.

### `if` / `else`

No `elif`. Use `match` for multi-branch.

```
if (hp <= 0) do { die() } else { recover() }

// multi-branch
match true {
  _ if hp <= 0  => die()
  _ if hp < 20  => warn()
  _             => recover()
}
```

Boolean operators: `and`, `or`, `not`, `in`.

### `match`

Expression — result can be assigned. `_` is wildcard. Compiler warns on non-exhaustive matches.

```
dmg := match element {
  Element.Fire(power) => power * 2
  Element.Ice(power)  => power * 1.5
  _                   => 0
}

match find_target(pos) {
  none   => idle()
  target => attack(target)
}
```

---

## Loops

```
for i in range(10) do { print(i) }
for item in inventory do { use(item) }

enemies ::= 10
while (enemies > 0) do { enemies ::= enemies - 1 }
```

`break` exits loop. `continue` skips iteration. Both work inside `match` blocks.

### `break => value` — Early Return

`break` exits a loop. `break => value` exits with a value.

```
// Two Sum
two_sum := (nums: @[], target) -> @[] {
  seen := @{}
  for i in range(len(nums)) do {
    comp := target - nums[i]
    if (comp in seen) do { break => @[seen[comp], i] }
    seen[nums[i]] := i
  }
}
```

`break => _` exits a void.

---

## Destructuring

Multiple bindings on left side of `:=` or `::=`. Right side fully evaluated first.

```
a, b    := b, a              // swap (immutable)
_, b    := get_pair()        // discard first
x, y    := y, x + y          // Fibonacci step

a, b    ::= b, a             // swap (mutable — lets you reassign a, b later)

{ name, hp, _ } := player    // struct destructure, skip speed
{ x, _, z } := pos           // skip y
```

`_` discards — no binding created.

---

## Error Handling

No try/catch, no exceptions. The Rust engine wraps every script in an error boundary (like Unity).
Runtime failures are caught, logged, and the game keeps running.

Use `none` + `match` for expected absence. For complex error states, model as enum variants.
For structured error handling, write it in Rust.

---

## Structs

No classes. Data = structs. Behavior = lambdas that accept structs.

```
Player := struct { name: str, hp: int, speed: float }
p := Player { name: "Aria", hp: 100, speed: 3.5 }
p.hp := p.hp - 10           // field mutation

pos := { x: 1.0, y: 2.0 }  // anonymous struct

// For in-place mutation via functions, use ::= :
Counter := struct { value: int }
c ::= Counter { value: 0 }           // mutable binding
add_n := (c ::= Counter, n: int) -> _ { c.value := c.value + n }
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
  Direction.North => move(0, 1)
  Direction.South => move(0, -1)
  _               => idle()
}
```

`match` is exhaustive — compiler warns if not all variants are covered and no `_` exists.

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

### Standard library (`hom/`)

Runtime libraries live in the `hom/` submodule ([homun-std](https://github.com/HomunMage/homun-std)).
User writes `use std` — the compiler resolves it to `hom/std/`.

```
use std    // provides: range, len, filter, map, reduce, split, join, abs, min, max, ...
use re     // regex: re_match, re_is_match, re_replace, re_split
use heap   // priority queue: heap_new, heap_push, heap_pop, heap_peek
use chars  // char utils: is_alpha, is_digit, is_alnum, is_space
```

**Standalone usage** — the runtime is embedded in the `homunc` binary. No external dependencies needed:

```bash
homunc main.hom -o main.rs
rustc main.rs -o main        # self-contained, no hom/ needed
```

**Multi-module Cargo projects** — add [homun-std](https://github.com/HomunMage/homun-std) as a git submodule so all modules share one runtime:

```bash
git submodule add https://github.com/HomunMage/homun-std.git src/hom
```

Then in `build.rs`, concatenate `src/hom/*.rs` into a shared `runtime.rs`.
Each `.hom` module is compiled with `homunc --module` which strips runtime embedding:

```bash
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
- `tests/hom_std.rs` — compiles and runs `runtime/test_*.hom` (hom-std tests)

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
move := (pos ::= Position, vel: Velocity, dt: float) -> _ {
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
move_system := (pos ::= Position, vel: Velocity, dt: float) -> _ {
  pos.x := pos.x + vel.dx * dt
  pos.y := pos.y + vel.dy * dt
}

damage_system := (health ::= Health, amount: int) -> _ {
  health.hp := max(0, health.hp - amount)
}

// The Rust engine handles entity storage and system scheduling.
// Homun scripts define components (data) and systems (logic).
```

### Design Philosophy

Homun delegates type checking to `rustc`. It does not reimplement Rust's type system — it generates Rust code and lets the Rust compiler enforce correctness. This is why Homun needs zero special syntax for traits, generics, or derive: **Rust already has them, and Homun compiles to Rust.**