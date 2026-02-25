# Language Specification (Unnamed)

> Compiles to Rust. No `def`, no classes, no interfaces, no methods, no `impl`. All functions are `|| -> Type {}`. Structs are pure data. Braces `{}`, `:=` assignment, `==` equality, 1-based indexing, Python keywords. Pipe use `.`.

---

## A. Basic Syntax

### Assignment

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

### Types

All types are deduced from the right-hand side (like C++ `auto`). No type annotations required. Use type constructors to be explicit when the compiler can't deduce.

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

Primitive types: `int`, `float`, `str`, `bool`, `none`

Strings are iterable over characters: `for ch in "hello" { ... }` iterates `"h"`, `"e"`, `"l"`, `"l"`, `"o"`. Strings support indexing (`s[1]`) and slicing (`s[1:3]`), both 1-based.

Compound types:
- `List<T>` — dynamic array
- `Dict<K, V>` — hash map
- `Set<T>` — hash set (literal: `$(1, 2, 3)`)
- `Tuple<A, B, ...>` — fixed-size tuple
- `T?` — shorthand for `Option<T>` (nullable)
- `Result<T, E>` — for error handling

Special values: `inf`, `-inf`, `none`

### Indexing (1-based)

**All indexing starts at 1.** Matches how math problems describe positions.

```
arr := [10, 20, 30, 40, 50]
arr[1]          # 10 (first element)
arr[5]          # 50 (last element)
arr[-1]         # 50 (last element, negative indexing works)
arr[-2]         # 40
```

Slicing — inclusive both ends:
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

Compiler translates `arr[i]` to `arr[(i-1) as usize]` automatically. No `&`, `*` pointers — compiler handles all borrowing/references in Rust output.

### Functions (no `def`, no `return`)

**No `def`, no `func`, no `fn`, no `return`.** Last expression is the return value. Functions use `|...|` delimiters to mark parameters, eliminating parsing ambiguity.

```
# full form — explicit return type
|params| -> ReturnType { body }
```

Parsing rules:
- `|...|` → always a function (unambiguous, LL(1) parsable — `|` is free since bitwise OR is `bor`)
- `|...|` followed by `->` → function with explicit return type
- `|...|` followed by `{` → function with inferred return type
- `(...)` → always a grouped expression or tuple (never a function)

```
# explicit return type
add := |a: int, b: int| -> int { a + b }

# multi-line — last expression returned
max_of := |a: int, b: int| -> int {
    if (a > b) { a } else { b }
}

# void
log := |msg: str| -> none { print(msg) }

# no params
tick := || -> int { counter + 1 }
noop := || -> none { print("done") }

# default parameters
connect := |host: str, port: int = 8080| -> Connection {
    ...
}
```

### Logical Operators (Python keywords)

```
if (x > 0 and x < 100) { ... }
if (name is none) { ... }
if (not name is none) { ... }
if (a or b) { ... }
if (not valid) { ... }
```

- `and` — logical AND
- `or` — logical OR
- `not` — logical NOT
- `is` — identity/type check
- `==` for value equality, `!=`, `<`, `>`, `<=`, `>=`

### `in` Keyword

```
if (x in arr) { ... }
if (k in dict) { ... }
if (ch in "aeiou") { ... }
if (not x in seen) { ... }

for x in arr { ... }
```

### Pipe `.` (dot pipe)

**No methods, no `impl`.** `.` is the pipe operator, borrowing familiar dot syntax. All operations are standalone functions — `.` just pipes the left-hand value as the **first argument**.

`.` serves two roles, disambiguated by `(`:
- `x.name` — **field access** (no parens)
- `x.name(args)` — **pipe**, desugars to `name(x, args)`

```
# compose functions
result := [1, 2, 3, 4, 5]
    .filter(|x| -> bool { x > 2 })
    .map(|x| -> int { x * 10 })
    .reduce(0, |acc, x| -> int { acc + x })

# nested becomes flat
# instead of: c(b(a(x)))
result := x.a().b().c()

# no-arg pipe — parens required to distinguish from field access
result := items.sort().reversed()
```

