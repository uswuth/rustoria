# Testing Patterns in Rust

## Overview

Rust has built-in testing support via `#[test]` and the `cargo test` command. The testing ecosystem includes unit tests,
integration tests, property-based testing, and async testing.

---

## Unit Tests

Unit tests live in the same file as the code they test, in a `#[cfg(test)]` module.

```rust
// src/lib.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-2, -3), -5);
    }

    #[test]
    fn test_add_zero() {
        assert_eq!(add(0, 0), 0);
    }
}
```

### Test naming conventions

- `test_<function>_<scenario>` — e.g., `test_parse_valid_email`.
- `test_<function>_<scenario>_<expected>` — e.g., `test_divide_by_zero_panics`.

### Assertion macros

| Macro | Use When |
| ------- | ---------- |
| `assert!(expr)` | Boolean condition |
| `assert_eq!(a, b)` | Equality (implements `PartialEq + Debug`) |
| `assert_ne!(a, b)` | Inequality |
| `assert_relative_eq!(a, b, epsilon = 1e-6)` | Floating-point comparison (`approx` crate macros; also `assert_abs_diff_eq!`) |

### Testing panics

```rust
#[test]
#[should_panic(expected = "division by zero")]
fn test_divide_by_zero() {
    divide(1, 0);
}
```

### Testing with `Result`

```rust
#[test]
fn test_parse_valid() -> Result<(), ParseError> {
    let result = parse("valid input")?;
    assert_eq!(result.value, 42);
    Ok(())
}
```

---

## Integration Tests

Integration tests live in the `tests/` directory and test the public API of your crate.

```rust
// tests/integration_test.rs
use mycrate::add;

#[test]
fn test_add_integration() {
    assert_eq!(add(1, 2), 3);
}
```

### Sharing test utilities

```rust
// tests/common/mod.rs
pub fn setup_test_db() -> TestDb {
    TestDb::new()
}

// tests/integration_test.rs
mod common;

#[test]
fn test_with_db() {
    let db = common::setup_test_db();
    // ...
}
```

### Testing binary crates

```rust
// tests/cli_test.rs
use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Usage:"));
}
```

---

## Documentation Tests

Code blocks in doc comments (`///`) are compiled and executed by
`cargo test --doc` — and a plain `cargo test` run includes them:

```rust
/// Adds two numbers.
///
/// ```
/// assert_eq!(mycrate::add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Why they matter: doc tests are the first usage examples users read, and they
**can't go stale** — when the API changes, the outdated example fails the build.
Like integration tests, they exercise the crate's public API from the outside.
Hide setup plumbing with `#` so the visible example stays focused:

```rust
/// ```
/// # fn main() -> anyhow::Result<()> {
/// let cfg = mycrate::load_config("app.toml")?;
/// # Ok(()) }
/// ```
```

A code block can also be marked `no_run` (compile but don't execute), `ignore`
(skip entirely), or `compile_fail` (must NOT compile — for documenting misuse).

---

## Property-Based Testing with `proptest`

Property-based testing generates random inputs and checks that properties hold.

### Setup

```toml
[dev-dependencies]
proptest = "1"
```

### Writing property tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_add_commutative(a in 0..1000i32, b in 0..1000i32) {
        prop_assert_eq!(add(a, b), add(b, a));
    }

    #[test]
    fn test_add_associative(a in 0..1000i32, b in 0..1000i32, c in 0..1000i32) {
        prop_assert_eq!(add(add(a, b), c), add(a, add(b, c)));
    }

    #[test]
    fn test_reverse_reverse_is_identity(
        v in prop::collection::vec(0..1000i32, 0..100)
    ) {
        let mut reversed = v.clone();
        reversed.reverse();
        reversed.reverse();
        prop_assert_eq!(v, reversed);
    }
}
```

### Custom strategies

```rust
use proptest::prelude::*;

// Generate valid email addresses — the regex strategy already yields String,
// no prop_map needed
fn valid_email() -> impl Strategy<Value = String> {
    "[a-z]{1,10}@[a-z]{1,5}\\.(com|org|net)"
}

proptest! {
    #[test]
    fn test_email_validation(email in valid_email()) {
        prop_assert!(is_valid_email(&email));
    }
}
```

### When to use property-based testing

- **Invariants** — properties that should always hold (e.g., `reverse(reverse(x)) == x`).
- **Edge cases** — the generator finds cases you didn't think of.
- **Round-trips** — encode then decode should give the original.

### When NOT to use property-based testing

- When you need specific test cases (use unit tests).
- When the property is hard to express.
- When the generator is too slow.

---

## Async Testing

