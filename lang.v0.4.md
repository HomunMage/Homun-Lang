# Language Specification (Unnamed)

> Compiles to Rust. No `def`, no classes, no interfaces, no methods, no `impl`. All functions are `() -> Type {}`. Structs are pure data. Braces `{}`, `:=` assignment, `==` equality, 1-based indexing, Python keywords. Pipe `|>` for composition — no method chaining. `.` is for field access only.

---

## 1. Assignment

```
x := 10
name := "hello"
nums := [1, 2, 3]
data := {"a": 1, "b": 2}
maybe := none

mut count := 0
count := count + 1             # := always, even reassignment

data["x"] := 3                 # new key
data["x"] := 5                 # update key
```

- `:=` for everything: binding, reassignment, dict updates
- `==` for equality comparison (no `=` for equality — avoids confusion)
- `mut` for mutable bindings
- Type constructors optional: `int(10)`, `str("hello")`, `List<int>([...])`

---

## 2. Types

All types are deduced from the right-hand side (like C++ `auto`). No type annotations. Use type constructors to be explicit when the compiler can't deduce.

```
x := 10                    # inferred as int
name := "hello"            # inferred as str
nums := [1, 2, 3]          # inferred as List<int>
data := {"a": 1, "b": 2}  # inferred as Dict<str, int>

# explicit type constructors when needed
x2 := int(10)
name2 := str("hello")
nums2 := List<int>([1, 2, 3])
data2 := Dict<str, int>({"a": 1, "b": 2})
maybe := int?(none)        # nullable / Option
```

### Primitive types
`int`, `float`, `str`, `bool`, `none`

Strings are iterable over characters: `for ch in "hello" { ... }` iterates `"h"`, `"e"`, `"l"`, `"l"`, `"o"`. Strings support indexing (`s[1]`) and slicing (`s[1:3]`), both 1-based.

### Compound types
- `List<T>` — dynamic array
- `Dict<K, V>` — hash map
- `Set<T>` — hash set (literal: `{1, 2, 3}`)
- `Tuple<A, B, ...>` — fixed-size tuple
- `T?` — shorthand for `Option<T>` (nullable)
- `Result<T, E>` — for error handling

### Algo data structures
- `Deque<T>` — double-ended queue
- `Heap<T>` — min-heap (use `Heap<T, "max">` for max-heap)
- `Counter<T>` — frequency counter (like Python `Counter`)
- `DefaultDict<K, V>` — dict with default values
- `SortedList<T>` — sorted container

### Comprehensions (type deduced from syntax)
```
# [expr for ...] → List
squares := [x * x for x in range(5)]

# {k: v for ...} → Dict
square_map := {x: x * x for x in range(5)}

# {expr for ...} → Set
unique := {x for x in nums}

# with filter
evens := [x for x in nums if (x % 2 == 0)]
```

### Special values
- `inf` — positive infinity
- `-inf` — negative infinity
- `none` — null/absent value

---

## 3. Indexing (1-based)

**All indexing starts at 1.** Matches how LeetCode/math problems describe positions.

```
arr := [10, 20, 30, 40, 50]
arr[1]          # 10 (first element)
arr[5]          # 50 (last element)
arr[-1]         # 50 (last element, negative indexing works)
arr[-2]         # 40
```

### Slicing (inclusive both ends)
With 1-based indexing, inclusive slicing is more natural and readable.
```
arr[1:2]        # [10, 20]       elements 1 and 2
arr[1:3]        # [10, 20, 30]   elements 1, 2, and 3
arr[3:]         # [30, 40, 50]   from 3 to end
arr[:3]         # [10, 20, 30]   from start to 3
arr[:-1]        # [10, 20, 30, 40]  up to last
arr[::2]        # [10, 30, 50]   every 2nd element
arr[::-1]       # [50, 40, 30, 20, 10]  reversed
arr[5:3:-1]     # [50, 40, 30]   from 5 down to 3
arr[3:1:-1]     # [30, 20, 10]   from 3 down to 1
```

### `range` (same inclusive logic as slicing)
`range(start, end)` is **inclusive on both ends**. `range(n)` is shorthand for `range(1, n)` (1-based).
```
for i in range(1, 10) { ... }      # 1, 2, ..., 10 (inclusive both ends)
for i in range(5) { ... }          # 1, 2, 3, 4, 5 (shorthand for range(1, 5))
for i in range(1, 10, 2) { ... }   # 1, 3, 5, 7, 9
for i in range(10, 1, -1) { ... }  # 10, 9, ..., 1
```

### Compilation to Rust
Compiler translates `arr[i]` to `arr[(i-1) as usize]` automatically. No `&`, `*` pointers — compiler handles all borrowing/references in Rust output.

---

## 4. Functions (no `def`, no `return`)

