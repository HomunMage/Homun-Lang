# Homun Language Reference

> A scripting language for Rust game engines — 0-based, embeddable, minimal, and expressive.
> Compiles / transpiles directly to Rust.

---

## Overview

Homun is **not a new language**. It is a scripting layer for a Rust-based ECS game engine. For
large-scale systems, architecture, and performance-critical code, you write Rust directly — we are
still in the Rust game engine, and Rust remains the source of truth. Homun exists only to give
game designers and gameplay programmers a lighter syntax for client-side logic that doesn't require
writing raw Rust for every behavior script.

Every valid Homun program transpiles 1-to-1 to valid Rust code, so you get Rust's performance and
safety guarantees at runtime. Homun scripts can **import** Rust libraries exposed to the scripting
layer (similar to how Python imports C extensions via pyo3), giving scripts full access to the
engine's Rust API without reimplementing anything.

Because we target Rust and interop with Rust code constantly, Homun uses **0-based indexing** — the
same as Rust, C, and every language in the engine stack. Slicing follows **Python semantics**:
`[start:end]` where `start` is inclusive and `end` is exclusive.

The philosophy of Homun is radical simplicity:

- No classes, no interfaces, no inheritance — only structs and enums.
- No functions as a distinct construct — everything callable is a lambda.
- `use` imports from Rust libraries exposed to the scripting layer — like pyo3 for Python.
- No semicolons — newlines are enough.
- No mutable/immutable split in the declaration syntax — assignments are clear and uniform.
- No `=` operator at all — `:=` binds, `==` compares. There is no ambiguity between assignment and equality.
- All collections share the `@` prefix — lists `@[]`, dicts `@{}`, sets `@()` — visually consistent and parser-friendly.
- Lambdas use `(params) -> { body }` — braces always required, no expression-body shorthand.
- `|` is the pipe operator — explicit, same-line or multi-line, no whitespace sensitivity.
- `-> _` marks a void-returning lambda — `_` means "discard," consistent with destructuring.
- Strict naming conventions: variables and lambdas are `snake_case`, structs and enums are recommended as `PascalCase`.
- 0-based indexing throughout, matching Rust and the engine stack.
- Native RON (Rusty Object Notation) integration — data structs round-trip to/from RON automatically.

---

## Design Goals

| Goal | Decision |
|---|---|
| Familiar to Python/Go developers | `:=` assignment, Python keywords like `and`, `or`, `in`, `not` |
| No ceremony around functions | All callables are lambdas `(params) -> { ... }` |
| Safe mapping to Rust | No raw pointers, no unsafe constructs |
| Readable game logic | `if () do {}`, `elif`, `match` with `_` wildcard |
| Minimal syntax noise | No `;`, no `mut`, no bare `=` |
| Unambiguous equality | Only `==` for comparison — `:=` is the sole assignment form |
| Consistent collection syntax | All collections prefixed with `@`: `@[]` `@{}` `@()` |
| Explicit pipe operator | `\|` pipes a value into the next call — no whitespace-sensitive rules |
| Clear naming at a glance | Variables/lambdas `snake_case`, types `PascalCase` (recommended) |
| Rust-aligned indexing | 0-based, Python-style `[start:end)` slicing |
| Game data pipeline | Native RON support — data structs are format and code simultaneously |
| Transparent recursion | Compiler auto-detects recursive lambdas via two-stage parse — no `rec` keyword needed |
| Unambiguous void return | `-> _` means this lambda produces nothing — `_` is the discard marker, consistent with destructuring |

---

## Comments

Single-line comments use `//`. Multi-line comments use `/* */`.

```
// this is a single-line comment

/* this is a
   multi-line comment */

x := 10  // inline comment
```

---

## Imports

Homun uses `use` to import from Rust libraries exposed to the scripting layer. This works like
Rust's `use` statements and is similar to how Python imports C extensions via pyo3. The engine
provides a set of crates that are pre-built in Rust and made available to Homun scripts.