### `#[tokio::test]`

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_add(2, 3).await;
    assert_eq!(result, 5);
}
```

### `#[tokio::test(flavor = "multi_thread")]`

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent() {
    let mut handles = vec![];
    for i in 0..10 {
        handles.push(tokio::spawn(async move {
            async_add(i, i).await
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }
}
```

### Testing with mock services

**Object-safety caveat:** a trait with a native `async fn` is not dyn-compatible,
so `Box<dyn UserRepository>` won't compile for the trait below unless the async
method is boxed. `#[async_trait]` does that boxing, and `mockall` mocks the
rewritten trait transparently — use both together:

```rust
use async_trait::async_trait;
use mockall::{automock, predicate::eq};

#[automock]
#[async_trait]
trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: u64) -> Option<User>;
}

#[tokio::test]
async fn test_service_with_mock() {
    let mut mock = MockUserRepository::new();
    mock.expect_find_by_id()
        .with(eq(42))
        .returning(|_| Some(User { id: 42, name: "Alice".into() }));

    let service = UserService::new(Box::new(mock));
    let user = service.get_user(42).await.unwrap();
    assert_eq!(user.name, "Alice");
}
```

---

## Running Tests

```bash
cargo test                          # unit + integration + doc tests
cargo test name_filter              # only tests whose name contains the filter
cargo test -- --nocapture           # show println!/eprintln! output from tests
cargo test -- --test-threads=1      # run serially (tests sharing a resource)
cargo test -- --ignored             # run only the #[ignore]'d (slow) tests
```

### `cargo nextest`

`cargo nextest run` is a faster test runner: process-per-test isolation (a hang
or crash doesn't take down the whole suite), better output, JUnit XML for CI,
and retries for flaky tests. It's the standard choice for CI on larger projects.

---

## Test Organization

### Test module structure

```text
src/
  lib.rs
  users/
    mod.rs
    service.rs
    service_tests.rs    # or inline #[cfg(test)] module
tests/
  integration_test.rs
  common/
    mod.rs
```

### `#[cfg(test)]` vs separate test files

| Approach | Pros | Cons |
| ---------- | ------ | ------ |
| Inline `#[cfg(test)]` | Access to private items | Clutters source files |
| Separate `*_tests.rs` | Cleaner source files | Only access to public items |
| `tests/` directory | Tests public API only | Can't test private items |

**Recommendation:** Use inline `#[cfg(test)]` for unit tests, `tests/` for integration tests.

### Test helpers

```rust
// tests/common/mod.rs
pub fn create_test_user() -> User {
    User {
        id: UserId::new(),
        name: "Test User".into(),
        email: "test@example.com".into(),
    }
}

pub fn create_test_db() -> TestDb {
    TestDb::new("sqlite::memory:")
}
```

---

## Test Coverage

### `cargo-tarpaulin`

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### `llvm-cov`

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

---

## Common Pitfalls

### 1. Tests that depend on execution order

```rust
// Bad: test_b depends on test_a having run first — tests run in parallel
// and in arbitrary order, so this flakes. (Edition 2024 goes further:
// creating a reference to a `static mut` — including the implicit ones in
// assert_eq! — is a hard error via the `static_mut_refs` lint.)
static mut COUNTER: u32 = 0;

#[test]
fn test_a() {
    unsafe { COUNTER += 1; }
}

#[test]
fn test_b() {
    unsafe { assert_eq!(COUNTER, 1); }
}

// Good: each test is independent — no shared state at all
#[test]
fn test_counter_increment() {
    let counter = Counter::new();
    counter.increment();
    assert_eq!(counter.get(), 1);
}

// If tests genuinely must share process-wide state, use an atomic (or a
// Mutex) and never depend on another test having run first:
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
fn test_increments_counter() {
    let before = COUNTER.fetch_add(1, Ordering::SeqCst);
    assert_eq!(COUNTER.load(Ordering::SeqCst), before + 1);
}
```

### 2. Ignoring test results

```rust
// Bad: test passes even if the function panics
#[test]
fn test_bad() {
    let _ = std::panic::catch_unwind(|| {
        dangerous_function();
    });
}

// Good: use should_panic or assert
#[test]
#[should_panic]
fn test_good() {
    dangerous_function();
}
```

### 3. Slow tests in the main suite

```rust
#[test]
#[ignore]
fn test_slow_integration() {
    // Run with: cargo test -- --ignored
}
```

### 4. Not testing error paths

```rust
// Bad: only tests the happy path
#[test]
fn test_parse_valid() {
    assert!(parse("valid").is_ok());
}

// Good: also test error paths
#[test]
fn test_parse_invalid() {
    assert!(parse("").is_err());
    assert!(parse("invalid").is_err());
}
```

---

## Summary

| Pattern | Use When | Tool |
| --------- | ---------- | ------ |
| Unit tests | Testing individual functions | `#[test]` |
| Integration tests | Testing public API | `tests/` directory |
| Doc tests | Keeping examples compiling | `///` code blocks |
| Fast test runs / CI | Isolation, retries, reporting | `cargo nextest` |
| Property-based | Testing invariants | `proptest` |
| Async testing | Testing async code | `#[tokio::test]` |
| Mock testing | Isolating dependencies | `mockall` |
| Coverage | Measuring test coverage | `cargo-tarpaulin` |