**No `def`, no `func`, no `fn`, no `return`.** Last expression is the return value.

### Syntax forms
```
# full form — explicit return type
(params) -> ReturnType { body }

# sugar — return type inferred from last expression
(params) { body }
```

### Parsing rules
- `(...)` followed by `->` → function with explicit return type
- `(...)` followed by `{` where `...` is a valid param list → function with inferred return type
- `(...)` followed by anything else → grouped expression or tuple

How the parser distinguishes `(params) { body }` from `(expr) { dict }`:
- Param lists contain identifiers, commas, and `:` type annotations: `(x)`, `(x, y)`, `(x: int)`
- Expressions contain operators, calls, literals: `(x + y)`, `(f(x))`, `(1)`
- Two expressions side-by-side with no operator is never valid, so no real ambiguity

### Examples
```
# explicit return type
add := (a: int, b: int) -> int { a + b }

# sugar — return type inferred
add := (a: int, b: int) { a + b }

# multi-line — last expression returned
max_of := (a: int, b: int) -> int {
    if (a > b) { a } else { b }
}

# void
log := (msg: str) -> none { print(msg) }
log := (msg: str) { print(msg) }              # sugar — inferred as none

# no params
tick := () -> int { counter + 1 }
noop := () -> none { print("done") }
noop := () { print("done") }                  # sugar

# inferred param types
inc := (x) -> int { x + 1 }
inc := (x) { x + 1 }                          # sugar
```

### Default parameters
```
connect := (host: str, port: int = 8080) -> Connection {
    ...
}
```

### Inline in pipes (sugar shines here)
```
# sugar — concise for inline use
[1, 2, 3]
    |> map((x) { x * 2 })
    |> filter((x) { x > 2 })

# full form — when you want explicit types
[1, 2, 3]
    |> map((x) -> int { x * 2 })
    |> filter((x) -> bool { x > 2 })
```

---

## 5. Tuple Unpacking

```
a, b := 1, 2
a, b := b, a            # swap

x, y, z := point
first, ...rest := arr     # first = arr[1], rest = arr[2:]
...init, last := arr      # last = arr[-1]

# in for loops
for i, v in enumerate(arr) {
    print(i, v)         # i starts at 1
}

for k, v in items(dict) {
    print(k, v)
}
```

---

## 6. `in` Keyword

```
if (x in arr) { ... }
if (k in dict) { ... }
if (ch in "aeiou") { ... }
if (not x in seen) { ... }
if (x not in seen) { ... }       # equivalent sugar

for x in arr { ... }
```

---

## 7. Pipe Operator `|>` (the only composition mechanism)

**No method chaining.** `.` is for field access only (`node.left`, `point.x`). All operations are standalone functions composed via pipe.

`x |> f(a, b)` desugars to `f(x, a, b)` — pipe passes the left-hand value as the **first argument**.

```
# compose functions
result := [1, 2, 3, 4, 5]
    |> filter((x) { x > 2 })
    |> map((x) { x * 10 })
    |> reduce(0, (acc, x) { acc + x })

# nested becomes flat
# instead of: c(b(a(x)))
result := x |> a |> b |> c

# sorting (just a function, not special)
sorted := arr |> sort
sorted := arr |> sort(key=(x) { x[2] })
sorted := arr |> sort(reverse=true)

# real-world pipeline
names := users
    |> map((u) { u.name })
    |> filter((n) { len(n) > 3 })
    |> sort
    |> to_list
```

### Standard library functions (all pipe-friendly, collection as first arg)
| Function | Works on | Description |
|---|---|---|
| `map(coll, fn)` | List, Set | transform each element |
| `filter(coll, fn)` | List, Set | keep matching elements |
| `reduce(coll, init, fn)` | List | fold into single value |
| `for_each(coll, fn)` | List, Set | side-effect iteration |
| `find(coll, fn)` | List | first match or none |
| `any(coll, fn)` | List | true if any match |
| `all(coll, fn)` | List | true if all match |
| `sort(coll)` | List | return new sorted list |
| `reversed(coll)` | List | return reversed copy |
| `enumerate(coll)` | List | index-value pairs (1-based) |
| `zip(coll, other)` | List | pair with another list |
| `flat_map(coll, fn)` | List | map + flatten |
| `take(coll, n)` | List | first n items |
| `skip(coll, n)` | List | skip first n items |
| `chunks(coll, n)` | List | split into groups of n |
| `keys(d)` | Dict | list of keys |
| `values(d)` | Dict | list of values |
| `items(d)` | Dict | list of (key, value) pairs |
| `contains(coll, k)` | Dict, Set | membership check |
| `len(coll)` | all | size |
| `push(list, x)` | List | return list with x appended |
| `pop(list)` | List | return (last, rest) |
| `push_front(deq, x)` | Deque | add to front |
| `push_back(deq, x)` | Deque | add to back |
| `pop_front(deq)` | Deque | remove and return front |
| `pop_back(deq)` | Deque | remove and return back |
| `add(set, x)` | Set | return set with x added |
| `remove(coll, x)` | Set, Dict | return coll without x |
| `to_list(coll)` | any iterable | collect into List |

