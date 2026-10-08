# Concurrency Patterns in Rust

## Overview

Rust's ownership system prevents data races at compile time. The `Send` and `Sync` traits are the foundation: `Send`
means a type can be moved to another thread, `Sync` means a type can be shared between threads.

---

## Threads

### Spawning threads

```rust
use std::thread;

let handle = thread::spawn(|| {
    println!("running on a new thread");
    42
});

let result = handle.join().unwrap();
println!("result: {result}");
```

### Thread with data

```rust
use std::thread;
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);
let mut handles = vec![];

for i in 0..3 {
    let data = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        println!("thread {i}: {:?}", data);
    }));
}

for handle in handles {
    handle.join().unwrap();
}
```

### Scoped threads

Scoped threads can borrow data from the parent thread (no `Arc` needed):

```rust
use std::thread;

let data = vec![1, 2, 3];

thread::scope(|s| {
    for item in &data {
        s.spawn(|| {
            println!("{item}");
        });
    }
}); // all threads joined here
```

---

## Channels

### `std::sync::mpsc` — multi-producer, single-consumer

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    tx.send("hello").unwrap();
});

let msg = rx.recv().unwrap();
println!("{msg}");
```

### `tokio::sync::mpsc` — async channels

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);

tokio::spawn(async move {
    tx.send(42).await.unwrap();
});

let val = rx.recv().await.unwrap();
```

### `tokio::sync::broadcast` — one-to-many

```rust
use tokio::sync::broadcast;

let (tx, _rx) = broadcast::channel(16);
let rx2 = tx.subscribe();

tx.send("hello").unwrap();
```

### `tokio::sync::watch` — single value, always current

```rust
use tokio::sync::watch;

let (tx, rx) = watch::channel("initial");

tx.send("updated").unwrap();
assert_eq!(*rx.borrow(), "updated");
```

---

## `Mutex` vs `RwLock`

### `std::sync::Mutex`

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        let mut val = counter.lock().unwrap();
        *val += 1;
    }));
}
```

### `std::sync::RwLock`

```rust
use std::sync::{Arc, RwLock};

let data = Arc::new(RwLock::new(vec![1, 2, 3]));

// Multiple readers
let read_handle = {
    let data = Arc::clone(&data);
    thread::spawn(move || {
        let val = data.read().unwrap();
        println!("read: {val:?}");
    })
};

// Exclusive writer
let write_handle = {
    let data = Arc::clone(&data);
    thread::spawn(move || {
        let mut val = data.write().unwrap();
        val.push(4);
    })
};
```

### When to use which

| Type | Use When | Contention |
| ------ | ---------- | ------------ |
| `Mutex` | Most cases, short critical sections | Serializes all access |
| `RwLock` | Read-heavy workloads, long read operations | Allows concurrent reads |
| `Atomic*` | Simple counters, flags | Lock-free, fastest |

**Pitfall:** `RwLock` can cause writer starvation if readers keep acquiring the lock. Prefer `Mutex` unless you have a
proven read-heavy workload.

### `std::sync::Condvar` — waiting for a condition

A `Condvar` lets threads sleep until a condition becomes true. It is always paired
with a `Mutex` that protects the condition itself:

```rust
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

let pair = Arc::new((Mutex::new(false), Condvar::new()));
let pair2 = Arc::clone(&pair);

thread::spawn(move || {
    let (lock, cvar) = &*pair2;
    let mut ready = lock.lock().unwrap();
    *ready = true;
    cvar.notify_one(); // sent after the state changed, so no waiter can miss it
});

let (lock, cvar) = &*pair;
let mut ready = lock.lock().unwrap();
while !*ready {
    // wait() atomically releases the lock and sleeps; returns a new guard on wake
    ready = cvar.wait(ready).unwrap();
}
```

**Pitfall — lost notification:** `notify_one()` is not sticky — if no thread is
waiting when it fires, it is lost. That is why the pattern is: the waiter checks
the condition *under the lock* before sleeping (so a notification can't slip
between the check and the sleep), and re-checks in a loop after every wake —
spurious wakeups are allowed.

---

## Atomics

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

let counter = Arc::new(AtomicUsize::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        counter.fetch_add(1, Ordering::Relaxed);
    }));
}

for handle in handles {
    handle.join().unwrap();
}

println!("count: {}", counter.load(Ordering::Relaxed));
```

### Memory orderings

| Ordering | Guarantees |
| ---------- | ------------ |
| `Relaxed` | Atomicity only — no ordering relative to other memory operations |
| `Acquire` | Reads/writes after it can't be reordered before it (pairs with a `Release` store) |
| `Release` | Reads/writes before it can't be reordered after it (pairs with an `Acquire` load) |
| `AcqRel` | Both, for read-modify-write ops like `fetch_add` |
| `SeqCst` | Plus a single global order all `SeqCst` operations agree on |