```
use engine::physics::{Vec2, RigidBody}
use engine::audio::play_sound
use game::items::{Weapon, Armor}
use game::ai::pathfind
```

Glob imports are supported:

```
use engine::math::*
```

Only Rust libraries explicitly exposed to the Homun scripting layer can be imported. You cannot
import arbitrary Rust crates — the engine controls which APIs are available, keeping the scripting
surface safe and sandboxed.

---

## Variable Assignment

Homun uses `:=` for all variable bindings, inspired by Go. There is no `var`, `let`, or `const`
keyword. There is also no `mut` keyword — mutability rules are handled at the Rust transpilation
layer, keeping the scripting surface clean.

```
x      := 10
name   := "hero"
speed  := float(3.14)
active := true
```

Types are inferred automatically from the right-hand side, similar to `auto` in C++. You may also
be explicit by wrapping the value in a type constructor:

```
hp    := int(100)
ratio := float(0.5)
label := str("player_one")
```

Rebinding an existing name simply updates the binding. There is no distinction between declaration
and reassignment in the syntax — the compiler resolves this contextually.

---

## Naming Conventions

Homun enforces and recommends strict naming conventions to keep code readable across a team of
engineers and non-engineers alike.

### Variables and Lambdas — enforced `snake_case`

All variable names and lambda names **must** be `lower_case` or `snake_case`. Uppercase letters
are **not permitted** in variable names. The compiler will reject names like `mySpeed`, `PlayerHP`,
or `X`. This is a hard rule, not a style suggestion.

```
// VALID
player_hp          := 100
move_speed         := 3.5
on_death           := () -> _ { respawn() }
base_attack_damage := 25

// INVALID — compiler error
playerHp  := 100    // camelCase not allowed
MoveSpeed := 3.5    // uppercase start not allowed
X         := 10     // single uppercase letter not allowed
```

### Structs and Enums — recommended `PascalCase`

Type definitions (structs and enums) are recommended to use `PascalCase`. This is not enforced by
the compiler but is the strong convention for all Homun code. It makes it immediately obvious when
a name refers to a type versus a value, without requiring a type-inference pass.

```
// Recommended: PascalCase for types
PlayerState := struct { hp: int, alive: bool }
Direction   := enum { North, South, East, West }
WeaponKind  := enum { Sword(int), Bow(int), Staff }

// Allowed but strongly discouraged
player_state := struct { hp: int }
```

This asymmetry is intentional: types exist at compile time, values exist at runtime. Keeping their
naming visually distinct helps tools, LLMs, and human readers alike.

---

## Operators and Equality

Homun has **no bare `=` operator**. This eliminates an entire class of bugs common in C-family
languages where `=` and `==` are accidentally swapped.

| Operator | Meaning |
|---|---|
| `:=` | Bind a name to a value (declaration or rebinding) |
| `==` | Equality comparison, returns `bool` |
| `!=` | Inequality comparison, returns `bool` |
| `<`, `>`, `<=`, `>=` | Numeric comparison |
| `and`, `or`, `not` | Boolean logic (Python-style keywords) |
| `in` | Membership test for lists, sets, dict keys — negate with `not x in s` |
| `+`, `-`, `*`, `/`, `%` | Arithmetic |
| `\|` | Pipe — passes left-hand value as first argument to right-hand call |

Using a bare `=` is a syntax error. If you see `:=` it is always a binding. If you see `==` it is
always a comparison. No ambiguity exists anywhere in the language.

```
x := 10
y := x == 10      // y is true
z := x != 5       // z is true

if (x == 10) do { print("ten") }
```

### Membership Tests

```
s := @("fire", "ice", "poison")

if x in s do { apply(x) }
if not x in s do { skip() }    // not negates the in test
```

Works on lists, sets, and dict keys. There is no `not in` operator — use `not x in s` instead.

---