---

## 8. Logical Operators (Python keywords)

```
if (x > 0 and x < 100) { ... }

if (name is none) { ... }

if (not name is none) { ... }
if (name is not none) { ... }     # equivalent sugar

if (a or b) { ... }

if (not valid) { ... }
```

- `and` — logical AND
- `or` — logical OR
- `not` — logical NOT
- `is` — identity/type check
- `is not` — negated identity (sugar for `not x is y`)

### Comparison
`==` for value equality, `!=`, `<`, `>`, `<=`, `>=`. No `=` for equality — `:=` is assignment, `==` is comparison.

### Bitwise operators (keyword-style — no symbol overloading)
All bitwise operators are keywords. No `&`, `|`, `^`, `~` — avoids conflicts with future syntax and keeps the lexer simple.
```
x band y        # bitwise AND
x bor y         # bitwise OR
x bxor y        # bitwise XOR
bnot x          # bitwise NOT (unary prefix)
x shl n         # shift left
x shr n         # shift right
```

Parsing: all are binary infix keywords (except `bnot` which is unary prefix). Same precedence rules as C bitwise ops. Compiler maps directly: `band` → `&`, `bor` → `|`, `bxor` → `^`, `bnot` → `!` (bitwise), `shl` → `<<`, `shr` → `>>`.

---

## 9. Control Flow

### if / elif / else (expression — returns a value)
No `do` keyword. Braces always required. `if` is an expression that returns a value.
```
# multi-line
if (x > 0) {
    print("positive")
} elif (x == 0) {
    print("zero")
} else {
    print("negative")
}

# expression form (braces required — no ambiguity)
val := if (x > 0) { x } else { -x }
label := if (score >= 90) { "A" } elif (score >= 80) { "B" } else { "C" }
```

### for loops
```
for item in list {
    print(item)
}

for i, item in enumerate(list) {
    print(i, item)
}

for k, v in items(dict) {
    print(k, v)
}
```

### while
```
while condition {
    ...
}
```

### break / continue
```
for x in arr {
    if (x == 0) { continue }
    if (x < 0) { break }
    print(x)
}

# break with value (loop as expression)
found := for x in arr {
    if (x > 100) { break x }
}
```

### match (pattern matching)
```
match value {
    1 => print("one"),
    2 => print("two"),
    n if n > 10 => print("big"),
    _ => print("other"),
}
```

---

## 10. Structs (no classes, no methods, no interfaces)

Structs are pure data containers. No `impl`, no methods, no `self`. All behavior lives in standalone functions that take structs as arguments. Pure functional style.

### Structs
```
struct TreeNode {
    val: int,
    left: TreeNode?,
    right: TreeNode?,
}

node := TreeNode { val: 1, left: none, right: none }
```

### Functions on structs (not methods — just functions)
```
is_leaf := (node: TreeNode) -> bool {
    node.left is none and node.right is none
}

depth := (node: TreeNode) -> int {
    mut l := 0
    mut r := 0
    if (node.left is not none) { l := depth(node.left) }
    if (node.right is not none) { r := depth(node.right) }
    1 + max(l, r)
}
```

### Enums (algebraic types)
```
enum Shape {
    Circle(radius: float),
    Rect(w: float, h: float),
    Empty,
}

area := (s: Shape) -> float {
    match s {
        Shape.Circle(r) => 3.14159 * r * r,
        Shape.Rect(w, h) => w * h,
        Shape.Empty => 0.0,
    }
}
```

---

## 11. String Interpolation

```
name := "world"
msg := "hello {name}, result is {1 + 2}"
```

All strings support interpolation by default (no `f` prefix needed).

### Escaping
- `{{` → literal `{`
- `}}` → literal `}`
- `\\` → literal `\`
- `\n`, `\t`, `\"` — standard escape sequences

```
json := "{{\"key\": \"{value}\"}}"    # {"key": "hello"}
```

Parsing: the lexer scans strings character by character. On `{`, check next char: if `{` → literal brace, otherwise enter expression parsing mode until matching `}`.

---

## 12. Error Handling

Result-based (like Rust), with cleaner syntax:

```
read_file := (path: str) -> Result<str, Error> {
    ...
}

# propagate with ?
process := () -> Result<Data, Error> {
    content := read_file("data.txt")?
    parse(content)
}

# handle explicitly
match read_file("data.txt") {
    Ok(content) => print(content),
    Err(e) => print("failed: {e}"),
}
```

