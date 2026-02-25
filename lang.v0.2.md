# Language Specification (Unnamed)

> Compiles to Rust. No `def`, no classes, no interfaces, no methods, no `impl`. All functions are `lambda()`. Structs are pure data. Braces `{}`, `:=` assignment, 1-based indexing, Python keywords.

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
- `=` is **only** for equality comparison (`==` is also valid)
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
- `Set<T>` — hash set
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
evens := [x for x in nums if (x % 2 = 0)]
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

## 4. Functions (lambda only)

**No `def`, no `func`, no `return`.** All functions are lambdas bound to names with `:=`. Last expression is the return value. No `->` means void.

```
# last expression is the return value
add := lambda(a: int, b: int) -> int { a + b }

# multi-line — last expression returned
max_of := lambda(a: int, b: int) -> int {
    if (a > b) do { a } else { b }
}

# no -> means void (returns nothing)
log := lambda(msg: str) {
    print(msg)
}

# no params
tick := lambda() -> int { counter + 1 }

# inferred types
inc := lambda(x) { x + 1 }
```

### Default parameters
```
connect := lambda(host: str, port: int = 8080) -> Connection {
    ...
}
```

### Inline in chains
```
[1, 2, 3]
    .map(lambda(x) { x * 2 })
    .filter(lambda(x) { x > 2 })
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
for i, v in arr.enumerate() {
    print(i, v)         # i starts at 1
}

for k, v in dict.items() {
    print(k, v)
}
```

---

## 6. `in` Keyword

```
if (x in arr) do { ... }
if (k in dict) do { ... }
if (ch in "aeiou") do { ... }
if (not x in seen) do { ... }

for x in arr { ... }
```

---

## 7. Sorting

No in-place mutation — all sort returns a new list (pure functional).
```
sorted := arr.sort()                        # returns new sorted list
sorted := arr.sort(key=lambda(x) { x[2] }) # sort by 3rd element
sorted := arr.sort(reverse=true)            # descending
sorted := arr.sort(key=lambda(x) { -x })   # custom order
```

---

## 8. Method Chaining (TS-style)

All built-in collections support chainable methods:

```
result := [1, 2, 3, 4, 5]
    .filter(lambda(x: int) -> bool { x > 2 })
    .map(lambda(x: int) -> int { x * 10 })
    .reduce(0, lambda(acc: int, x: int) -> int { acc + x })

names := users
    .map(lambda(u) { u.name })
    .filter(lambda(n) { n.len() > 3 })
    .sorted()
    .to_list()
```

### Key collection methods
| Method | On | Description |
|---|---|---|
| `.map(fn)` | List, Set | transform each element |
| `.filter(fn)` | List, Set | keep matching elements |
| `.reduce(init, fn)` | List | fold into single value |
| `.for_each(fn)` | List, Set | side-effect iteration |
| `.find(fn)` | List | first match or none |
| `.any(fn)` | List | true if any match |
| `.all(fn)` | List | true if all match |
| `.sort()` | List | return new sorted list |
| `.reversed()` | List | return reversed copy |
| `.enumerate()` | List | index-value pairs (1-based) |
| `.zip(other)` | List | pair with another list |
| `.flat_map(fn)` | List | map + flatten |
| `.take(n)` | List | first n items |
| `.skip(n)` | List | skip first n items |
| `.chunks(n)` | List | split into groups of n |
| `.keys()` | Dict | list of keys |
| `.values()` | Dict | list of values |
| `.items()` | Dict | list of (key, value) pairs |
| `.contains(k)` | Dict, Set | membership check |
| `.len()` | all | size |
| `.push(x)` | List | append to end |
| `.pop()` | List | remove and return last |
| `.push_front(x)` | Deque | add to front |
| `.push_back(x)` | Deque | add to back |
| `.pop_front()` | Deque | remove and return front |
| `.pop_back()` | Deque | remove and return back |
| `.add(x)` | Set | insert element |
| `.remove(x)` | Set, Dict | remove element/key |
| `.to_list()` | any iterable | collect into List |

---

## 9. Pipe Operator `|>`

Passes the left-hand value as the first argument to the right-hand function. Alternative to method chaining for standalone functions.

```
# pipe into functions
result := [1, 2, 3, 4, 5]
    |> filter(lambda(x) { x > 2 })
    |> map(lambda(x) { x * 10 })
    |> reduce(0, lambda(acc, x) { acc + x })

# mix with method chaining
result := data
    |> process
    |> transform
    .to_list()

# nested becomes flat
# instead of: c(b(a(x)))
result := x |> a |> b |> c
```

---

## 10. Logical Operators (Python keywords)

```
if (x > 0 and x < 100) do { ... }

if (name is none) do { ... }

if (not name is none) do { ... }

if (a or b) do { ... }

if (not valid) do { ... }
```

- `and` — logical AND
- `or` — logical OR
- `not` — logical NOT
- `is` — identity/type check
- use `not x is y` for negation (no `is not` — keeps `not` always at the front)

### Comparison
`=` or `==` for value equality (both work), `!=`, `<`, `>`, `<=`, `>=`. No ambiguity since `:=` is always assignment.