## Primitive Types

| Type | Example | Notes |
|---|---|---|
| `int` | `42`, `int(42)` | 64-bit signed integer |
| `float` | `3.14`, `float(3.14)` | 64-bit float |
| `bool` | `true`, `false` | |
| `str` | `"hello"` | UTF-8, supports `${}` interpolation |
| `none` | `none` | Missing value — equivalent to Rust's `Option::None`. Use `match` to handle safely. |

`none` is a value that represents absence. It is never used as a return type annotation. To express
that a lambda returns nothing, use `-> _` — this is the sole void return form, mapping to Rust's
unit type `()`. Use `match` to safely handle expressions that may produce `none`.

---

## String Interpolation

Strings support inline variable interpolation using `${}` syntax:

```
name  := "Aria"
level := 5

greeting := "Hello, ${name}! You are level ${level}."
log      := "Dealt ${base * multiplier * 2} damage after crit."
```

Any expression is valid inside `${}`.

---

## Lambdas (All Callables)

There are no named functions in Homun. Every callable value is a lambda. Lambdas are first-class
values that can be assigned to names, passed as arguments, or returned from other lambdas.

Braces `{}` are **always required** around the lambda body. There is no expression-body shorthand.
This keeps every lambda visually uniform regardless of its complexity — the parser always knows
exactly where the body starts and ends.

### Basic Lambda

```
greet := (name) -> { "Hello ${name}" }
```

The last expression in the body is implicitly returned. No `return` keyword is needed.

### Lambda with No Arguments

```
tick := () -> { update_physics() }
```

### Return Type Hints

Annotate the return type with `->` before the opening brace. For void returns, use `-> _`:

```
add       := (a, b) -> int { a + b }
greet     := (name) -> str { "Hello ${name}" }
log_event := (msg)  -> _   { print(msg) }
tick      := ()     -> _   { update() }
```

`-> _` is the only way to express a void return. The `_` is the discard marker — consistent with
its use in destructuring — and maps directly to Rust's unit type `()`.

When no return type annotation is given, the compiler infers it from the body:

```
double := (x) -> { x * 2 }   // return type inferred as the type of x * 2
```

The three forms the parser sees after `->` are:

| Form | Meaning |
|---|---|
| `-> _  {` | void return, body follows |
| `-> TypeName {` | explicit return type, body follows |
| `-> {` | inferred return type, body follows |

All three are disambiguated by a single token of lookahead after `->`.

### Recursive Lambdas

Homun uses a **two-stage compiler**. In stage one, the compiler scans all lambda bodies for
self-references before full parsing. Any lambda whose body contains its own name is automatically
treated as recursive — no `rec` keyword or special syntax is needed by the author.

```
// this just works — compiler detects self-reference automatically
fib := (n) -> { if (n <= 1) do { n } else { fib(n-1) + fib(n-2) } }
```

Mutual recursion also works at the top level, because stage one registers all top-level names
before any body is resolved:

```
is_even := (n) -> { if (n == 0) do { true }  else { is_odd(n-1) } }
is_odd  := (n) -> { if (n == 0) do { false } else { is_even(n-1) } }
```

The compiler emits a Rust `fn` for recursive lambdas and a Rust closure for non-recursive ones.
This distinction is completely invisible to the Homun author.

### Lambdas as Values

```
transform := (x) -> { x * 2 }
doubled   := @[1, 2, 3, 4] | map(transform)
```

---

## Pipe Operator `|`

The `|` operator pipes the left-hand value as the first argument into the right-hand call.
It is an explicit operator — no whitespace sensitivity, no position rules. It can appear
on the same line or be used to build multi-line chains. The two are completely equivalent.

```
// same-line pipe
result := @[1, 2, 3] | map((x) -> { x * 2 })

// multi-line pipe chain — identical semantics
result := @[1, 2, 3, 4, 5]
  | filter((x) -> { x > 2 })
  | map((x) -> { x * 10 })
  | reduce((a, b) -> { a + b })
```

