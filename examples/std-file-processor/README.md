# std-file-processor

A word-frequency file processor built with **zero third-party dependencies** —
only the Rust standard library. It reads a single text file or every `.txt`
file in a directory, counts word frequencies line by line, and prints (and
optionally saves) a small report.

## Running

```bash
# Count words in one file
cargo run -- path/to/file.txt

# Count words across every .txt file in a directory, show the top 20
cargo run -- path/to/dir --top 20

# Also save the report
cargo run -- path/to/dir --save report.txt

# Run tests
cargo test
```

## What It Demonstrates

- **`Path` / `PathBuf`** for filesystem handling
- **`fs`, `File`, `BufReader`** with line-by-line reading via `BufRead`
- **A hand-rolled error enum** implementing `std::error::Error`, `Display`,
  and `From<io::Error>` — no `thiserror`
- **`Result` + `?`** for error propagation
- **Iterator combinators** — `flat_map`, `filter_map`, `map_while`, `fold` —
  collecting into a `HashMap`
- **Sorting** word/count pairs by count (then alphabetically) with `Vec::sort_by`
- **Clean ownership**: pure functions over owned data, tested in isolation
- **Std-only testing**: unit tests for the pure functions plus integration
  tests that create fixtures in `std::env::temp_dir()` with `std::fs`
  (no `tempfile` crate)
- **Hand-written CLI parsing** with `std::env::args` (no `clap`)

## Project Structure

```text
src/
├── lib.rs    # Error type + pure counting/reporting logic (unit-tested)
└── main.rs   # Thin CLI shell: argument parsing, exit codes
tests/
└── integration.rs  # Filesystem fixtures in the temp dir
```
