# Performance Engineering in Rust

## Overview

Rust provides zero-cost abstractions, but performance still requires understanding allocation patterns, cache effects,
and measurement. The golden rule: **measure first, optimize second**.

---

## Allocations

### Stack vs heap

Stack allocation is nearly free (just a pointer bump). Heap allocation involves the allocator and potential system calls.

```rust
// Stack — fast
let x: [u8; 64] = [0; 64];

// Heap — slower
let y: Box<[u8; 64]> = Box::new([0; 64]);
let z: Vec<u8> = vec![0; 64];
```

### Reducing allocations

```rust
// Bad: allocates a new String on every call
fn bad(parts: &[String]) -> String {
    let mut result = String::new();
    for part in parts {
        result.push_str(part);
        result.push(' ');
    }
    result
}

// Good: pre-allocate with capacity
fn good(parts: &[String]) -> String {
    let total_len: usize = parts.iter().map(|s| s.len() + 1).sum();
    let mut result = String::with_capacity(total_len);
    for part in parts {
        result.push_str(part);
        result.push(' ');
    }
    result
}
```

### `SmallVec` and `ArrayVec`

For small, fixed-size collections, avoid heap allocation entirely:

```rust
use smallvec::SmallVec;

// Inline storage for up to 4 elements, then heap
let mut v: SmallVec<[u8; 4]> = SmallVec::new();
v.push(1); // no allocation
v.push(2);
v.push(3);
v.push(4);
v.push(5); // now heap-allocated
```

### `bumpalo` — arena allocation

For short-lived objects with the same lifetime, arena allocation is much faster than individual allocations:

```rust
use bumpalo::Bump;

let bump = Bump::new();
let mut v = bumpalo::vec![in &bump; 1, 2, 3];
// All allocations come from the arena — freed in one shot on `bump.reset()`
// or when `bump` is dropped. Values in the arena can't outlive `bump`.
```

---

## Clones

### Avoiding unnecessary clones

```rust
// Bad: clones the whole String just to read from it
fn bad(s: &str) -> usize {
    let owned = s.to_string(); // unnecessary allocation + copy
    owned.to_uppercase().len()
}

// Good: borrow the input; allocate only the uppercase result
fn good(s: &str) -> usize {
    s.to_uppercase().len() // to_uppercase itself allocates, the clone doesn't happen
}
```

### `Rc`/`Arc` for cheap cloning

```rust
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3, 4, 5]);
let ref2 = Rc::clone(&data); // just increments refcount — O(1)
let ref3 = Rc::clone(&data);
```

### `clone_from` for reusing allocations

```rust
let mut dst = String::with_capacity(100);
let src = String::from("hello world");

dst.clone_from(&src); // reuses dst's allocation if src.len() <= dst.capacity()
```

---

## `Cow` (Clone on Write)

`Cow<'a, T>` is an enum that holds either a borrowed or owned value. It avoids cloning until mutation is needed.

```rust
use std::borrow::Cow;

fn process(input: &str) -> Cow<'_, str> {
    if input.contains("bad") {
        // Only allocate if we need to modify
        Cow::Owned(input.replace("bad", "good"))
    } else {
        // Zero-cost: just borrow
        Cow::Borrowed(input)
    }
}
```

### When to use `Cow`

- Functions that usually return borrowed data but sometimes need to own.
- Configuration values that are usually static but can be overridden.
- Avoiding clones in read-heavy paths.

### When NOT to use `Cow`

- When you always need to own the value.
- When the borrowed and owned types have different APIs.
- When the overhead of the enum tag matters (rare).

---

## Iterator Optimization

### Lazy evaluation

Iterators are lazy — they don't do anything until consumed:

```rust
// Bad: collect builds an intermediate Vec we immediately walk again
let first_even = data.iter()
    .map(|x| x * 2)
    .collect::<Vec<_>>()
    .into_iter()
    .find(|x| *x % 4 == 0);

// Good: the lazy chain allocates nothing and stops at the first match
let first_even = data.iter()
    .map(|x| x * 2)
    .find(|x| *x % 4 == 0);
```

Consumers differ in how much they pull: `collect` always drains the whole
iterator; `find`, `any`, `fold`, and `sum` pull only what they need (`find` and
`any` short-circuit). There is no allocation difference between `fold` and `sum`
— pick whichever reads best; the waste to avoid is the *intermediate collection*.

### `Iterator::size_hint` and `ExactSizeIterator`

```rust
// Pre-allocate when size is known
let v: Vec<_> = (0..100).collect();
let mapped: Vec<_> = v.iter().map(|x| x * 2).collect();
// collect() uses the iterator's size_hint to reserve capacity up front
```

When writing your own adapters or collecting from an iterator with an unreliable
`size_hint`, use `Vec::with_capacity(n)` yourself — each `push` past capacity
grows the buffer (typically doubling), so a known size turns ~log₂(n)
reallocations-and-copies into zero.

### `itertools` for advanced iterators

```rust
use itertools::Itertools;

// Chunked iteration
for chunk in &data.iter().chunks(100) {
    process_chunk(chunk);
}

// Cartesian product
for (a, b) in iproduct!(&list_a, &list_b) {
    process(a, b);
}
```

---

## Profiling with `cargo flamegraph`

### Installation

```bash
cargo install flamegraph
```