Each step desugars to a regular function call with the accumulated value inserted as the first argument:

```
// these are identical
a | map(f) | filter(g)
filter(map(a, f), g)
```

### Field Access — `.` (unchanged)

`.` is always field access or a lambda-field call. There is no pipe meaning attached to `.` at all.
Field access and pipe are now completely separate operators with no overlap and no positional rules.

```
p.hp          // field read
e.on_tick()   // lambda-field call
e.on_tick(x)  // lambda-field call with argument
```

To pipe into a global function, use `|`:

```
p.hp | clamp(0, 100)   // pipe hp value into clamp — NOT a field access
```

### Summary

| Operator | Meaning |
|---|---|
| `x.field` | field read |
| `x.field(args)` | call lambda stored in field |
| `x \| fn(args)` | pipe — desugars to `fn(x, args)` |

---

## Collections

All collection literals are prefixed with `@`. The bracket shape that follows tells you the kind.
A bare `(...)` without `@` is always a grouping expression, never a collection.

| Syntax | Kind | Ordered | Duplicates | Rust type |
|---|---|---|---|---|
| `@[...]` | List | yes | yes | `Vec<T>` |
| `@{...}` | Dict | no | keys unique | `HashMap<K,V>` |
| `@(...)` | Set | no | no | `HashSet<T>` |

### Lists (Dynamic Arrays)

```
empty := @[]
items := @[1, 2, 3]
names := @["Alice", "Bob", "Charlie"]
```

List comprehension:

```
squares := @[x * x for x in range(1, 11)]
evens   := @[x for x in range(2, 21, 2)]
```

### Dicts (Hash Maps)

```
empty_map := @{}
scores    := @{"alice": 100, "bob": 80}
```

Dict comprehension:

```
inverted := @{v: k for k, v in scores}
squared  := @{x: x*x for x in range(1, 6)}
```

Dict access and update:

```
s := scores["alice"]    // returns the value, or none if key missing
scores["alice"] := 90   // update or insert
```

If a key does not exist, dict access returns `none`. Guard with `match` when the key may be absent:

```
match scores["unknown"] {
  none => print("not found")
  val  => print("score: ${val}")
}
```

### Sets

Sets are unordered and contain no duplicate values.

```
visited   := @(1, 3, 5, 7)
flags     := @("fire", "ice", "poison")
empty_set := @()
```

Set comprehension:

```
unique_floors := @(x for x in floor_list)
```

---

## Slicing and Indexing

Homun is **0-based**, matching Rust and the engine stack. Slicing uses **Python semantics**:
`[start:end]` where `start` is inclusive and `end` is exclusive.

### Single Index

```
first := items[0]
third := items[2]
last  := items[-1]     // negative indexing from end
```

### Slicing `[start:end]` and `[start:end:step]`

```
x := @[10, 20, 30, 40, 50]

x[0:3]       // [10, 20, 30]      — items at index 0, 1, 2
x[1:4]       // [20, 30, 40]      — items at index 1, 2, 3
x[::2]       // [10, 30, 50]      — every other element
x[::-1]      // [50, 40, 30, 20, 10]  — reversed
x[1:]        // [20, 30, 40, 50]  — from index 1 to end
x[:3]        // [10, 20, 30]      — from start to index 3 (exclusive)
```

Slicing follows Python exactly: `start` is inclusive, `end` is exclusive, omitted bounds default
to the start/end of the sequence, and negative indices count from the end.

### Numeric Ranges with `range`

`range(end)` generates integers from `0` to `end` (exclusive). `range(start, end)` generates from
`start` (inclusive) to `end` (exclusive). An optional third argument sets the step size.