To call a function stored in a struct field: `(obj.callback)(args)` (parens force field access first).

### Control Flow

if / elif / else — expression, returns a value. Braces always required.
```
if (x > 0) {
    print("positive")
} elif (x == 0) {
    print("zero")
} else {
    print("negative")
}

# expression form
val := if (x > 0) { x } else { -x }
label := if (score >= 90) { "A" } elif (score >= 80) { "B" } else { "C" }
```

for loops:
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

while:
```
while condition {
    ...
}
```

break / continue:
```
for x in arr {
    if (x == 0) { continue }
    if (x < 0) { break }
    print(x)
}
```

break with value — use `break =>` to return a value from a loop expression:
```
for x in arr {
    if (x > 100) { break => x }
}
```

match (pattern matching):
```
match value {
    1 => print("one"),
    2 => print("two"),
    n if n > 10 => print("big"),
    _ => print("other"),
}
```

### Structs (no classes, no methods, no interfaces)

Structs are pure data containers. No `impl`, no methods, no `self`. All behavior lives in standalone functions.

```
struct TreeNode {
    val: int,
    left: TreeNode?,
    right: TreeNode?,
}

node := TreeNode { val: 1, left: none, right: none }

is_leaf := |node: TreeNode| -> bool {
    node.left is none and node.right is none
}

depth := |node: TreeNode| -> int {
    mut l := 0
    mut r := 0
    if (node.left is not none) { l := depth(node.left) }
    if (node.right is not none) { r := depth(node.right) }
    1 + max(l, r)
}
```

Enums (algebraic types):
```
enum Shape {
    Circle(radius: float),
    Rect(w: float, h: float),
    Empty,
}

area := |s: Shape| -> float {
    match s {
        Shape.Circle(r) => 3.14159 * r * r,
        Shape.Rect(w, h) => w * h,
        Shape.Empty => 0.0,
    }
}
```

### String Interpolation

```
name := "world"
msg := "hello ${name}, result is ${1 + 2}"
```

All strings support interpolation by default (no `f` prefix needed). `${...}` marks interpolated expressions — the lexer scans for `$` followed by `{`, so bare `{` and `}` are always literal.

Escaping: `\\` → literal `\`, `\n`, `\t`, `\"` — standard escape sequences. `\$` → literal `$` (to suppress interpolation).

### Error Handling

Result-based (like Rust), with cleaner syntax:

```
read_file := |path: str| -> Result<str, Error> {
    ...
}

# propagate with ?
process := || -> Result<Data, Error> {
    content := read_file("data.txt")?
    parse(content)
}

# handle explicitly
match read_file("data.txt") {
    Ok(content) => print(content),
    Err(e) => print("failed: ${e}"),
}
```

### Modules

```
# math_utils.lang
pub square := |x: int| -> int { x * x }

# main.lang — explicit named import
import square from math_utils

result := square(5)
```

---

## B. Sugar Candy Syntax

Convenience forms that desugar to basic syntax. Not required — you can always write the full form.

### Inferred return type

```
# sugar
add := |a: int, b: int| { a + b }

# desugars to
add := |a: int, b: int| -> int { a + b }
```

Works everywhere:
```
log := |msg: str| { print(msg) }             # inferred as -> none
inc := |x| { x + 1 }                         # inferred param + return type
noop := || { print("done") }                 # inferred as -> none
```

### Inline lambdas in pipes

```
# sugar
[1, 2, 3]
    .map(|x| { x * 2 })
    .filter(|x| { x > 2 })

# desugars to
[1, 2, 3]
    .map(|x| -> int { x * 2 })
    .filter(|x| -> bool { x > 2 })
```

### `x not in` / `is not`

```
# sugar
if (x not in seen) { ... }
if (name is not none) { ... }

# desugars to
if (not x in seen) { ... }
if (not name is none) { ... }
```

### Comprehensions

Comprehensions use `$[...]`, `${...}`, and `$(...)` prefixes so the parser immediately knows the form without lookahead.

