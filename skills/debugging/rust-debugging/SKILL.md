---
name: rust-debugging
description: Use when debugging Rust code or fixing errors.
---

# Rust Debugging Guide

## Compiler Errors

### 1. Borrow Checker Errors

```text
error[E0502]: cannot borrow `x` as mutable because it is also borrowed as immutable
```

**Fix**: Ensure immutable borrows are dropped before mutable borrows:

```rust
// WRONG
let mut x = 5;
let r1 = &x;
let r2 = &mut x; // ERROR: r1 still alive
println!("{r1}");

// RIGHT
let mut x = 5;
let r1 = &x;
println!("{r1}"); // last use of r1
let r2 = &mut x;  // OK: r1 no longer used
*r2 += 1;
```

### 2. Lifetime Errors

```text
error[E0515]: cannot return reference to local variable
```

**Fix**: return owned data. A `&'static` reference can't point at a local — locals are dropped when the function
returns. The real options:

- Return an owned value (`String`, `Vec`, ...) — almost always the right answer.
- Return a reference to a genuine `static` / `const` item.
- `Box::leak` to promote an owned value to `&'static mut` — permanently leaks memory until process exit; rarely appropriate.

```rust
// WRONG
fn bad() -> &str {
    let s = String::from("hello");
    &s // ERROR: s dropped at end of function
}

// RIGHT: return owned data
fn good() -> String {
    String::from("hello")
}

// RIGHT (when you truly need &'static): a real static
static GREETING: &str = "hello";
fn greeting() -> &'static str {
    GREETING
}
```

### 3. Trait Bound Errors

```text
error[E0277]: the trait bound `T: Clone` is not satisfied
```

**Fix**: Add trait bounds or use concrete types:

```rust
// WRONG
fn process<T>(item: T) -> T {
    item.clone() // ERROR: T might not be Clone
}

// RIGHT
fn process<T: Clone>(item: T) -> T {
    item.clone()
}
```

### 4. Type Mismatch

```text
error[E0308]: mismatched types
```

**Fix**: Check types match, use `into()`, `as`, or `From`:

```rust
let x: i32 = 5;
let y: i64 = x.into(); // OK
let z: i64 = x as i64; // OK
```

### 5. Mutating While Iterating (Iterator Invalidation)

In C++, pushing into a vector during iteration is a runtime iterator-invalidation bug. Rust turns it into a compile
error (E0502/E0499):

```rust
// Fragment — `vec: Vec<T>` where `T: Clone`.
// WRONG (does not compile)
for item in &vec {
    vec.push(item.clone()); // ERROR: can't borrow vec mutably while borrowed
}

// RIGHT: collect first, then extend
let new_items: Vec<_> = vec.iter().cloned().collect();
vec.extend(new_items);
```

## Runtime Debugging

### 1. println! Debugging

```rust
println!("DEBUG: value = {:?}", value);
println!("DEBUG: struct = {:#?}", my_struct); // pretty print
```

### 2. dbg! Macro

```rust
let x = 5;
let y = dbg!(x * 2); // Prints: [src/main.rs:2:5] x * 2 = 10
```

### 3. Logging with tracing

```rust
// Fragment — `db`, `User`, and the error type are placeholders for your code.
use tracing::{debug, info, instrument};

#[instrument]
async fn fetch_user(id: u64) -> Result<User> {
    debug!("Fetching user {id}");
    let user = db.fetch(id).await?;
    info!("Fetched user: {user:?}");
    Ok(user)
}
```

### 4. Debugger (GDB/LLDB)

```bash
# Build with debug info (default for the dev profile)
cargo build

# Run under GDB (rust-gdb ships with rustup and adds Rust pretty-printers)
rust-gdb target/debug/myapp
# or plain: gdb target/debug/myapp

# LLDB (rust-lldb also ships with rustup)
rust-lldb target/debug/myapp
```

### 5. VS Code Debugging

