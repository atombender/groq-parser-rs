# groq-parser

A Rust parser for [GROQ](https://www.sanity.io/docs/groq) (Graph-Relational Object Queries), the
query language created by Sanity.io for filtering and projecting JSON documents.

## Features

- Complete GROQ language support (version 1.3)
- Minimal dependencies
- Comprehensive error handling with source position tracking
- Extensively tested (~98 test cases)

## Compatibility

This library may return floating point numbers that differ from the reference implementation
due to differences in floating point representation and evaluation in Rust.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
groq-parser = { path = "path/to/groq-parser" }
```

## Usage

```rust
use groq_parser::parser::Parser;

fn main() {
  let query = r#"*[_type == "post"]{title, author->name}"#;
  let mut parser = Parser::new(query);

  match parser.parse() {
    Ok(result) => println!("{:?}", result.expr),
    Err(e) => eprintln!("Parse error: {}", e),
  }
}
```

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test --lib
```

## Comparison Tool

The `comparison/` directory contains tools for comparing this parser against the reference
[go-groq](https://github.com/sanity-io/go-groq) implementation.

### Setup

```bash
# Build the Rust AST dumper
cargo build --release --bin dump_ast

# Build the Go AST dumper
cd comparison/go-dumper
go build -o go-dumper .
cd ../..
```

### Running Comparisons

Using the built [GROQ test suite](https://github.com/sanity-io/groq-test-suite):

```bash
cd comparison
python3 compare.py ../groq-test-suite.ndjson ../target/release/dump_ast ./go-dumper/go-dumper
```

Filter by GROQ spec version (excludes legacy 0.1 tests):

```bash
python3 compare.py ../groq-test-suite.ndjson ../target/release/dump_ast ./go-dumper/go-dumper --version 1.3
```

## License

MIT License - see [LICENSE](LICENSE) for details.