---

## 13. Modules

```
# math_utils.lang
pub square := (x: int) -> int { x * x }

# main.lang
import math_utils
result := math_utils.square(5)

# or selective import
from math_utils import square
result := square(5)
```

---

## 14. Examples: LeetCode

### Two Sum (LC #1)
```
two_sum := (nums: List<int>, target: int) -> List<int> {
    seen := Dict<int, int>()
    for i, n in enumerate(nums) {
        comp := target - n
        if (comp in seen) { break [seen[comp], i] }
        seen[n] := i
    }
}
```

### Valid Parentheses (LC #20)
```
is_valid := (s: str) -> bool {
    mut stack := List<str>()
    for ch in s {
        if (ch in "([{") { stack := stack + [ch] }
        elif (ch in ")]}") {
            pairs := {")" : "(", "]" : "[", "}" : "{"}
            if (len(stack) == 0 or stack[-1] != pairs[ch]) { break false }
            stack := stack[:-1]
        }
    }
    len(stack) == 0
}
```

### Quicksort
```
quicksort := (arr: List<int>) -> List<int> {
    if (len(arr) <= 1) { arr }
    else {
        pivot := arr[1]
        rest := arr[2:]
        left := rest |> filter((x) { x <= pivot })
        right := rest |> filter((x) { x > pivot })
        quicksort(left) + [pivot] + quicksort(right)
    }
}
```

### Top K Frequent (LC #347)
```
top_k_frequent := (nums: List<int>, k: int) -> List<int> {
    Counter(nums)
        |> items
        |> sort(key=(x) { -x[2] })
        |> take(k)
        |> map((x) { x[1] })
        |> to_list
}
```

### Palindrome Check
```
is_palindrome := (s: str) -> bool { s == s[::-1] }
```

---

## 15. Compilation Target

Compiles to **Rust**. The compiler:
1. Parses `.lang` source files
2. Type-checks using inference + annotations
3. Emits Rust source code
4. Invokes `rustc` / `cargo` for final binary

### Mapping to Rust
| Lang | Rust |
|---|---|
| `x := 5` | `let x = 5;` |
| `mut x := 5` | `let mut x = 5;` |
| `arr[1]` | `arr[0]` (1-based to 0-based) |
| `List<T>` | `Vec<T>` |
| `Dict<K,V>` | `HashMap<K,V>` |
| `Set<T>` | `HashSet<T>` |
| `T?` | `Option<T>` |
| `str` | `String` |
| `and` / `or` / `not` | `&&` / `\|\|` / `!` |
| `is none` / `is not none` | `.is_none()` / `.is_some()` |
| `==` / `!=` | `==` / `!=` |
| struct (data only) | struct (compiler adds `&`, `*`, `Clone` automatically) |
| `(x) { x + 1 }` | `\|x\| x + 1` (closure) |
| `(x) -> int { x + 1 }` | `\|x\| -> i64 { x + 1 }` (typed closure) |
| `x \|> map(f) \|> filter(g)` | `.iter().map(f).filter(g)` (iterator adapters) |
| `x \|> f \|> g` | `g(f(x))` |
| `if (p) { x } else { y }` | `if p { x } else { y }` (expression) |
| `band` / `bor` / `bxor` | `&` / `\|` / `^` |
| `shl` / `shr` | `<<` / `>>` |
| last expression = return | last expression = return (same as Rust) |

---

## Summary

```
# functions — last expression is the return value
add := (a: int, b: int) -> int { a + b }     # explicit return type
add := (a: int, b: int) { a + b }            # sugar — inferred
greet := (name: str) -> str { "hello {name}" }
log := (msg: str) -> none { print(msg) }     # void
log := (msg: str) { print(msg) }             # sugar — void inferred

# if/else expression (braces always required)
val := if (x > 0) { x } else { -x }

# pipe operator
result := data |> process |> transform

# 1-based indexing, inclusive slicing
arr[1]          # first element
arr[2:]         # slice from 2nd
arr[::-1]       # reversed

# tuple unpacking
a, b := b, a

# pipe (the only composition mechanism)
[1,2,3] |> map((x) { x * 2 }) |> filter((x) { x > 2 })

# in keyword
if (x in arr) { ... }
if (not x in seen) { ... }

# logic (Python keywords)
if (a and b or not c) { ... }
if (val is none) { ... }
if (val is not none) { ... }

# bitwise (keyword operators)
x band y          # AND
x bor y           # OR
x bxor y          # XOR
x shl 2           # shift left
x shr 2           # shift right
bnot x            # bitwise NOT

# := assign, == compare, no ambiguity
x := 10
if (x == 10) { ... }

# no classes, no interfaces, no methods — structs are pure data, all behavior is functions
```
