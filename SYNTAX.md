# Patina Language Syntax Reference

> *Iron rusts. C(silver) gets patina.*
>
> A language that feels like Python, runs like Rust.

Patina transpiles to Rust and compiles to native machine code. No garbage collector, no runtime — just clean syntax and full performance.

**File extension:** `.pa`

---

## Quick Start

```patina
fn main() {
    var name = "world"
    println("Hello, {name}!")
}
```

```bash
patina hello.pa --run
```

---

## Table of Contents

- [Running Patina](#running-patina)
- [The Basics](#the-basics)
- [Variables](#variables)
- [Types](#types)
- [Functions](#functions)
- [String Interpolation](#string-interpolation)
- [Operators](#operators)
- [Control Flow](#control-flow)
- [Classes](#classes)
- [Methods](#methods)
- [Traits](#traits)
- [Generics](#generics)
- [Enums](#enums)
- [Arrays & Collections](#arrays--collections)
- [Lambdas](#lambdas)
- [Imports](#imports)
- [Constants & Type Aliases](#constants--type-aliases)
- [Type Casting](#type-casting)
- [Standard Library](#standard-library)
- [Patina vs Rust Cheat Sheet](#patina-vs-rust-cheat-sheet)

---

## Running Patina

```bash
patina file.pa              # compile to binary
patina file.pa --run        # compile and run
patina file.pa --emit-rust  # print generated Rust code
patina file.pa --release    # compile with optimizations
patina file.pa -o myapp     # custom output binary name
patina --version            # print version
```

---

## The Basics

Patina strips away Rust's ceremony while keeping its power:

- **No semicolons** — newlines end statements
- **`#` for comments** — like Python
- **No lifetime annotations** — the compiler handles it
- **No `&` references in user code** — borrowing is inferred
- **No macros** — `print()` is just a function
- **No `pub` keyword** — everything is public by default

---

## Variables

```patina
# Immutable (default)
var x = 42
var name: string = "Patina"

# Mutable
mut count = 0
count += 1
count = 99
```

In Rust you'd write `let mut count = 0;` — in Patina it's just `mut`.

---

## Types

| Patina           | Rust             | Notes                        |
|------------------|------------------|------------------------------|
| `int`            | `i64`            | 64-bit integer               |
| `float`          | `f64`            | 64-bit float                 |
| `string`         | `String`         | Owned string                 |
| `bool`           | `bool`           | `true` / `false`             |
| `char`           | `char`           | Single character             |
| `Vec<int>`       | `Vec<i64>`       | Dynamic array                |
| `HashMap<K, V>`  | `HashMap<K, V>`  | Key-value map                |
| `MyStruct`       | `MyStruct`       | Any user-defined type        |
| `T`              | `T`              | Generic type parameter       |

---

## Functions

```patina
fn add(a: int, b: int) -> int {
    return a + b
}

fn greet(name: string) {
    println("Hello, {name}!")
}

fn main() {
    var sum = add(3, 4)
    greet("world")
}
```

- Return type goes after `->` (omit for void functions)
- `main()` is the entry point

---

## String Interpolation

Use `{expression}` inside any string — no f-prefix needed, no macros:

```patina
var name = "Patina"
var x = 42

println("Hello {name}")            # variables
println("x + 1 = {x + 1}")        # expressions
println("sqrt(2) = {sqrt(2.0)}")   # function calls
println("pi = {PI}")               # constants
```

To write a literal `{`, escape it: `\{`.

---

## Operators

### Arithmetic
| Op | Meaning | Example |
|----|---------|---------|
| `+` `-` `*` `/` | basic math | `a + b` |
| `%` | modulo | `10 % 3` → `1` |
| `**` | power | `2 ** 10` → `1024` |

### Comparison
| Op | Meaning |
|----|---------|
| `==` `!=` | equal / not equal |
| `<` `>` `<=` `>=` | ordering |

### Logical
| Op | Meaning |
|----|---------|
| `&&` | and |
| `\|\|` | or |
| `!` | not |

### Assignment
| Op | Meaning |
|----|---------|
| `=` | assign |
| `+=` `-=` `*=` `/=` | compound assign |

### Range
| Op | Meaning | Example |
|----|---------|---------|
| `..` | exclusive range | `0..10` → 0 to 9 |
| `..=` | inclusive range | `0..=10` → 0 to 10 |

---

## Control Flow

### If / Else

```patina
if score >= 90 {
    println("A")
} else if score >= 80 {
    println("B")
} else {
    println("C")
}
```

### While

```patina
mut i = 0
while i < 10 {
    println("{i}")
    i += 1
}
```

### For Loops

```patina
# range (Python-style)
for i in range(10) { ... }           # 0..9
for i in range(5, 10) { ... }        # 5..9
for i in range(0, 100, 5) { ... }    # 0, 5, 10, ... 95

# range (Rust-style)
for i in 0..10 { ... }
for i in 0..=10 { ... }              # inclusive

# iterate over a collection
for item in my_list { ... }
```

### Match

```patina
match day {
    1 => println("Monday")
    2 => println("Tuesday")
    _ => println("other")
}

# with enum variants
match color {
    Color::Red => println("red")
    Color::Custom(r, g, b) => {
        println("rgb({r}, {g}, {b})")
    }
    _ => println("other")
}
```

### Break & Continue

```patina
while true {
    if done {
        break
    }
    if skip_this {
        continue
    }
}
```

---

## Classes

```patina
class Point {
    x: float
    y: float
}

# Create an instance
var p = Point { x: 3.0, y: 4.0 }

# Shorthand when variable matches field name
var x = 3.0
var y = 4.0
var p = Point { x, y }

# Access fields
println("{p.x}, {p.y}")
```

---

## Methods

```patina
methods Point {
    # Associated function (like a static method)
    fn new(x: float, y: float) -> Point {
        return Point { x, y }
    }

    # Instance method (takes self)
    fn magnitude(self) -> float {
        return sqrt(self.x ** 2 + self.y ** 2)
    }

    # Mutable method
    fn scale(mut self, factor: float) {
        self.x *= factor
        self.y *= factor
    }
}

# Call associated function with dot syntax
var p = Point.new(3.0, 4.0)

# Call instance method
var m = p.magnitude()
```

---

## Traits

Traits define shared behavior — like interfaces with optional default implementations:

```patina
trait Shape {
    fn area(self) -> float               # abstract — must implement
    fn perimeter(self) -> float          # abstract

    fn describe(self) -> string {        # default — can override
        return "a shape"
    }
}

methods Shape for Circle {
    fn area(self) -> float {
        return PI * self.radius ** 2
    }

    fn perimeter(self) -> float {
        return 2.0 * PI * self.radius
    }
}
```

Use traits as bounds on generic functions:

```patina
fn print_area<T: Shape>(s: T) {
    println("Area: {s.area()}")
}
```

---

## Generics

### Generic functions

```patina
fn largest<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        return a
    }
    return b
}

# Multiple trait bounds with +
fn display_shape<T: Shape + Describable>(s: T) { ... }
```

### Generic classes

```patina
class Pair<T> {
    first: T
    second: T
}

class Entry<K, V> {
    key: K
    value: V
}

var p = Pair { first: 1, second: 2 }
```

### Generic enums

```patina
enum Option<T> {
    Some(T)
    None
}
```

---

## Enums

```patina
enum Direction {
    North
    South
    East
    West
}

enum Result {
    Ok(string)
    Error(int, string)
}

# Construct with :: syntax
var d = Direction::North
var r = Result::Error(404, "not found")

# Match on them
match d {
    Direction::North => println("going up")
    _ => println("going somewhere else")
}
```

---

## Arrays & Collections

```patina
# Arrays (become Vec in Rust)
var nums = [1, 2, 3, 4, 5]
var first = nums[0]

mut list = [10, 20]
list.push(30)

# HashMap
use std.collections.HashMap
mut scores: HashMap<string, int> = HashMap.new()
scores.insert("Alice".to_string(), 95)
```

---

## Lambdas

```patina
var doubled = map([1, 2, 3], |x| x * 2)
var evens = filter(numbers, |n| n % 2 == 0)
var pairs = zip(names, scores)
```

---

## Imports

Use **dots** instead of Rust's `::` for paths:

```patina
use std.collections.HashMap
use std.collections.{HashMap, HashSet}   # grouped
use std.io.*                              # glob
use std.collections.HashMap as Map        # alias
```

---

## Constants & Type Aliases

```patina
const MAX_SIZE: int = 1024
const GREETING = "hello"         # type inferred

type IntList = Vec<int>
type Matrix = Vec<Vec<float>>
type StringMap = HashMap<string, string>
```

---

## Type Casting

```patina
var f = 42 as float       # int → float
var n = 3.99 as int       # float → int (truncates)
```

---

## Standard Library

All of these are built-in — no imports needed.

### Math

| Function | Description |
|----------|-------------|
| `sqrt(x)` | Square root |
| `abs(x)` | Absolute value |
| `pow(base, exp)` | Power |
| `min(a, b)` | Minimum |
| `max(a, b)` | Maximum |
| `clamp(val, lo, hi)` | Clamp to range |
| `floor(x)` | Round down |
| `ceil(x)` | Round up |
| `round(x)` | Round to nearest |
| `sin(x)` `cos(x)` `tan(x)` | Trigonometry |
| `log(x)` `log2(x)` `log10(x)` | Logarithms |

**Constants:** `PI`, `E`, `TAU`, `INFINITY`, `NEG_INFINITY`, `NAN`, `MAX_INT`, `MIN_INT`

### Strings

| Function | Description |
|----------|-------------|
| `to_upper(s)` | `"hello"` → `"HELLO"` |
| `to_lower(s)` | `"HELLO"` → `"hello"` |
| `trim(s)` | Strip whitespace |
| `split(s, delim)` | Split into array |
| `join(arr, delim)` | Join array into string |
| `contains(s, sub)` | Check substring |
| `starts_with(s, prefix)` | Check prefix |
| `ends_with(s, suffix)` | Check suffix |
| `replace(s, from, to)` | Replace occurrences |
| `chars(s)` | String → array of chars |

### Collections

| Function | Description |
|----------|-------------|
| `len(x)` | Length |
| `push(vec, val)` | Append |
| `pop(vec)` | Remove last |
| `sort(vec)` | Return sorted copy |
| `reverse(vec)` | Return reversed copy |
| `map(vec, fn)` | Transform each element |
| `filter(vec, fn)` | Keep matching elements |
| `enumerate(vec)` | Pairs of (index, value) |
| `zip(a, b)` | Pair up two arrays |

### Ranges

| Call | Result |
|------|--------|
| `range(5)` | 0, 1, 2, 3, 4 |
| `range(2, 5)` | 2, 3, 4 |
| `range(0, 10, 2)` | 0, 2, 4, 6, 8 |

### I/O

| Function | Description |
|----------|-------------|
| `print(msg)` | Print (no newline) |
| `println(msg)` | Print with newline |
| `input(prompt)` | Read line from stdin |
| `read_file(path)` | Read file to string |
| `write_file(path, content)` | Write string to file |
| `append_file(path, content)` | Append to file |
| `file_exists(path)` | Check if file exists |

### Conversion

| Function | Description |
|----------|-------------|
| `str(x)` | Anything → string |
| `int(s)` | String → int |
| `float(s)` | String → float |

### Misc

| Function | Description |
|----------|-------------|
| `exit(code)` | Exit program |
| `type_of(x)` | Get type name (for debugging) |
| `assert(cond)` | Panic if false |
| `assert(cond, msg)` | Panic with message |

---

## Patina vs Rust Cheat Sheet

| What | Patina | Rust |
|------|--------|------|
| Variable | `var x = 5` | `let x: i64 = 5;` |
| Mutable | `mut x = 5` | `let mut x: i64 = 5;` |
| Print | `println("hi {x}")` | `println!("hi {}", x);` |
| Read input | `input("Name: ")` | `io::stdin().read_line(...)` |
| Types | `int`, `float`, `string` | `i64`, `f64`, `String` |
| Comments | `# comment` | `// comment` |
| Power | `x ** 2` | `x.pow(2)` |
| Imports | `use std.io` | `use std::io;` |
| Statement end | newline | `;` |
| Class | `class Point { ... }` | `struct Point { ... }` |
| Methods | `methods Point { ... }` | `impl Point { ... }` |
| Associated fn | `Type.new()` | `Type::new()` |
| Semicolons | never | everywhere |
| Lifetimes | never | `'a` everywhere |

---

*Patina v0.2 — because code should be easy to write and fast to run.*