```
# $[expr for ...] → List
squares := $[x * x for x in range(5)]

# ${k: v for ...} → Dict
square_map := ${x: x * x for x in range(5)}

# $(expr for ...) → Set
unique := $(x for x in nums)

# with filter
evens := $[x for x in nums if (x % 2 == 0)]
```

Desugars to `range.map(f).filter(g).to_list()` (or `.to_dict()`, `.to_set()`).

### Tuple unpacking

```
a, b := 1, 2
a, b := b, a                 # swap

x, y, z := point
first, ...rest := arr         # first = arr[1], rest = arr[2:]
...init, last := arr          # last = arr[-1]

# in for loops
for i, v in enumerate(arr) {
    print(i, v)               # i starts at 1
}

for k, v in items(dict) {
    print(k, v)
}
```

### `range(n)` shorthand

```
# sugar
for i in range(5) { ... }          # 1, 2, 3, 4, 5

# desugars to
for i in range(1, 5) { ... }       # inclusive both ends
```

Full form: `range(start, end)` is **inclusive on both ends**.
```
for i in range(1, 10) { ... }      # 1, 2, ..., 10
for i in range(1, 10, 2) { ... }   # 1, 3, 5, 7, 9
for i in range(10, 1, -1) { ... }  # 10, 9, ..., 1
```

### Break with value (loop as expression)

```
found := for x in arr {
    if (x > 100) { break => x }
}
```

---

## C. Standard Library (`std`)

Functions and types available via import. All pipe-friendly (collection as first arg).

### Import system

```
import map from std              # explicit named import
import Heap from collections     # import specific type

unpack std                       # import * — brings all std names into scope
unpack collections               # import * from collections
```

`unpack` is shorthand for importing everything from a module. Use `import x from y` when you want to be explicit.

### Core functions (auto-available, no import needed)

| Function | Description |
|---|---|
| `print(...)` | output to stdout |
| `len(coll)` | size of any collection |
| `range(start, end)` | inclusive range |
| `enumerate(coll)` | index-value pairs (1-based) |

### `std` — collection operations

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
| `zip(coll, other)` | List | pair with another list |
| `flat_map(coll, fn)` | List | map + flatten |
| `take(coll, n)` | List | first n items |
| `skip(coll, n)` | List | skip first n items |
| `chunks(coll, n)` | List | split into groups of n |
| `keys(d)` | Dict | list of keys |
| `values(d)` | Dict | list of values |
| `items(d)` | Dict | list of (key, value) pairs |
| `contains(coll, k)` | Dict, Set | membership check |
| `push(list, x)` | List | return list with x appended |
| `pop(list)` | List | return (last, rest) |
| `add(set, x)` | Set | return set with x added |
| `remove(coll, x)` | Set, Dict | return coll without x |
| `to_list(coll)` | any iterable | collect into List |
| `max(a, b)` | numeric | larger value |
| `min(a, b)` | numeric | smaller value |

### `std` — bitwise operations

Keyword-style operators. No `&`, `|`, `^`, `~` symbols.

```
unpack std    # makes band, bor, bxor, etc. available

x band y        # bitwise AND
x bor y         # bitwise OR
x bxor y        # bitwise XOR
bnot x          # bitwise NOT (unary prefix)
x shl n         # shift left
x shr n         # shift right
```

Compiler maps: `band` → `&`, `bor` → `|`, `bxor` → `^`, `bnot` → `!`, `shl` → `<<`, `shr` → `>>`.

### `collections` — algo data structures

```
import Heap from collections
import Counter from collections
```

| Type | Description |
|---|---|
| `Deque<T>` | double-ended queue |
| `Heap<T>` | min-heap (use `Heap<T, "max">` for max-heap) |
| `Counter<T>` | frequency counter (like Python `Counter`) |
| `DefaultDict<K, V>` | dict with default values |
| `SortedList<T>` | sorted container |

Deque operations: `push_front`, `push_back`, `pop_front`, `pop_back`