```
range(5)           // 0, 1, 2, 3, 4
range(3, 7)        // 3, 4, 5, 6
range(1, 10, 2)    // 1, 3, 5, 7, 9
range(10, 0, -1)   // 10, 9, 8, 7, 6, 5, 4, 3, 2, 1
range(0, 100, 25)  // 0, 25, 50, 75
```

`range` is usable in `for` loops, comprehensions, and anywhere a sequence is expected:

```
for i in range(10) do { print(i) }
for i in range(5, 10) do { print(i) }
for i in range(10, 0, -1) do { print("countdown: ${i}") }

mid    := @[x for x in range(3, 7)]
evens  := @[x for x in range(2, 20, 2)]
```

---

## Control Flow

### The `do` Rule

Any block preceded by a condition expression uses `do` before the opening `{`. This applies
uniformly to `if`, `elif`, `for`, and `while`. A bare `else` has no condition and therefore
no `do`.

### `if` / `elif` / `else`

```
if (hp <= 0) do {
  die()
} elif (hp < 20) do {
  play_low_health_sound()
} else {
  recover()
}
```

One-liner form:

```
if (x > 10) do { print("big") } else { print("small") }
```

Boolean operators use Python-style keywords:

```
if (alive and not frozen) do { move() }
if (on_fire or in_water) do { apply_effect() }
if not x in visited do { explore(x) }
```

### `match`

`match` is an expression — its result can be directly assigned. Use `_` as the wildcard/default
arm. The compiler warns if a `match` is non-exhaustive and no `_` arm exists.

```
result := match dir {
  Direction.North => move(0, 1)
  Direction.South => move(0, -1)
  Direction.East  => move(1, 0)
  Direction.West  => move(-1, 0)
}
```

Matching enum variants that carry data:

```
dmg := match element {
  Element.Fire(power) => power * 2
  Element.Ice(power)  => power * 1.5
  _                   => 0
}
```

Matching `none` for missing values:

```
match find_target(pos) {
  none   => idle()
  target => attack(target)
}
```

---

## Loops

### `for` over a range

```
for i in range(10) do {
  print("Step ${i}")
}
```

### `for` over a range with step

```
for i in range(1, 10, 2) do {
  print("Odd step ${i}")
}
```

### `for` over a list

```
for item in inventory do {
  use(item)
}
```

### `while`

```
while (enemies_remaining > 0) do {
  attack_nearest()
}
```

### `break` and `continue`

`break` exits the nearest enclosing loop. `continue` skips to the next iteration. Both work
transparently inside `match` blocks — `match` is not a loop and does not intercept them.

```
for entity in entities do {
  if (entity.hp == 0) do { continue }
  if (entity.is_boss)  do { break }
  attack(entity)
}
```

Using `break` and `continue` inside a `match` inside a loop:

```
for item in inventory do {
  match item.kind {
    ItemKind.Key  => break     // exits the for loop
    ItemKind.Junk => continue  // skips to next item
    _             => use(item)
  }
}
```

---

## Destructuring and Swap

Multiple names can be bound simultaneously on the left side of `:=`. The right-hand side is fully
evaluated before any binding occurs, making swaps always safe. Use `_` to discard a value.

### Tuple Destructuring

```
a, b    := b, a              // swap a and b
_, b    := get_pair()        // discard first, keep second
x, y    := y, x + y          // Fibonacci step
_, b, _ := get_triple()      // keep only middle value
```

### Struct Destructuring

Named and anonymous structs can be destructured the same way. Use `_` to skip fields you don't
need.

```
Player := struct { name: str, hp: int, speed: float }
p := Player { name: "Aria", hp: 100, speed: 3.5 }

// destructure by field name
{ name, hp, _ } := p             // skip speed
{ _, hp, _ }    := p             // only keep hp
{ name, _, _ }  := p             // only keep name

// anonymous structs too
pos := { x: 1.0, y: 2.0, z: 3.0 }
{ x, _, z } := pos               // skip y
```

`_` in a destructuring pattern discards that position entirely — no binding is created and the
value is dropped. This is consistent with how `_` is used everywhere in Homun as the discard marker.

