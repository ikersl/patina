# Patina

**A programming language that feels like Python, runs like Rust.**

> Iron rusts. C(silver) gets patina.

Patina transpiles `.pa` source to Rust, then compiles to a native binary via `rustc`. You get Rust's speed, safety, and zero-cost abstractions — without the syntax tax.

## Quick Start

```patina
fn main() {
    var name = "world"
    println("Hello, {name}!")

    for i in range(5) {
        println("  {i} squared = {i ** 2}")
    }
}
```

```bash
patina hello.pa --run
```

## Install

You need Rust installed ([rustup.rs](https://rustup.rs)). Then:

```bash
git clone https://github.com/ikersl/patina && cd patina
cargo build --release
./target/release/patina examples/hello.pa --run
```

Or during development:

```bash
cargo run -- examples/hello.pa --run
```

## What Makes Patina Different

| What | Patina | Rust |
|------|--------|------|
| Variable | `var x = 5` | `let x: i64 = 5;` |
| Mutable | `mut x = 5` | `let mut x: i64 = 5;` |
| Print | `println("hi {x}")` | `println!("hi {}", x);` |
| Types | `int`, `float`, `string` | `i64`, `f64`, `String` |
| Class | `class Point { ... }` | `struct Point { ... }` |
| Methods | `methods Point { ... }` | `impl Point { ... }` |
| Power | `x ** 2` | `x.pow(2)` |
| Comments | `# comment` | `// comment` |
| Imports | `use std.io` | `use std::io;` |
| Semicolons | never | everywhere |
| Lifetimes | never | `'a` everywhere |

## Features

- **No semicolons** — newlines end statements
- **Simple types** — `int`, `float`, `string`, `bool`, `char`
- **String interpolation** — `"{name} is {age}"` with any expression
- **No lifetimes** — the transpiler handles borrowing
- **`**` power operator** — `x ** 2` instead of `x.pow(2)`
- **Python-style `range()`** — `range(10)`, `range(1, 10)`, `range(0, 100, 5)`
- **`# comments`** — like Python
- **Classes & methods** — `class`, `methods`, `trait` keywords
- **Generics & traits** — full Rust-style polymorphism
- **Enums with pattern matching** — `match` with `=>` arms
- **50+ builtin functions** — math, strings, collections, I/O
- **Colored error messages** — with source context and caret

## Standard Library

All built in — no imports needed.

| Category | Functions |
|----------|-----------|
| **Math** | `sqrt` `abs` `pow` `min` `max` `clamp` `floor` `ceil` `round` `sin` `cos` `tan` `log` |
| **Strings** | `split` `join` `trim` `to_upper` `to_lower` `contains` `replace` `starts_with` `ends_with` `chars` |
| **Collections** | `len` `push` `pop` `sort` `reverse` `map` `filter` `enumerate` `zip` `range` |
| **I/O** | `print` `println` `input` `read_file` `write_file` `append_file` `file_exists` |
| **Conversion** | `str` `int` `float` |
| **Constants** | `PI` `E` `TAU` `INFINITY` `MAX_INT` `MIN_INT` |

## Example

```patina
trait Shape {
    fn area(self) -> float
}

class Circle { radius: float }

methods Shape for Circle {
    fn area(self) -> float {
        return PI * self.radius ** 2
    }
}

fn main() {
    var c = Circle { radius: 5.0 }
    println("Area: {c.area()}")
}
```

## CLI

```bash
patina file.pa              # compile to binary
patina file.pa --run        # compile and run
patina file.pa --emit-rust  # show generated Rust code
patina file.pa --release    # compile with optimizations
patina file.pa -o myapp     # custom output name
patina --version            # print version
```

## Docs

- [Full Syntax Reference](SYNTAX.md)
- [Homepage](https://chaimon.com/patina/)

## License

No rights reserved.