```json
// .vscode/launch.json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug",
            "program": "${workspaceFolder}/target/debug/myapp",
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

## Common Runtime Bugs and Fixes

### 1. Mutex Held Across `.await`

```rust
// Fragment — `std_mutex: std::sync::Mutex<T>` and `some_async()` are placeholders.
// WRONG: holding a std MutexGuard across .await
let guard = std_mutex.lock().unwrap();
some_async().await; // `guard` is !Send, so this future can't be spawned on the
                    // multi-threaded runtime (compile error). If it compiles
                    // (e.g. block_on / current_thread), other tasks needing the
                    // lock are starved while the holder is suspended — and a
                    // second lock attempt from the same task deadlocks it
                    // (std Mutex is not reentrant).
drop(guard);

// RIGHT: keep the critical section short — drop the guard BEFORE .await
let snapshot = {
    let guard = std_mutex.lock().unwrap(); // handle poisoning in real code
    (*guard).clone() // clone the inner value (T: Clone), not the guard
}; // guard dropped here
some_async().await;
println!("snapshot: {snapshot:?}");
```

This is usually **contention/starvation, not a true deadlock** — unless the same task re-locks (non-reentrant mutex)
or two tasks acquire two mutexes in opposite orders. If the guard genuinely must live across `.await`, use
`tokio::sync::Mutex` (its guard is `Send`) — but held-across-await guards serialize tasks, so contention becomes a
throughput bottleneck. See the rust-tokio skill.

### 2. Integer Overflow

```rust
// WRONG: Panics in debug, wraps in release
let x: u8 = 255;
let y = x + 1; // PANIC in debug!

// RIGHT: Use wrapping_add or checked_add
let y = x.wrapping_add(1); // 0
let y = x.checked_add(1); // None
```

### 3. Unwrap on None/Err

```rust
// Fragment — `some_option: Option<T>`, `some_result: Result<T, E>`, inside a
// function returning Result for the `?` line.
// WRONG: Panics
let x = some_option.unwrap();
let y = some_result.unwrap();

// RIGHT: Handle gracefully
let x = some_option.unwrap_or_default();
let y = some_result?; // propagate error
```

### 4. Async Blocking

```rust
// WRONG: Blocking call inside async — stalls the whole worker thread
async fn bad() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}

// RIGHT: Async sleep yields to the runtime
async fn good() {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}
```

## Debugging Tools

| Tool | Purpose |
| ------ | --------- |
| `dbg!` | Quick value inspection |
| `println!` | Simple logging |
| `tracing` | Structured logging |
| `gdb`/`lldb` | Step-through debugging |
| `cargo test` | Unit testing |
| `cargo clippy` | Linting |
| `cargo fmt` | Formatting |
| `cargo flamegraph` | Performance profiling |
| `cargo asm` | Assembly inspection |

`cargo asm` is provided by the cargo-show-asm plugin — `cargo install cargo-show-asm` first. `cargo flamegraph`
likewise needs `cargo install flamegraph` (plus `perf` on Linux or DTrace on macOS).

## Debugging Workflow

1. **Read the whole diagnostic** - rustc emits notes, secondary spans, and "help:" lines; the primary span often
   marks the *use* site, not the root cause (E0502 reports the second borrow, not the live `r1`).
2. **Understand the type** - use `dbg!`, `std::any::type_name`, or a deliberate type error to see what the compiler
   actually inferred; most "wrong lifetime" bugs are really unexpected derefs or clones.
3. **Isolate the question** - `cargo check` for compile errors, `cargo test` for behavior, `cargo build` for
   profile-specific issues; cut the repro down until one command flips the result.
4. **Fix and verify** - re-run the failing command first, then `cargo test` and clippy to catch the second-order
   breakage (a borrow fix often surfaces an unused-mut or dead-code warning).

## When to Use

- Compiler errors (borrow checker, lifetimes, traits)
- Runtime panics
- Contention and deadlocks in async code
- Performance issues
- Logic bugs
