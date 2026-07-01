# regex

A small regex engine in Rust. Compiles patterns into a table-driven NFA, then matches inputs by walking prebuilt transition columns.

## Build

```bash
cargo build --release
```

## Usage

```rust
use regex::Fsm;

let fsm = Fsm::compile("a+bc");
assert!(fsm.match_str("abc"));
```

Run the demo binary:

```bash
cargo run
```

## Test

```bash
cargo test
./verify.sh
```

## Supported syntax

- Literals, `.`, `*`, `+`, `?`, `{n}`, `{n,m}`, `{n,}`
- Alternation `|`, grouping `( )`
- Character classes `[a-z]`, `[^0-9]`
- Anchors `^`, `$`
- Escapes `\d`, `\w`, `\s`, `\D`, `\W`, `\S`, and literal escapes like `\.`

Patterns compile once; the same `Fsm` can be reused across many inputs.
