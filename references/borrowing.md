# Borrowing in Rust

## Overview

Borrowing lets you access data without taking ownership. A borrow is a **reference** — a pointer to data owned by
someone else. The borrow checker enforces that references are always valid.

Two kinds of references:

- **Shared reference** `&T` — read-only, any number can coexist.
- **Mutable reference** `&mut T` — read-write, only one at a time, no shared references coexist.

---

## The Borrowing Rules

1. At any given time, you can have **either** one mutable reference **or** any number of immutable references.
2. References must always be **valid** — no dangling pointers.

These rules are enforced at compile time. Violations are compile errors, not runtime bugs.

---

## Shared References (`&T`)

```rust
fn print_len(s: &String) {
    println!("{}", s.len());
}

let s = String::from("hello");
print_len(&s); // borrow
println!("{s}"); // still valid — borrow ended
```

Multiple shared references can coexist:

```rust
let s = String::from("hello");
let r1 = &s;
let r2 = &s;
let r3 = &s;
println!("{r1} {r2} {r3}"); // fine
```

---

## Mutable References (`&mut T`)

```rust
fn append_world(s: &mut String) {
    s.push_str(" world");
}

let mut s = String::from("hello");
append_world(&mut s);
println!("{s}"); // "hello world"
```

Only one mutable reference at a time:

```rust
let mut s = String::from("hello");
let r1 = &mut s;
// let r2 = &mut s; // ERROR: cannot borrow `s` as mutable more than once
r1.push_str("!");
```

Mutable and shared references cannot coexist:

```rust
let mut s = String::from("hello");
let r1 = &s;
// let r2 = &mut s; // ERROR: cannot borrow as mutable because also borrowed as immutable
println!("{r1}");
```

---

## Lifetimes

Lifetimes are the borrow checker's way of tracking how long references are valid. Most of the time, lifetimes are
**elided** (inferred).

### Explicit lifetimes

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

The `'a` lifetime is the **intersection** of the two input lifetimes: the returned reference is valid only as long as
the **shorter-lived** of the two inputs. The caller cannot use the result beyond the point where *either* input expires.

### Lifetime elision rules

The compiler infers lifetimes in three cases:

1. **Each reference parameter gets its own lifetime.**
2. **If there's exactly one input lifetime, it's assigned to all output lifetimes.**
3. **If there's `&self` or `&mut self`, the lifetime of `self` is assigned to all output lifetimes.**

```rust
// Elided:
fn first_word(s: &str) -> &str { /* ... */ }

// Desugared:
fn first_word<'a>(s: &'a str) -> &'a str { /* ... */ }
```

### When you need explicit lifetimes

```rust
// Multiple input lifetimes — compiler can't infer which one the output uses
struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    fn parse(&self) -> &'a str {
        &self.input[..10]
    }
}
```

### `'static` lifetime

`'static` means the reference is valid for the entire program duration. String literals are `'static`:

```rust
let s: &'static str = "hello"; // stored in the binary's read-only data
```

**Pitfall:** `'static` as a **bound** (`T: 'static`) does not mean "owns its data" — it means the type contains **no
borrowed references with lifetimes shorter than `'static`**. Owning all your data satisfies this, but so does borrowing
`'static` data: `&'static str` itself implements `T: 'static`, while `&'a str` for any shorter `'a` does not. This is
different from *being* a `'static` reference — `String` satisfies `String: 'static` without any reference at all.

---

## Common Borrow Checker Errors and Fixes

### Error 1: Cannot borrow as mutable because also borrowed as immutable

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];
v.push(4); // ERROR: cannot borrow `v` as mutable
println!("{first}");
```

**Fix:** Narrow the scope of the immutable borrow:

```rust
let mut v = vec![1, 2, 3];
let first = v[0].clone(); // copy the value
v.push(4);
println!("{first}");
```

Or use a block to end the borrow:

```rust
let mut v = vec![1, 2, 3];
let first = {
    let r = &v[0];
    *r
};
v.push(4);
```

### Error 2: Cannot return reference to local variable

```rust
fn bad() -> &String {
    let s = String::from("temp");
    &s
}
```

**Fix:** Return an owned value:

```rust
fn good() -> String {
    String::from("temp")
}
```

### Error 3: Calling a `&mut self` method while a field is immutably borrowed

```rust
struct Builder {
    parts: Vec<String>,
}

impl Builder {
    fn add(&mut self, s: &str) {
        self.parts.push(s.to_string());
    }
}

let mut b = Builder { parts: vec![] };
b.add("a");
b.add("b"); // fine — each call's &mut borrow ends before the next
```

The error occurs when you hold a reference to a field across calls:

```rust
let part = &b.parts[0];
b.add("c"); // ERROR: `b` is borrowed as immutable
```

**Fix:** Clone or restructure.

### Error 4: Multiple mutable borrows in a loop

```rust
let mut data = vec![1, 2, 3];
for item in &mut data {
    data.push(*item); // ERROR: cannot borrow `data` as mutable
}
```

**Fix:** Collect changes and apply after the loop:

```rust
let mut data = vec![1, 2, 3];
let to_add: Vec<_> = data.iter().map(|x| x * 2).collect();
data.extend(to_add);
```

### Error 5: Holding a `RefCell` borrow across an `await`

Borrowing ordinary references across `.await` is fine (the future just carries
them). The real pitfall is holding a **runtime-checked borrow** — a
`Ref<'_, T>` from `RefCell` — across an await: `Ref` is not `Send`, so the future
becomes `!Send` and `tokio::spawn` rejects it:

```rust
use std::cell::RefCell;

async fn length_later(cell: &RefCell<Vec<u8>>) -> usize {
    let borrow = cell.borrow();   // `borrow: Ref<'_, Vec<u8>>` — not Send
    some_async_op().await;        // held across .await -> future is !Send
    borrow.len()
}

// tokio::spawn(length_later(&cell));
// ERROR: future cannot be sent between threads safely
```

**Fix:** end the borrow before the `.await`:

```rust
async fn length_later(cell: &RefCell<Vec<u8>>) -> usize {
    let len = cell.borrow().len(); // borrow ends at the semicolon
    some_async_op().await;
    len
}
```

(There's a second hazard too: even without threads, a `RefCell` borrow held across
`.await` panics if the task that runs meanwhile borrows the same cell mutably.)

---

## Interior Mutability

Interior mutability lets you mutate data through a shared (`&T`) reference. The cell types move borrow checking from
compile time to runtime:

| Type | Mutation model | Checking |
| ------ | ---------------- | ---------- |
| `Cell<T>` | Copy in/out — `get()`/`set()`/`replace()`, no references into the cell | None needed; values move by copy |
| `RefCell<T>` | `borrow()`/`borrow_mut()` hand out tracked `Ref`/`RefMut` | Runtime — **panics** on violation |
| `OnceCell<T>` / `OnceLock<T>` | Write exactly once via `set()`, then read freely | `set()` returns `Err` on a second write |
| `LazyLock<T>` | Initialized on first deref from a closure | Thread-safe, `static`-compatible |

### `Cell<T>` — small `Copy` types

No borrow checking is needed because values go in and out by copy:

```rust
use std::cell::Cell;

struct RequestMetrics {
    served: Cell<u64>,
}

impl RequestMetrics {
    fn record(&self) {
        self.served.set(self.served.get() + 1); // &self suffices
    }
}
```

### `RefCell<T>` — single-threaded, runtime-checked

```rust
use std::cell::RefCell;

struct Cache {
    data: RefCell<Vec<u8>>,
}

impl Cache {
    fn get(&self) -> usize {
        self.data.borrow().len()
    }
    fn push(&self, val: u8) {
        self.data.borrow_mut().push(val);
    }
}
```

`borrow()`/`borrow_mut()` **panic** if the rules are violated at runtime (e.g., a mutable borrow while a shared one is
live). Use `try_borrow()`/`try_borrow_mut()` to get a `Result` instead of a panic. Keep borrows short — never hold a
`Ref`/`RefMut` across `.await` (see Error 5).

### `OnceCell` / `OnceLock` — write-once

`OnceCell<T>` is single-threaded; `OnceLock<T>` (std 1.70+) is thread-safe. Ideal for values computed once and read many
times:

```rust
use std::sync::OnceLock;

fn config_path() -> &'static str {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| "/etc/myapp/config.toml".to_string())
}
```

### `LazyLock<T>` — lazy `static`s (std 1.80+)

```rust
use std::sync::LazyLock;

static CONFIG: LazyLock<Config> = LazyLock::new(Config::load);
// First deref runs Config::load exactly once, synchronized across threads.
```

`LazyLock` covers most former uses of the `lazy_static` and `once_cell` crates.

### When to use what

- Simple counters/flags in one thread → `Cell`.
- Shared mutable state in one thread → `RefCell`.
- Computed-once value → `OnceCell` (single-threaded) / `OnceLock` (multi-threaded).
- Global/lazy statics → `LazyLock`.
- Mutable state across threads → `Mutex`/`RwLock`/atomics instead — the cell types are not `Sync`.

---

## NLL (Non-Lexical Lifetimes)

Since Rust 2018, the borrow checker uses NLL — borrows end at the **last use**, not at the end of the scope.

```rust
let mut s = String::from("hello");
let r = &s;
println!("{r}"); // last use of `r`
// borrow ends here, not at end of scope
s.push_str(" world"); // fine — `r` is no longer used
```

This makes many patterns work that would have failed under the older lexical lifetime rules.

---

## Summary

| Pattern | Rule | When to Use |
| --------- | ------ | ------------- |
| `&T` | Any number, read-only | Default choice |
| `&mut T` | Exactly one, no `&T` coexisting | When you need to mutate |
| `&'a T` | Lifetime must outlive usage | Structs holding references |
| `Cell<T>` | Copy in/out, no borrow checking | Small `Copy` values, single-threaded |
| `RefCell<T>` | Runtime borrow checking | Interior mutability, single-threaded |
| `OnceLock<T>` / `LazyLock<T>` | Write-once / lazy init | Computed-once values, global statics |
| `Mutex<T>` | Runtime borrow checking | Interior mutability, multi-threaded |