---

## D. Compilation Target

Compiles to **Rust**. The compiler:
1. Parses `.lang` source files
2. Type-checks using inference + annotations
3. Emits Rust source code
4. Invokes `rustc` / `cargo` for final binary

| Lang | Rust |
|---|---|
| `x := 5` | `let x = 5;` |
| `mut x := 5` | `let mut x = 5;` |
| `arr[1]` | `arr[0]` (1-based to 0-based) |
| `List<T>` | `Vec<T>` |
| `Dict<K,V>` | `HashMap<K,V>` |
| `Set<T>` / `$(1, 2, 3)` | `HashSet<T>` / `HashSet::from([1, 2, 3])` |
| `T?` | `Option<T>` |
| `str` | `String` |
| `and` / `or` / `not` | `&&` / `\|\|` / `!` |
| `is none` / `is not none` | `.is_none()` / `.is_some()` |
| struct (data only) | struct (compiler adds `&`, `*`, `Clone` automatically) |
| `|x| -> int { x + 1 }` | `\|x\| -> i64 { x + 1 }` (typed closure) |
| `x.map(f).filter(g)` | `.iter().map(f).filter(g)` (iterator adapters) |
| `x.f().g()` | `g(f(x))` |
| `band` / `bor` / `bxor` | `&` / `\|` / `^` |
| `shl` / `shr` | `<<` / `>>` |
| `$(1, 2, 3)` | `HashSet::from([1, 2, 3])` |
| `"hello ${x}"` | `format!("hello {}", x)` |
| `break => x` | labeled block return |
| last expression = return | last expression = return (same as Rust) |

---

## E. Examples

Two Sum (LC #1):
```
unpack std

two_sum := |nums: List<int>, target: int| -> List<int> {
    seen := Dict<int, int>()
    for i, n in enumerate(nums) {
        comp := target - n
        if (comp in seen) { break => [seen[comp], i] }
        seen[n] := i
    }
}
```

Valid Parentheses (LC #20):
```
unpack std

is_valid := |s: str| -> bool {
    mut stack := List<str>()
    for ch in s {
        if (ch in "([{") { stack := stack + [ch] }
        elif (ch in ")]}") {
            pairs := {")" : "(", "]" : "[", "}" : "{"}
            if (len(stack) == 0 or stack[-1] != pairs[ch]) { break => false }
            stack := stack[:-1]
        }
    }
    len(stack) == 0
}
```

Quicksort:
```
unpack std

quicksort := |arr: List<int>| -> List<int> {
    if (len(arr) <= 1) { arr }
    else {
        pivot := arr[1]
        rest := arr[2:]
        left := rest.filter(|x| { x <= pivot })
        right := rest.filter(|x| { x > pivot })
        quicksort(left) + [pivot] + quicksort(right)
    }
}
```

Top K Frequent (LC #347):
```
import Counter from collections
unpack std

top_k_frequent := |nums: List<int>, k: int| -> List<int> {
    Counter(nums)
        .items()
        .sort(key=|x| { -x[2] })
        .take(k)
        .map(|x| { x[1] })
        .to_list()
}
```

Palindrome Check:
```
is_palindrome := |s: str| -> bool { s == s[::-1] }
```

---

## F. Parser-Friendly Design Summary

This language is designed for **LL(1) parsing** — the parser never needs to backtrack or look ahead more than one token. Key design choices:

| Syntax | Prefix token | What it tells the parser |
|---|---|---|
| `|...|` | `\|` | This is a function/lambda, not a grouped expression |
| `$(...)` | `$(` | This is a set literal or set comprehension |
| `$[...]` | `$[` | This is a list comprehension, not a list literal |
| `${...}` | `${` | This is a dict comprehension, not a dict literal |
| `"...${expr}..."` | `$` inside string | Start of interpolation (bare `{` is literal) |
| `break => x` | `=>` after `break` | Break carries a value (bare `break` doesn't) |
| `{...}` | bare `{` | Always a block or dict literal (never a set) |