**Rule of thumb:** default to `SeqCst`. It's the strongest and easiest to reason
about, and the cost is rarely measurable until profiling says otherwise. Weaken
deliberately, not by default:

| Situation | Ordering |
| ----------- | ---------- |
| Unsure / default | `SeqCst` |
| Publishing data: writer stores data, then sets a flag; reader loads the flag, then reads the data | flag store = `Release`, flag load = `Acquire` |
| Read-modify-write inside a publish pattern (`fetch_add`, `compare_exchange`) | `AcqRel` |
| Independent counter/statistic where only the final value matters and no other data's ordering depends on it | `Relaxed` |

The counter example above is a legitimate `Relaxed` use: the counter itself is
atomic, and nothing else in the program is ordered by it. `Relaxed` is wrong the
moment a reader must also see *other* data the writer published — that requires
`Acquire`/`Release` pairing.

---

## `Send` and `Sync`

### Auto-derived traits

Most types are automatically `Send` and `Sync` if their fields are. You rarely need to implement them manually.

### Types that are NOT `Send`/`Sync`

| Type | `Send`? | `Sync`? | Why |
| ------ | --------- | --------- | ----- |
| `Rc<T>` | No | No | Non-atomic refcount — increments would race across threads |
| `Cell<T>` | Yes if `T: Send` | No | Mutation through `&T` isn't synchronized |
| `RefCell<T>` | Yes if `T: Send` | No | Runtime borrow flags aren't atomic — two threads borrowing concurrently would race |
| `*const T`, `*mut T` | No | No | Raw pointers — no aliasing or lifetime guarantees |

Note the asymmetry: `Cell`/`RefCell` **are** `Send` when their contents are — you
can *move* one to another thread — but they are never `Sync`, so two threads can't
*share* one through references.

### Wrapping non-thread-safe types

```rust
// Rc -> Arc
let shared = Arc::new(data);

// RefCell -> Mutex or RwLock
let shared = Arc::new(Mutex::new(data));

// Rc<RefCell<T>> -> Arc<Mutex<T>>
let shared = Arc::new(Mutex::new(data));
```

### Manual `unsafe` implementation

Only do this if you know exactly what you're doing:

```rust
struct MyUnsafeType {
    ptr: *mut u8,
}

unsafe impl Send for MyUnsafeType {}
unsafe impl Sync for MyUnsafeType {}
```

---

## Common Concurrency Bugs

### 1. Data race (prevented at compile time)

You can't write a naive data race in safe Rust — the compiler rejects it. The
error you get is a **lifetime** error, because `thread::spawn` requires the
closure (and everything it captures) to be `'static`:

```rust
let mut data = vec![1, 2, 3];
thread::spawn(|| {
    data.push(4); // ERROR: closure may outlive the current function,
                  // but it borrows `data`, which is owned by the current function
});
```

This is race prevention at work: to make it compile you must pick a sound sharing
strategy —

```rust
// Fix 1: move ownership into the thread — only the child can touch the data,
// so no two threads ever access it concurrently
let mut data = vec![1, 2, 3];
thread::spawn(move || {
    data.push(4);
}).join().unwrap();
// `data` is moved out of this frame — the parent can no longer use it

// Fix 2: borrow with a scoped thread — the scope joins all its threads
// before returning, so the borrow can't outlive the frame
let mut data = vec![1, 2, 3];
thread::scope(|s| {
    s.spawn(|| data.push(4));
});
assert_eq!(data.len(), 4);

// Fix 3: shared ownership + mutual exclusion
let data = Arc::new(Mutex::new(vec![1, 2, 3]));
let d = Arc::clone(&data);
thread::spawn(move || d.lock().unwrap().push(4)).join().unwrap();
```

### 2. Deadlock from inconsistent lock ordering

```rust
// Bad: two threads calling transfer with swapped arguments deadlock
// (thread 1 holds a, waits for b; thread 2 holds b, waits for a)
fn transfer(a: &Mutex<u64>, b: &Mutex<u64>, amount: u64) {
    let mut a = a.lock().unwrap();
    let mut b = b.lock().unwrap();
    *a -= amount;
    *b += amount;
}

// Good: lock in a globally consistent order (here: by address)
fn transfer(a: &Mutex<u64>, b: &Mutex<u64>, amount: u64) {
    if std::ptr::eq(a, b) {
        return; // same mutex: locking it twice would self-deadlock
    }
    let (mut ga, mut gb) = if std::ptr::from_ref(a) < std::ptr::from_ref(b) {
        (a.lock().unwrap(), b.lock().unwrap())
    } else {
        let gb = b.lock().unwrap();
        let ga = a.lock().unwrap();
        (ga, gb) // lock in address order, but bind each guard to the right name
    };
    *ga -= amount;
    *gb += amount;
}
```