---

## `none` — Missing Values

`none` represents absence — a missing or optional value with no error reason. Use `match` to
safely handle expressions that may produce `none`:

```
target := find_nearest_enemy(pos)   // returns a value or none

match target {
  none => idle()
  t    => attack(t)
}
```

Errors and failures are handled in Rust space, not script space. Homun lambdas that encounter
error conditions should communicate through `none` or by producing a sentinel value appropriate
to the game logic. Runtime error handling lives in the engine layer.

---

## Structs

Homun has no classes. Data is organized with structs. Behavior is modeled by assigning lambdas
that accept structs as parameters.

### Named Structs

```
Player := struct {
  name:  str
  hp:    int
  speed: float
}

create_player := (n, h, s) -> Player {
  Player { name: n, hp: h, speed: s }
}

p := create_player("Aria", 100, 3.5)
print(p.name)
```

### Anonymous Structs

A struct literal without a named type is valid. The compiler generates a synthetic Rust struct
behind the scenes. Field access by name works normally. Two anonymous structs with identical field
names and types are treated as the same type.

```
pos := { x: 1.0, y: 2.0 }
print(pos.x)
```

### Field Mutation

Fields are updated using `:=` with dot access:

```
p.hp    := p.hp - 10
p.speed := 5.0
```

This desugars to a Rust `let mut` rebinding of the struct. Structs are value types — mutations are
local unless the struct is explicitly returned or passed back out.

### Data Structs vs Behavior Structs

The compiler automatically classifies every struct into one of two kinds based on its fields:

**Data structs** — all fields are primitives, other data structs, lists, dicts, or sets. No lambda
fields. These are automatically RON-serializable and get `#[derive(Serialize, Deserialize, Clone)]`
in the transpiled Rust.

```
// data struct — RON compatible, auto-derives Serialize + Deserialize
Vec2   := struct { x: float, y: float }
Player := struct { name: str, hp: int, pos: Vec2 }
```

**Behavior structs** — at least one field is a lambda type. Not RON-serializable. Get only
`#[derive(Clone)]`.

```
// behavior struct — NOT RON compatible
EnemyAI := struct {
  state:   str
  on_tick: () -> _    // lambda field disqualifies RON
}
```

The author never declares which kind a struct is. The compiler infers it entirely from field types.

---

## Enums and Match

Enums define a closed set of named variants, optionally carrying data.

```
Direction := enum { North, South, East, West }

Element := enum {
  Fire(int)
  Ice(int)
  Neutral
}

WeaponKind := enum {
  Sword(int)
  Bow(int)
  Staff
}
```

Pattern matching with wildcard:

```
result := match dir {
  Direction.North => move(0, 1)
  Direction.South => move(0, -1)
  Direction.East  => move(1, 0)
  Direction.West  => move(-1, 0)
}

dmg := match element {
  Element.Fire(p) => p * 2
  Element.Ice(p)  => p * 1.5
  _               => 0        // wildcard arm
}
```

`match` is exhaustive — the compiler warns if not all variants are covered and no `_` arm exists.

---

## RON Integration

Homun has native support for **RON (Rusty Object Notation)**, the structured data format used
throughout the Rust game engine ecosystem. Because Homun struct literals and RON share the same
conceptual model, data structs in Homun are simultaneously code and serialized data format.

### Loading RON Files

```
map := load_ron("levels/world1.ron") as Map
print("width: ${map.width}")
```

The `as Type` annotation is required. The compiler validates the RON file against the named struct
at **compile time** — missing fields, wrong types, and unknown keys are compile errors, not runtime
crashes. Level designers editing RON files get full type checking for free.

The `as` keyword is reserved exclusively for this construct. It is not a general cast or
type-coercion operator anywhere else in the language.

### Saving RON Files

Any data struct can be written to RON with no extra configuration:

