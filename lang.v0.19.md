# Homun Language Reference

> A scripting language for Rust game engines — 0-based, embeddable, minimal, and expressive.
> Compiles / transpiles directly to Rust.

---

## Overview

Homun is a scripting layer for a Rust-based ECS game engine. Every valid Homun program transpiles
1-to-1 to Rust. For performance-critical code and architecture, you write Rust directly. Homun
gives game designers a lighter syntax for gameplay scripts without writing raw Rust.

Scripts can `use` Rust libraries exposed to the scripting layer (like pyo3 for Python). 0-based
indexing throughout, matching Rust. Slicing follows Python semantics: `[start:end)`.

---

## Examples

```
// Two Sum
two_sum := (nums: @[int], target: int) -> @[int] {
  seen := @{}
  for i in range(len(nums)) do {
    comp := target - nums[i]
    if (comp in seen) do { break => @[seen[comp], i] }
    seen[nums[i]] := i
  }
}

// Valid Parentheses
is_valid := (s: str) -> bool {
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
quicksort := (arr: @[int]) -> @[int] {
  if (len(arr) <= 1) do { break => arr }
  pivot := arr[0]
  rest  := arr[1:]
  left  := rest | filter((x) -> { x <= pivot })
  right := rest | filter((x) -> { x > pivot })
  quicksort(left) + @[pivot] + quicksort(right)
}

// DFS — recursive
dfs := (graph: @{int, @[int]}, node: int, visited: @[int]) -> @[int] {
  if (node in visited) do { break => visited }
  visited := visited + @[node]
  for neighbor in graph[node] do {
    visited := dfs(graph, neighbor, visited)
  }
  visited
}

// Polymorphic — type inferred at call site
first := (items) -> { items[0] }

// One-liners
is_palindrome := (s: str) -> bool { s == s[::-1] }
fib := (n) -> { if (n <= 1) do { n } else { fib(n-1) + fib(n-2) } }
```

---

## Basics

```
// Comments
// single-line comment
/* multi-line comment */

// Imports — only Rust libs exposed to the scripting layer
use engine::physics::{Vec2, RigidBody}
use engine::math::*

// Variables — := for all bindings, no let/var/const/mut
x      := 10
name   := "hero"
speed  := float(3.14)
hp     := int(100)         // explicit type via constructor

// String interpolation — any expression inside ${}
greeting := "Hello, ${name}! HP: ${hp * 2}"
```

### Naming

Variables and lambdas **must** be `snake_case` — enforced by compiler. Types are recommended
`PascalCase` — not enforced but strong convention.

```
player_hp := 100           // VALID
playerHp  := 100           // INVALID — compiler error

PlayerState := struct { hp: int, alive: bool }   // PascalCase for types
```

### Primitive Types

| Type | Example | Notes |
|---|---|---|
| `int` | `42`, `int(42)` | 64-bit signed |
| `float` | `3.14`, `float(3.14)` | 64-bit |
| `bool` | `true`, `false` | |
| `str` | `"hello"` | UTF-8, `${}` interpolation |
| `none` | `none` | Missing value. Use `match` to handle. |

`-> _` is the void return (Rust `()`). `none` is absence (Rust `Option::None`). They are different.

---

## Operators

No bare `=` exists. `:=` binds, `==` compares. No ambiguity.

| Operator | Meaning |
|---|---|
| `:=` | Bind / rebind |
| `==`, `!=`, `<`, `>`, `<=`, `>=` | Comparison |
| `and`, `or`, `not` | Boolean (Python-style) |
| `in` | Membership (lists, sets, dict keys). Negate: `not x in s` |
| `+`, `-`, `*`, `/`, `%` | Arithmetic |
| `\|` | Pipe — `x \| f(args)` desugars to `f(x, args)` |
| `.` | Field access / lambda-field call |

---

## Type Inference

Strongly typed — every value has a known type at compile time. No `<T>` in syntax. Types are
inferred from contents or first use. Unused declarations are compile errors.

```
items := @[1, 2, 3]        // int list — inferred from contents
empty := @[]                // type unknown until first use
empty := empty + @["hi"]    // now compiler knows: str list

unused := @[]               // COMPILE ERROR — never used

// Context flows from parameters and return types
process := (items: @[int]) -> _ { ... }
buffer := @[]
process(buffer)             // compiler infers buffer is @[int]
```

---

## Lambdas

Every callable is a lambda. Braces always required. Last expression is the return value.

```
double    := (x) -> { x * 2 }                       // fully inferred
add       := (a: int, b: int) -> int { a + b }      // typed
log_event := (msg) -> _ { print(msg) }               // void (-> _)
tick      := () -> _ { update() }                    // no args
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

Auto-detected — no `rec` keyword. Mutual recursion is forbidden (compile error).

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
while (enemies > 0) do { attack_nearest() }
```

`break` exits loop. `continue` skips iteration. Both work inside `match` blocks.

### `break => value` — Early Return

`break` exits a loop. `break => value` exits the **enclosing lambda** with a value.

```
clamp_hp := (hp) -> int {
  if (hp < 0)   do { break => 0 }
  if (hp > 100) do { break => 100 }
  hp
}
```

`break => _` exits a void (`-> _`) lambda early.

---

## Destructuring

Multiple bindings on left side of `:=`. Right side fully evaluated first.

```
a, b    := b, a              // swap
_, b    := get_pair()        // discard first
x, y    := y, x + y          // Fibonacci step

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

## License

Homun is part of the game engine runtime. See `LICENSE` for terms.