---

## 11. Control Flow

### if / elif / else (expression — returns a value)
One unified syntax: `if (condition) do`. Braces for blocks, bare for single expressions.
```
# multi-line
if (x > 0) do {
    print("positive")
} elif (x == 0) do {
    print("zero")
} else {
    print("negative")
}

# single expression (sugar — no braces)
val := if (x > 0) do x else -x
label := if (score >= 90) do "A" else if (score >= 80) do "B" else "C"
```

### for loops
```
for item in list {
    print(item)
}

for i, item in list.enumerate() {
    print(i, item)
}

for k, v in dict.items() {
    print(k, v)
}
```

### while
```
while condition {
    ...
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

## 12. Structs (no classes, no methods, no interfaces)

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
is_leaf := lambda(node: TreeNode) -> bool {
    node.left is none and node.right is none
}

depth := lambda(node: TreeNode) -> int {
    mut l := 0
    mut r := 0
    if (not node.left is none) do { l := depth(node.left) }
    if (not node.right is none) do { r := depth(node.right) }
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

area := lambda(s: Shape) -> float {
    match s {
        Shape.Circle(r) => 3.14159 * r * r,
        Shape.Rect(w, h) => w * h,
        Shape.Empty => 0.0,
    }
}
```

---

## 13. String Interpolation

```
name := "world"
msg := "hello {name}, result is {1 + 2}"
```

All strings support interpolation by default (no `f` prefix needed).

---

## 14. Error Handling

Result-based (like Rust), with cleaner syntax:

```
read_file := lambda(path: str) -> Result<str, Error> {
    ...
}

# propagate with ?
process := lambda() -> Result<Data, Error> {
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

## 15. Modules

```
# math_utils.lang
pub square := lambda(x: int) -> int { x * x }

# main.lang
import math_utils
result := math_utils.square(5)

# or selective import
from math_utils import square
result := square(5)
```

---

## 16. Examples: LeetCode

### Two Sum (LC #1)
```
two_sum := lambda(nums: List<int>, target: int) -> List<int> {
    seen := Dict<int, int>()
    nums.enumerate()
        .find(lambda(i, n) {
            comp := target - n
            if (comp in seen) do true else { seen[n] := i; false }
        })
        |> lambda(result) {
            if (result is none) do [] else {
                i, n := result
                [seen[target - n], i]
            }
        }
}
```

### Valid Parentheses (LC #20)
```
is_valid := lambda(s: str) -> bool {
    s.reduce(List<str>(), lambda(stack, ch) {
        if (ch in "([{") do stack + [ch]
        else if (ch in ")]}") do {
            pairs := {")" : "(", "]" : "[", "}" : "{"}
            if (stack.len() = 0 or stack[-1] != pairs[ch]) do ["INVALID"]
            else stack[:-1]
        } else stack
    })
    |> lambda(stack) { stack.len() = 0 }
}
```

### Quicksort
```
quicksort := lambda(arr: List<int>) -> List<int> {
    if (arr.len() <= 1) do arr
    else {
        pivot := arr[1]
        rest := arr[2:]
        left := rest.filter(lambda(x) { x <= pivot })
        right := rest.filter(lambda(x) { x > pivot })
        quicksort(left) + [pivot] + quicksort(right)
    }
}
```

### Top K Frequent (LC #347)
```
top_k_frequent := lambda(nums: List<int>, k: int) -> List<int> {
    Counter(nums).items()
        .sort(key=lambda(x) { -x[2] })
        .take(k)
        .map(lambda(x) { x[1] })
        .to_list()
}
```

### Palindrome Check
```
is_palindrome := lambda(s: str) -> bool { s = s[::-1] }
```

---

## 17. Compilation Target

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
| `is none` | `.is_none()` |
| struct (data only) | struct (compiler adds `&`, `*`, `Clone` automatically) |
| `lambda(x) { x + 1 }` | `\|x\| x + 1` (closure) |
| `lambda(x) { ... }` | `\|x\| { ... }` (closure) |
| `.map()` / `.filter()` | iterator adapters |
| `x \|> f \|> g` | `g(f(x))` or `.pipe()` chains |
| `if (p) do x else y` | `if p { x } else { y }` (expression) |
| last expression = return | last expression = return (same as Rust) |

---

## Summary

```
# everything is lambda, no return — last expression is the value
add := lambda(a: int, b: int) -> int { a + b }
greet := lambda(name: str) -> str { "hello {name}" }
log := lambda(msg: str) { print(msg) }       # no -> = void

# if/else expression
val := if (x > 0) do x else -x

# pipe operator
result := data |> process |> transform

# 1-based indexing
arr[1]          # first element
arr[2:]         # slice from 2nd
arr[::-1]       # reversed

# tuple unpacking
a, b := b, a

# chaining
[1,2,3].map(lambda(x) { x*2 }).filter(lambda(x) { x > 2 })

# in keyword
if (x in arr) do { ... }
if (not x in seen) do { ... }

# logic (Python keywords)
if (a and b or not c) do { ... }
if (val is none) do { ... }

# no classes, no interfaces, no methods — structs are pure data, all behavior is functions
```
