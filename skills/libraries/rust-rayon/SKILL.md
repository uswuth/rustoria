---
name: rust-rayon
description: Use when adding data parallelism to Rust with rayon.
---

# Rayon — Data Parallelism for Rust

## Overview

Rayon is a data-parallelism library that makes it easy to convert sequential computations into parallel ones. It
guarantees data-race freedom and dynamically adapts for maximum performance.

## Installation

```toml
[dependencies]
rayon = "1.12"
```

## Core Concepts

### 1. Parallel Iterators

The simplest way to use Rayon — just change `.iter()` to `.par_iter()`:

```rust
use rayon::prelude::*;

fn sum_of_squares(input: &[i32]) -> i32 {
    input.par_iter()
        .map(|&i| i * i)
        .sum()
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let sum = sum_of_squares(&numbers);
    println!("Sum of squares: {}", sum);
}
```

### 2. Parallel Operations

```rust
// Fragment — statements assume a surrounding `fn main()`.
use rayon::prelude::*;

let numbers = vec![1, 2, 3, 4, 5];

// Parallel map
let squares: Vec<i32> = numbers.par_iter()
    .map(|&x| x * x)
    .collect();

// Parallel filter
let evens: Vec<&i32> = numbers.par_iter()
    .filter(|&&x| x % 2 == 0)
    .collect();

// Parallel for_each
numbers.par_iter()
    .for_each(|x| println!("{}", x));

// Parallel reduce - needs an identity element plus an associative op; items are
// &i32, so copy them out first (identity and op must yield the item type)
let sum = numbers.par_iter()
    .copied()
    .reduce(|| 0, |a, b| a + b);

// Parallel fold + reduce — same identity requirement on reduce
let sum = numbers.par_iter()
    .fold(|| 0, |a, &b| a + b)
    .reduce(|| 0, |a, b| a + b);

// No identity element? Use reduce_with — returns Option (None on empty input)
let max: Option<i32> = numbers.par_iter()
    .copied()
    .reduce_with(|a, b| a.max(b));
```

### 3. Parallel Sort

```rust
// Fragment — statements assume a surrounding `fn main()`.
use rayon::prelude::*;

let mut numbers = vec![5, 2, 8, 1, 9, 3];
numbers.par_sort();
// or
numbers.par_sort_by(|a, b| b.cmp(a)); // descending
```

### 4. Custom Tasks with Join

A teaching example of divide-and-conquer with `join` — for real sorting use `par_sort` (§3):

```rust
use rayon::join;

fn partition<T: Ord>(slice: &mut [T]) -> usize {
    let pivot = slice.len() - 1;
    let mut i = 0;
    for j in 0..pivot {
        if slice[j] <= slice[pivot] {
            slice.swap(i, j);
            i += 1;
        }
    }
    slice.swap(i, pivot);
    i
}

fn quicksort<T: Ord + Send>(slice: &mut [T]) {
    if slice.len() <= 1 {
        return;
    }
    let pivot = partition(slice);
    let (left, right) = slice.split_at_mut(pivot);
    join(
        || quicksort(left),
        || quicksort(&mut right[1..]), // right[0] is the pivot, already placed
    );
}

fn main() {
    let mut data = vec![5, 2, 8, 1, 9, 3];
    quicksort(&mut data);
    assert_eq!(data, [1, 2, 3, 5, 8, 9]);
}
```

### 5. Scopes

```rust
// Fragment — statements assume a surrounding `fn main()`.
use rayon::scope;

scope(|s| {
    s.spawn(|_| {
        println!("Running in parallel");
    });
    s.spawn(|_| {
        println!("Also running in parallel");
    });
});
// Blocks until all spawned tasks complete
```

### 6. Custom Thread Pools

```rust
// Fragment — statements assume a surrounding `fn main()`.
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

let pool = ThreadPoolBuilder::new()
    .num_threads(4)
    .build()
    .unwrap();

pool.install(|| {
    // This closure runs on the custom pool
    let sum = (0..1000).into_par_iter().sum::<i32>();
    println!("Sum: {}", sum);
});
```

### 7. Parallel Collections

```rust
// Fragment — statements assume a surrounding `fn main()`.
use rayon::prelude::*;

// Parallel extend
let mut vec = Vec::new();
vec.par_extend((0..1000).into_par_iter());

// Parallel slice operations
let slice: &[i32] = &[1, 2, 3, 4, 5];
let sum = slice.par_iter().sum::<i32>();

// Parallel string search — &str has no par_iter; use the ParallelString methods
let text = "hello world";
let found = text.par_chars().any(|c| c == 'w');
let has_long_word = text.par_split(' ').any(|w| w.len() > 4);
```

### 8. Performance Considerations

Parallelism has per-task overhead (work stealing, synchronization, allocations in `collect`). For tiny inputs or trivial
per-element work, the sequential version is faster:

```rust
// Fragment — the loose statements assume a surrounding `fn main()`.
use rayon::prelude::*;

let small = vec![1, 2, 3];
let sum: i32 = small.iter().sum(); // sequential: cheaper here

// There is no universal size threshold. Break-even depends on per-element
// cost, cache behavior, and core count — the 10_000 below is a starting
// guess, not a rule. MEASURE your real workload (benchmark with criterion;
// see the testing skill) and pick the crossover from data.
fn total(numbers: &[i32]) -> i32 {
    if numbers.len() < 10_000 {
        numbers.iter().sum()
    } else {
        numbers.par_iter().sum()
    }
}
```

Also: use `par_iter()` for **CPU-bound** work only. For I/O-bound concurrency, use an async runtime (tokio) instead —
rayon threads block on I/O just like any other thread.

### 9. Best Practices

1. **Use `par_iter()` for CPU-bound work** — not I/O
2. **Avoid side effects** in parallel closures (order is non-deterministic)
3. **`reduce` needs an identity element** — use `reduce_with` (returns `Option`) when no identity exists
4. **Use `par_sort`** for large datasets
5. **Use `scope`** for tasks that need to borrow stack data
6. **Use `ThreadPoolBuilder`** for custom thread pools
7. **Measure before parallelizing** — overhead dominates on small/cheap workloads; benchmark to find your break-even point
8. **Combine with `Arc`** for shared read-only data

## When to Use Rayon

- CPU-bound parallel computation
- Data processing pipelines
- Parallel sorting
- Image processing
- Scientific computing
- Any workload that can be expressed as parallel iterators