```
config := ServerConfig { host: "localhost", port: 8080 }
save_ron(config, "config.ron")
```

### Homun Struct Literals and RON Share the Same Grammar

A pure-data Homun struct literal is valid RON and can round-trip through it without loss. The only
syntactic difference is that Homun list literals use `@[...]` while RON uses `[...]` — the compiler
strips the `@` prefix when emitting RON automatically.

```
// this Homun value...
template := Enemy {
  name: "Goblin",
  hp:   30,
  loot: @["gold_coin", "rusty_sword"],
}

// ...round-trips to/from this RON exactly:
// Enemy(
//   name: "Goblin",
//   hp: 30,
//   loot: ["gold_coin", "rusty_sword"],
// )
```

### RON Collection Mapping

| Homun | RON |
|---|---|
| `@[...]` list | `[...]` array |
| `@{...}` dict | `{...}` map |
| `@(...)` set | `[...]` array (deduplication applied on load) |
| Struct literal | `TypeName(field: value, ...)` |

### Restrictions

Only **data structs** are RON-compatible. Calling `save_ron` on a behavior struct (one with lambda
fields) is a **compile error**. The entire game data pipeline — levels, configs, templates, save
files — can be built on data structs and RON with zero boilerplate.

---

## Built-in Utilities

These are provided by the engine runtime environment:

| Name | Description |
|---|---|
| `range(end)` | Integers from `0` to `end` (exclusive) |
| `range(start, end)` | Integers from `start` (inclusive) to `end` (exclusive) |
| `range(start, end, step)` | Integers from `start` to `end` with given step. Negative step counts down. |
| `print(x)` | Output to engine console |
| `len(col)` | Length of a list, dict, or set |
| `keys(d)` | Keys of a dict as a list |
| `values(d)` | Values of a dict as a list |
| `zip(a, b)` | Pair two lists element-wise |
| `map(col, f)` | Apply f to each element — also via pipe: `col \| map(f)` |
| `filter(col, f)` | Keep elements where f returns true |
| `reduce(col, f)` | Fold a list using a binary lambda |
| `floor(x)` | Floor of a float |
| `ceil(x)` | Ceiling of a float |
| `clamp(x, lo, hi)` | Clamp x between lo and hi inclusive |
| `abs(x)` | Absolute value |
| `load_ron(path) as T` | Load and validate a RON file against struct T (compile-time checked) |
| `save_ron(val, path)` | Serialize a data struct to a RON file |

---

## Rust Transpilation Notes

Homun compiles to idiomatic Rust. The naming conventions of Homun (`snake_case` for values,
`PascalCase` for types) match Rust's own conventions exactly, so no name mangling is ever needed.

| Homun | Rust |
|---|---|
| `use engine::x` | `use engine::x` (direct mapping) |
| `:=` binding | `let` (or `let mut` when compiler detects reassignment) |
| `==` equality | `==` (no bare `=` exists in Homun) |
| Non-recursive lambda | Rust closure `\|...\| { ... }` |
| Recursive lambda | Rust named `fn` (auto-detected, two-stage compile) |
| `@[...]` list | `Vec<T>` |
| `@{...}` dict | `HashMap<K, V>` |
| `@(...)` set | `HashSet<T>` |
| Data struct | `struct` with `#[derive(Serialize, Deserialize, Clone)]` |
| Behavior struct | `struct` with `#[derive(Clone)]` |
| Enum | `enum` |
| `match` with `_` | `match` with `_` wildcard arm |
| `x.field` | field access on struct |
| `x.field(args)` | call lambda stored in field |
| `x \| fn(args)` | function call with x as first argument |
| String `${}` | `format!()` macro |
| `and`, `or`, `not` | `&&`, `\|\|`, `!` |
| `in` | `.contains()` |
| `not x in s` | `!s.contains(x)` |
| `-> _` return | `()` unit type |
| `none` value | `Option::None` |
| `p.field := v` | `let mut p = p; p.field = v;` |
| `a, b := b, a` | `let (a, b) = (b, a);` |
| `_, b := expr` | `let (_, b) = expr;` |
| `load_ron(p) as T` | `ron::from_str::<T>(...)` with compile-time schema validation |
| `range(e)` | `0..e` iterator |
| `range(s, e)` | `s..e` iterator |
| `range(s, e, k)` | `(s..e).step_by(k)` iterator |
| `[i:j]` slice | `&v[i..j]` with bounds check |
| Variable `snake_case` | Rust `snake_case` — no mangling |
| Type `PascalCase` | Rust `PascalCase` — no mangling |

