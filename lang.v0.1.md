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

**No `def` `func` keywords.** All functions are lambdas bound to names with `:=`.

```
# named function
add := lambda(a: int, b: int) -> int {
    return a + b
}

# single-expression
double := lambda(x: int) -> int { x * 2 }

# no return type = void
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
if x in arr { ... }
if k in dict { ... }
if ch in "aeiou" { ... }
if not x in seen { ... }

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

## 9. Logical Operators (Python keywords)

```
if x > 0 and x < 100 {
    ...
}

if name is none {
    ...
}

if not name is none {
    ...
}

if a or b {
    ...
}

if not valid {
    ...
}
```

- `and` — logical AND
- `or` — logical OR
- `not` — logical NOT
- `is` — identity/type check
- use `not x is y` for negation (no `is not` — keeps `not` always at the front)

### Comparison
`=` or `==` for value equality (both work), `!=`, `<`, `>`, `<=`, `>=`. No ambiguity since `:=` is always assignment.

---

## 10. Control Flow

### if / elif / else
```
if x > 0 {
    print("positive")
} elif x == 0 {
    print("zero")
} else {
    print("negative")
}
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

## 11. Structs (no classes, no methods, no interfaces)

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
    return node.left is none and node.right is none
}

depth := lambda(node: TreeNode) -> int {
    mut l := 0
    mut r := 0
    if not node.left is none { l := depth(node.left) }
    if not node.right is none { r := depth(node.right) }
    return 1 + max(l, r)
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

## 12. String Interpolation

```
name := "world"
msg := "hello {name}, result is {1 + 2}"
```

All strings support interpolation by default (no `f` prefix needed).

---

## 13. Error Handling

Result-based (like Rust), with cleaner syntax:

```
read_file := lambda(path: str) -> Result<str, Error> {
    ...
}

# propagate with ?
process := lambda() -> Result<Data, Error> {
    content := read_file("data.txt")?
    return parse(content)
}

# handle explicitly
match read_file("data.txt") {
    Ok(content) => print(content),
    Err(e) => print("failed: {e}"),
}
```

---

## 14. Modules

```
# math_utils.lang
pub square := lambda(x: int) -> int {
    return x * x
}

# main.lang
import math_utils
result := math_utils.square(5)

# or selective import
from math_utils import square
result := square(5)
```

---

## 15. Examples: LeetCode

### Two Sum (LC #1)
```
two_sum := lambda(nums: List<int>, target: int) -> List<int> {
    seen := Dict<int, int>()
    for i, n in nums.enumerate() {
        comp := target - n
        if comp in seen {
            return [seen[comp], i]
        }
        seen[n] := i
    }
    return []
}
```

### Valid Parentheses (LC #20)
```
is_valid := lambda(s: str) -> bool {
    stack := List<str>()
    pairs := {")" : "(", "]" : "[", "}" : "{"}
    for ch in s {
        if ch in "([{" {
            stack.push(ch)
        } elif ch in pairs {
            if stack.len() == 0 or stack[-1] != pairs[ch] {
                return false
            }
            stack.pop()
        }
    }
    return stack.len() == 0
}
```

### BFS Shortest Path (LC #127 style)
```
bfs := lambda(graph: Dict<int, List<int>>, start: int, end: int) -> int {
    q := Deque<Tuple<int, int>>()
    visited := Set<int>()
    q.push_back((start, 0))
    visited.add(start)

    while q.len() > 0 {
        node, dist := q.pop_front()
        if node == end {
            return dist
        }
        for nei in graph[node] {
            if not nei in visited {
                visited.add(nei)
                q.push_back((nei, dist + 1))
            }
        }
    }
    return -1
}
```

### Quicksort
```
quicksort := lambda(arr: List<int>) -> List<int> {
    if arr.len() <= 1 {
        return arr
    }
    pivot := arr[1]
    rest := arr[2:]
    left := rest.filter(lambda(x) { x <= pivot })
    right := rest.filter(lambda(x) { x > pivot })
    return quicksort(left) + [pivot] + quicksort(right)
}
```

### Top K Frequent (LC #347)
```
top_k_frequent := lambda(nums: List<int>, k: int) -> List<int> {
    cnt := Counter(nums)
    return cnt.items()
        .sorted(key=lambda(x) { -x[2] })
        .take(k)
        .map(lambda(x) { x[1] })
        .to_list()
}
```

### Palindrome Check
```
is_palindrome := lambda(s: str) -> bool {
    return s == s[::-1]
}
```

---

## 16. Compilation Target

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

---

## Summary

```
# everything is lambda
add := lambda(a: int, b: int) -> int { a + b }

greet := lambda(name: str) -> str {
    return "hello {name}"
}

# 1-based indexing
arr[1]          # first element
arr[2:]         # slice from 2nd
arr[::-1]       # reversed

# tuple unpacking
a, b := b, a

# chaining
[1,2,3].map(lambda(x) { x*2 }).filter(lambda(x) { x > 2 })

# in keyword
if x in arr { ... }
if not x in seen { ... }

# logic (Python keywords)
if a and b or not c { ... }
if val is none { ... }

# no classes, no interfaces, no methods — structs are pure data, all behavior is functions
```
