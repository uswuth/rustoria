# Error Handling in Rust

## Overview

Rust has no exceptions. Errors are values — typically `Result<T, E>` or `Option<T>`. The compiler forces you to handle
(or explicitly propagate) every error.

---

## `Result<T, E>` vs `Option<T>`

| Type             | Use When                                     | Variants            |
| ---------------- | -------------------------------------------- | ------------------- |
| `Result<T, E>`   | Operation can fail with a meaningful error   | `Ok(T)`, `Err(E)`   |
| `Option<T>`      | Value may be absent, no error to report      | `Some(T)`, `None`   |

```rust
// Result: parsing can fail with a specific error
fn parse_port(s: &str) -> Result<u16, ParseError> {
    s.parse::<u16>().map_err(|e| ParseError::InvalidPort(e))
}

// Option: looking up a key may return nothing
fn get<'a>(map: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    map.get(key).map(|s| s.as_str())
}
```

### Converting between them

```rust
// Option -> Result
let val = some_option.ok_or(Error::NotFound)?;

// Result -> Option (discards error)
let val = some_result.ok();

// Option -> Result with context
let val = some_option.ok_or_else(|| Error::Custom("was none".into()))?;
```

---

## `thiserror` vs `anyhow`

### `thiserror` — for libraries

Define structured, typed errors. Each error variant can carry data.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("not found: {resource} with id {id}")]
    NotFound { resource: String, id: u64 },

    #[error("validation failed: {field}")]
    Validation { field: String },
}
```

**When to use `thiserror`:**

- Library code — consumers need to match on specific error variants.
- When you want `#[from]` for automatic `?` conversion.
- When you want `Display` and `Error` impls derived.

### `anyhow` — for applications

Ergonomic error handling with context. Errors are opaque — you don't match on variants.

```rust
use anyhow::{Context, Result};

fn load_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config from {path}"))?;

    let config: Config = toml::from_str(&content)
        .context("failed to parse config as TOML")?;

    Ok(config)
}
```

**When to use `anyhow`:**

- Application code — you care about the error message, not the variant.
- When you want to add context with `.context()` or `.with_context()`.
- When you don't want to define an error enum.

### Mixing them

Libraries use `thiserror`. Applications use `anyhow`. At the boundary, convert:

```rust
// Library function
fn library_fn() -> Result<(), LibraryError> { /* ... */ }

// Application calls it
fn app_fn() -> anyhow::Result<()> {
    // `?` converts LibraryError into anyhow::Error automatically,
    // via anyhow's blanket From<E: Error + Send + Sync + 'static> impl
    library_fn()?;
    Ok(())
}
```

---

## Error Propagation with `?`

The `?` operator propagates errors. If the value is `Err`, it returns early. If `Ok`, it unwraps.

```rust
fn read_config(path: &str) -> Result<Config, AppError> {
    let content = std::fs::read_to_string(path)?; // io::Error -> AppError via From
    let config: Config = serde_json::from_str(&content)?; // serde_json::Error -> AppError
    Ok(config)
}
```

### `?` in functions returning `Option`

```rust
fn find_user(id: u64) -> Option<User> {
    let conn = get_connection()?; // returns None if no connection
    conn.query_user(id) // returns None if not found
}
```

### `?` in `main`

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config("app.toml")?;
    run(config)?;
    Ok(())
}
```

---

## Combinator Cheat Sheet

For simple transformations, prefer combinators over `match`:

| Method | On | Does |
| -------- | ---- | ------ |
| `map(f)` / `map_err(f)` | both | Transform the success (or error) value |
| `and_then(f)` | both | Chain a step that itself returns `Option`/`Result` (flattens) |
| `or_else(f)` | both | Recover: on `None`/`Err`, produce an alternative from `f` |
| `ok_or(e)` / `ok_or_else(f)` | `Option` | `Option` → `Result`; `_else` builds the error lazily |
| `map_or(d, f)` / `map_or_else(d, f)` | both | Fold to one value with a default; `_else` computes the default lazily |
| `as_deref()` | `Option<T: Deref>` | `Option<String>` → `Option<&str>` without consuming |
| `transpose()` | `Option<Result<T, E>>` | Swap to `Result<Option<T>, E>` — pairs with `?` |
| `flatten()` | `Option<Option<T>>`; `Result<Result<T, E>, E>` (1.89+) | Collapse one level of nesting |
| `inspect(f)` / `inspect_err(f)` | both (1.76+) | Peek at the value (e.g., log) without consuming it |

```rust
// transpose + ?: propagate errors while keeping the Option
let maybe: Option<Result<Config, Error>> = path.map(load_config);
let config: Option<Config> = maybe.transpose()?;