**Platform note:** `cargo flamegraph` samples via `perf` (Linux) or DTrace
(macOS). It does not work natively on Windows — use WSL2, or a Windows-native
profiler (e.g., Visual Studio's profiler or `wpa`/`wpr`).

### Usage

```bash
# Profile a binary
cargo flamegraph --bin my-app

# Profile a specific test
cargo flamegraph --test integration_tests -- test_name

# Profile with custom options
cargo flamegraph --freq 999 --bin my-app
```

### Reading flamegraphs

- **X-axis:** sample count (not time).
- **Y-axis:** call stack depth.
- **Width:** proportion of samples in that function.
- **Leaf functions:** the actual hot spots.

### What to look for

- Wide leaf functions — these consume the most CPU.
- Unexpected function calls — allocations, clones, locks.
- Deep call stacks — potential for optimization.

---

## Benchmarking with `criterion`

### Setup

```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.8", features = ["html_reports"] }

[[bench]]
name = "my_benchmark"
harness = false
```

### Writing benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn bench_fibonacci(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| fibonacci(black_box(20)))
    });
}

criterion_group!(benches, bench_fibonacci);
criterion_main!(benches);
```

### Running benchmarks

```bash
# Run all benchmarks
cargo bench

# Run a specific benchmark
cargo bench fib_20

# Compare against a baseline
cargo bench --baseline main
```

### Best practices

- Use `black_box` to prevent the compiler from optimizing away your code. Since
  Rust 1.66, `std::hint::black_box` is built in — criterion re-exports it, so no
  extra import is needed inside criterion benches.
- Benchmark realistic workloads, not microbenchmarks.
- Run multiple times and look at the distribution, not just the mean.
- Use `criterion_group!` to organize related benchmarks.

---

## Release Profiles

The release profile controls how `cargo build --release` optimizes. Defaults are
a good start; tune when binary size or latency matters:

```toml
# Cargo.toml
[profile.release]
opt-level = 3        # 0–3 speed, "s"/"z" optimize for size
lto = "thin"         # "off" (default), "thin" (good default upgrade), "fat" (max, slow builds)
codegen-units = 16   # default; 1 = best optimization, slowest compile
panic = "unwind"     # "abort": smaller/faster, but no catch_unwind
strip = false        # true / "symbols": smaller binary, no symbol names in backtraces
```

For iteration speed, the dev profile can be tweaked too — a common trick is
optimizing dependencies but not your own crate:

```toml
[profile.dev.package."*"]
opt-level = 2
```

See `toolchain.md` for the full profile reference.

---

## Memory Layout

### Struct field ordering

With the default `repr(Rust)`, the compiler is already free to reorder fields to
minimize padding — handwritten "bad" and "good" orderings usually produce the
same layout. Manual ordering matters in two cases: `#[repr(C)]` (fields stay in
declaration order, padding included) and cache-line layout.

```rust
// With repr(C), declaration order is preserved — this is 24 bytes:
#[repr(C)]
struct Bad {
    a: u8,  // 1 byte + 7 padding
    b: u64, // 8 bytes
    c: u8,  // 1 byte + 7 padding
}

// Same fields, ordered by alignment — 16 bytes:
#[repr(C)]
struct Good {
    b: u64, // 8 bytes
    a: u8,  // 1 byte
    c: u8,  // 1 byte (+ 6 padding so the struct aligns to 8)
}
```

Check actual sizes with `std::mem::size_of::<T>()`. For hot loops, group fields
accessed together onto the same cache line (typically 64 bytes), and keep fields
written by different threads on separate lines (false sharing).

### `#[repr(C)]` and `#[repr(packed)]`

```rust
#[repr(C)] // C-compatible layout
struct FFI {
    x: u32,
    y: u64,
}

#[repr(packed)] // no padding — use with caution
struct Packed {
    x: u8,
    y: u32, // unaligned — slower access
}
```

---

## Cache-Friendly Code

### Data-oriented design

```rust
// Bad: array of structs — cache unfriendly
struct Particle { x: f64, y: f64, vx: f64, vy: f64, mass: f64 }
let particles: Vec<Particle> = Vec::new();

// Good: struct of arrays — cache friendly
struct Particles {
    x: Vec<f64>,
    y: Vec<f64>,
    vx: Vec<f64>,
    vy: Vec<f64>,
    mass: Vec<f64>,
}
```

### Iterating in order

```rust
// Good: sequential memory access
for i in 0..n {
    process(data[i]);
}

// Bad: random memory access
for i in random_indices {
    process(data[i]);
}
```

---

## Summary

| Technique | When to Use | Impact |
| ----------- | ------------- | -------- |
| Pre-allocation | Known capacity | Avoids reallocations |
| `SmallVec` | Small fixed-size collections | Avoids heap allocation |
| `Cow` | Borrow-or-own patterns | Avoids clones |
| `Rc`/`Arc` | Shared ownership | O(1) clone |
| `bumpalo` | Short-lived objects | Fast arena allocation |
| `black_box` | Benchmarks | Prevents optimization |
| `criterion` | Benchmarking | Statistical rigor |
| `flamegraph` | Profiling | Visual hot spots |
| Struct field ordering | Memory-constrained | Reduces padding |
| Data-oriented design | CPU-bound loops | Cache efficiency |
