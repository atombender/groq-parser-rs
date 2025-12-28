# groq-parser

A Rust parser for [GROQ](https://www.sanity.io/docs/groq) (Graph-Relational Object Queries), the
query language created by Sanity.io for filtering and projecting JSON documents.

## Features

- Complete GROQ language support
- Minimal dependencies
- Comprehensive error handling with source position tracking
- Extensively tested (~98 test cases)

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
    Ok(expr) => println!("{:?}", expr),
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

### Current Results (GROQ 1.3)

| Metric          | Count         | Notes                             |
|-----------------|---------------|-----------------------------------|
| Total tests     | 9,867         | (287 legacy 0.1 tests excluded)   |
| Matching        | 8,971 (90.9%) |                                   |
| Status mismatch | 0             |                                   |
| AST mismatch    | 896           | Numeric & param expansion diffs   |

**AST mismatches** fall into two categories:

1. **Numeric formatting** (cosmetic):
   - Float precision (1736 line diffs): Minor floating-point representation differences
   - Float notation (4 line diffs): Different scientific notation thresholds
   - Integer overflow (4 line diffs): Rust uses Float for values > i64::MAX (more precise),
     Go clamps to i64::MAX

2. **Parameter expansion** (526 line diffs): Go expands `$param` references to their literal
   values at parse time. Rust keeps them as `Param` nodes, which is the correct parser behavior
   (evaluation/interpolation is separate from parsing).

## License

MIT License - see [LICENSE](LICENSE) for details.