// inspect_err: log without restructuring the flow
let cfg = load_config("app.toml")
    .inspect_err(|e| eprintln!("config load failed: {e}"))?;
```

---

## Error Context

Adding context to errors makes debugging easier. Without context, you know *what* failed but not *why*.

### With `anyhow`

```rust
use anyhow::Context;

let user = database::find_user(id)
    .with_context(|| format!("failed to find user {id}"))?
    .ok_or_else(|| anyhow::anyhow!("user {id} not found"))?;
```

### With `thiserror`

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("failed to process order {order_id}: {source}")]
    OrderProcessing {
        order_id: u64,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
```

### The `snafu` alternative

`snafu` provides context selectors:

```rust
use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("failed to load config from {path}"))]
    LoadConfig {
        path: String,
        source: std::io::Error,
    },
}

let config = load_config(&path).context(LoadConfigSnafu { path })?;
```

---

## Custom Error Types

### Minimal custom error

```rust
use std::fmt;

#[derive(Debug)]
struct MyError {
    message: String,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MyError {}
```

### Error with source chain

```rust
#[derive(Debug)]
struct AppError {
    kind: ErrorKind,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(Debug)]
enum ErrorKind {
    Network,
    Database,
    Config,
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}
```

### `Error::source` and the error chain

```rust
fn print_error_chain(err: &(dyn std::error::Error + 'static)) {
    eprintln!("error: {err}");
    let mut source = std::error::Error::source(err);
    while let Some(err) = source {
        eprintln!("  caused by: {err}");
        source = std::error::Error::source(err);
    }
}
```

---

## Common Pitfalls

### 1. Using `unwrap()` in production code

```rust
// Bad: panics on error
let config: Config = serde_json::from_str(&content).unwrap();

// Good: propagate the error
let config: Config = serde_json::from_str(&content)?;
```

**Exception:** `unwrap()` is acceptable in tests, `main` (where you want to crash), and when you've proven the value
can't be `None`/`Err`.

### 2. Swallowing errors

`let _ = ...` and `.ok()` are the *same* mistake — both discard the error silently:

```rust
// Bad: the error is gone, and nobody knows the cleanup failed
let _ = std::fs::remove_file("/tmp/lock");
std::fs::remove_file("/tmp/lock").ok();

// Good: propagate...
std::fs::remove_file("/tmp/lock")?;

// ...or handle it explicitly, saying why failure is acceptable at this site:
if let Err(e) = std::fs::remove_file("/tmp/lock") {
    eprintln!("could not remove stale lock file, continuing anyway: {e}");
}
```

Discarding is only acceptable when the error genuinely carries no information —
make that a conscious, commented choice, not a habit.

### 3. `Box<dyn Error>` loses type information

```rust
// Bad for libraries: callers can't match on specific error types
fn library_fn() -> Result<(), Box<dyn std::error::Error>> { /* ... */ }

// Good for libraries: a concrete error type
fn library_fn() -> Result<(), AppError> { /* ... */ }
```

The rule, reconciled with `main() -> Result<(), Box<dyn Error>>` earlier:
**libraries return concrete error types** (usually via `thiserror`) so callers can
inspect and handle specific failures; **application entry points** may use
`Box<dyn Error>` or `anyhow::Result`, where nothing downstream needs to match on
the error and ergonomics win.

### 4. Not implementing `Send + Sync` on error types

If you use `?` in async code or across threads, your error type must be `Send + Sync`:

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("io error")]
    Io(#[from] std::io::Error), // std::io::Error is Send + Sync
}
```

### 5. Overly broad error types

```rust
// Bad: one giant error enum for the whole app
enum AppError {
    Database,
    Network,
    Config,
    Auth,
    // ... dozens of unrelated variants every caller must wade through
}

// Good: module-specific errors, composed at the top level
mod database {
    pub enum Error { Connection, Query }
}
mod network {
    pub enum Error { Timeout, Tls }
}
```

---

## Summary

| Pattern | Use When | Crate |
| --------- | ---------- | ------- |
| `Result<T, E>` | Fallible operations | std |
| `Option<T>` | Optional values | std |
| `thiserror` | Library error types | thiserror |
| `anyhow` | Application error handling | anyhow |
| `?` operator | Error propagation | std |
| `.context()` | Adding error context | anyhow |
| `Error::source` | Error chains | std |