---

## Quick Reference Card

```
// Variables (snake_case enforced)
x            := 42
player_name  := "Aria"
player_hp    := int(100)

// Lambdas — braces always required
double := (x)    -> { x * 2 }
greet  := (name) -> str { "Hi ${name}" }
tick   := ()     -> _   { update() }

// Recursion — no special syntax
fib := (n) -> { if (n <= 1) do { n } else { fib(n-1) + fib(n-2) } }

// Operators
x == 42           // equality
x != 0            // inequality
"fire" in flags   // membership
not "x" in flags  // negate with not

// Pipe — explicit | operator
@[1, 2, 3, 4, 5]
  | filter((x) -> { x > 2 })
  | map((x) -> { x * 10 })
  | reduce((a, b) -> { a + b })

// Same-line pipe also valid
@[1, 2, 3] | map((x) -> { x * 2 })

// Field access and lambda-field calls — dot only
p.hp              // field read
e.on_tick()       // call lambda stored in field on_tick
e.on_tick(delta)  // call lambda stored in field on_tick with argument

// If / elif / else  (do rule: condition blocks always use do)
if (hp <= 0) do {
  die()
} elif (hp < 20) do {
  warn()
} else {
  recover()
}

// Loops with break / continue
for item in inventory do {
  if not item.usable  do { continue }
  if item.is_key      do { break }
  use(item)
}

while (alive and enemies > 0) do {
  attack_nearest()
}

// Imports — use Rust libs exposed to scripting layer
use engine::physics::{Vec2, RigidBody}
use game::ai::pathfind

// Ranges — range(end) or range(start, end) or range(start, end, step)
range(5)            // 0, 1, 2, 3, 4
range(3, 7)         // 3, 4, 5, 6
range(1, 10, 2)     // 1, 3, 5, 7, 9
range(10, 0, -1)    // 10, 9, 8 ... 1

for i in range(10) do { print(i) }
for i in range(10, 0, -1) do { print("T-minus ${i}") }

// Collections (all use @ prefix)
items  := @[1, 2, 3]
scores := @{"alice": 100, "bob": 80}
flags  := @("fire", "ice", "poison")

// Dict access
val := scores["alice"]    // value or none
scores["bob"] := 99       // update or insert

// Slicing (0-based, end-exclusive — Python semantics)
items[0:3]        // first three elements
items[::-1]       // reversed

// Destructuring / swap
a, b      := b, a           // swap
_, b      := get_pair()     // discard first, keep second
_, b, _   := get_triple()   // keep only middle value
x, y      := y, x + y       // Fibonacci step

// Struct (PascalCase recommended)
Vec2 := struct { x: float, y: float }
p    := Vec2 { x: 1.0, y: 2.0 }
p.x  := 5.0               // field mutation

// Enum + match with wildcard
Dir := enum { Up, Down, Left, Right }
match dir {
  Dir.Up   => move_up()
  Dir.Down => move_down()
  _        => idle()       // wildcard
}

// none — missing value
match find_enemy(pos) {
  none   => idle()
  target => attack(target)
}

// RON integration
level := load_ron("level1.ron") as Level
save_ron(player_state, "save.ron")
```

---

## License

Homun is part of the game engine runtime. See `LICENSE` for terms.