**Caveat:** address-ordered locking only works if every code path that takes these
locks uses the *same* ordering rule — and you must guard against `a` and `b` being
the same mutex, as above.

### 3. Holding a lock across `.await`

Same rule as in `async.md` (pitfall 2), because it bites in both directions:

- `std::sync::MutexGuard` is `!Send` → holding it across `.await` makes the future
  `!Send`; `tokio::spawn` **rejects it at compile time**. Even unspawned, the OS
  thread keeps the lock while the task is suspended.
- `tokio::sync::MutexGuard` is `Send` → it **compiles**, but it serializes every
  task wanting that lock behind your await, and deadlocks if the awaited code
  needs the same lock.

Drop the guard before `.await`, whichever mutex you use:

```rust
// Compiles, but bad: tokio guard held across await — other tasks stall
async fn bad(mutex: &tokio::sync::Mutex<Vec<u8>>) {
    let mut guard = mutex.lock().await;
    guard.push(1);
    some_async_op().await; // lock still held here
}

// Good: scope the guard so it is dropped before the await
async fn good(mutex: &tokio::sync::Mutex<Vec<u8>>) {
    {
        let mut guard = mutex.lock().await;
        guard.push(1);
    }
    some_async_op().await;
}
```

### 4. `std::sync::Mutex` in async code

```rust
// Compile error once spawned: the guard is !Send, so the future is !Send
async fn bad(mutex: &std::sync::Mutex<u64>) {
    let mut guard = mutex.lock().unwrap();
    some_async_op().await;
    *guard += 1;
}
// tokio::spawn(bad(&mutex)); // ERROR: future cannot be sent between threads safely

// Good: scope the guard before the await (or, if you truly must hold a lock
// across .await, use tokio::sync::Mutex — but prefer restructuring)
async fn good(mutex: &std::sync::Mutex<u64>) {
    {
        let mut guard = mutex.lock().unwrap();
        *guard += 1;
    }
    some_async_op().await;
}
```

### 5. Mutex poisoning after a panic

`Mutex::lock()` returns `LockResult<MutexGuard>`. If a thread panics **while
holding the lock**, the mutex is *poisoned*, and every later `lock()` returns
`Err(PoisonError)` — so the common `.unwrap()` propagates the failure to all
other users of that mutex:

```rust
let mutex = Arc::new(Mutex::new(0u64));

let m = Arc::clone(&mutex);
thread::spawn(move || {
    let mut guard = m.lock().unwrap();
    *guard += 1;
    panic!("boom"); // guard dropped during unwind -> mutex is poisoned
})
.join()
.unwrap_err();

// Default policy: fail fast
assert!(mutex.lock().is_err());

// Explicit recovery: take the guard anyway, re-validate or reset the state
let mut guard = mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
*guard = 0; // known-good state before continuing
```

`unwrap()` is the deliberate "panic propagates" policy — usually the right one,
since the protected data may be mid-update and inconsistent. Recover with
`into_inner()` only after re-validating or resetting the invariant yourself.

---

## `rayon` for Data Parallelism

```rust
use rayon::prelude::*;

let data: Vec<u64> = (0..1_000_000).collect();

// Parallel map
let squares: Vec<u64> = data.par_iter()
    .map(|x| x * x)
    .collect();

// Parallel reduce
let sum: u64 = data.par_iter().sum();

// Parallel for_each
data.par_iter().for_each(|x| {
    process(x);
});
```

### When to use rayon

- CPU-bound data processing.
- Embarrassingly parallel workloads.
- When you want automatic work-stealing.

### When NOT to use rayon

- I/O-bound work (use async instead).
- When the overhead of parallelism exceeds the benefit (small datasets).

---

## Summary

| Pattern | Use When | Thread-Safe | Overhead |
| --------- | ---------- | ------------- | ---------- |
| `std::thread` | OS-level parallelism | Yes | High (OS thread) |
| `tokio::spawn` | Async concurrency | Yes | Low (green thread) |
| `mpsc` channel | Message passing | Yes | Medium |
| `Mutex<T>` | Exclusive access | Yes | Medium |
| `Condvar` | Waiting for a condition | Yes | Medium |
| `RwLock<T>` | Read-heavy access | Yes | Medium |
| `Atomic*` | Simple counters | Yes | Low |
| `rayon` | Data parallelism | Yes | Work-stealing |
