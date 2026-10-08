---
name: rust-tokio
description: Use when working with async Rust and tokio.
---

# Tokio — Async Runtime for Rust

## Overview

Tokio is an event-driven, non-blocking I/O platform for writing asynchronous applications. It provides:

- Multithreaded, work-stealing task scheduler
- Reactor backed by OS event queue (epoll, kqueue, IOCP)
- Async TCP/UDP sockets
- Timers, channels, and synchronization primitives

## Installation

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }  # 1.x series (1.53 as of late 2026)
```

## Core Concepts

### 1. The Runtime

```rust
#[tokio::main]
async fn main() {
    // This runs on the tokio runtime
    println!("Hello from tokio!");
}
```

The `#[tokio::main]` macro sets up a multi-threaded runtime. For custom configuration:

```rust
// Multi-threaded runtime with 4 worker threads
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    println!("running on 4 worker threads");
}

// Current-thread runtime (single-threaded execution; tokio::spawn still requires Send -
// use LocalSet + spawn_local for !Send tasks)
#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("running on one thread");
}
```

### 2. Async Functions

```rust
async fn fetch_data() -> String {
    // .await suspends until the timer fires; other tasks run meanwhile
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    "data".to_string()
}
```

- `async fn` returns a `Future`
- `.await` suspends execution until the future completes
- No blocking — other tasks can run while waiting

### 3. Spawning Tasks

Futures passed to `tokio::spawn` must be `Send + 'static`:

- **`Send`** — the work-stealing scheduler can move a task to another worker thread between `.await` points, so the
  future and everything it holds must be safe to transfer across threads. (`Sync` — safe to *share* `&T` across
  threads — matters when tasks share references.) Most types are both; notable non-`Send` types: `Rc`, `RefCell`,
  `std::sync::MutexGuard`.
- **`'static`** — the task may outlive the function that spawned it, so it cannot borrow stack data. Every value
  alive across an `.await` becomes part of the future's state; move in owned values or `Arc`.

```rust
#[tokio::main]
async fn main() {
    // Spawn a new task onto the runtime
    let handle = tokio::spawn(async {
        println!("Running in a spawned task");
        42
    });

    // Await the task's result (JoinError if the task panicked or was aborted)
    let result = handle.await.unwrap();
    println!("Result: {}", result);
}
```

### 4. Join Multiple Tasks

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn fetch_data() -> i32 {
    sleep(Duration::from_millis(10)).await;
    1
}

async fn fetch_other() -> i32 {
    sleep(Duration::from_millis(20)).await;
    2
}

async fn fallible1() -> Result<i32, &'static str> { Ok(1) }
async fn fallible2() -> Result<i32, &'static str> { Ok(2) }

#[tokio::main]
async fn main() {
    // Both run concurrently; waits for both
    let (result1, result2) = tokio::join!(fetch_data(), fetch_other());
    println!("{result1} {result2}");

    // try_join! returns early on the first error
    let (r1, r2) = tokio::try_join!(fallible1(), fallible2()).unwrap();
    println!("{r1} {r2}");
}
```

### 5. Select — Race Multiple Futures

```rust
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let slow_future = sleep(Duration::from_secs(5));

    tokio::select! {
        _ = sleep(Duration::from_secs(1)) => {
            println!("Timeout!");
        }
        _ = slow_future => {
            println!("Completed first");
        }
    }
}
```

The losing branch's future is dropped mid-execution — see §11 for cancellation-safety.

### 6. Channels

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32); // bounded: buffer size 32

    tokio::spawn(async move {
        for i in 0..10 {
            tx.send(i).await.unwrap(); // demo-grade unwrap: fails only if rx dropped
        }
    });

    while let Some(value) = rx.recv().await {
        println!("Received: {}", value);
    }
}
```

### 7. Shared State

Choosing the mutex (this matches Tokio's own docs):

- **Short critical sections never held across `.await`**: use `std::sync::Mutex`. It's cheaper, needs no `.await` to
  lock, and is the recommended default. Just ensure the guard drops before any `.await` — a `std::sync::MutexGuard`
  held across `.await` makes the future `!Send`.
- **Guard must be held across `.await`**: use `tokio::sync::Mutex` (its guard is `Send`). Beware: it serializes tasks,
  so a contended async mutex becomes a bottleneck.

```rust
use std::sync::{Arc, Mutex}; // std Mutex: short critical sections only

#[tokio::main]
async fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = tokio::spawn(async move {
            // Guard is created, used, and dropped with no .await in between
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("Counter: {}", *counter.lock().unwrap());
}
```

If the critical section must span an `.await`, swap in `tokio::sync::Mutex` and use `counter.lock().await`.

### 8. Timers

```rust
use std::time::Duration;
use tokio::time::{interval, sleep};

async fn some_slow_operation() -> i32 {
    sleep(Duration::from_secs(10)).await;
    42
}

#[tokio::main]
async fn main() {
    // Sleep
    sleep(Duration::from_secs(1)).await;

    // Timeout
    let result = tokio::time::timeout(Duration::from_secs(5), some_slow_operation()).await;
    assert!(result.is_err()); // Elapsed

    // Interval (first tick completes immediately)
    let mut interval = interval(Duration::from_secs(1));
    for _ in 0..3 {
        interval.tick().await;
        println!("Tick");
    }
}
```

### 9. Async I/O

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn echo(socket: &mut TcpStream) -> std::io::Result<()> {
    let mut buf = [0; 1024];
    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            return Ok(()); // peer closed the connection
        }
        socket.write_all(&buf[..n]).await?;
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = echo(&mut socket).await {
                eprintln!("connection error: {e}");
            }
        });
    }
}
```

### 10. Async Traits

`async fn` in traits is **stable since Rust 1.75** — this is the default choice, no macro needed:

```rust
trait Database {
    async fn fetch_user(&self, id: u64) -> Option<String>;
}

struct Pg;

impl Database for Pg {
    async fn fetch_user(&self, id: u64) -> Option<String> {
        Some(format!("user-{id}"))
    }
}
```

Limitation: a trait with native `async fn` is **not dyn-compatible** (`Box<dyn Database>` fails to compile). When you
need trait objects, use the [`async-trait`](https://crates.io/crates/async-trait) crate, which boxes the returned
futures to restore object safety:

```toml
[dependencies]
async-trait = "0.1"
```

```rust
use async_trait::async_trait;

#[async_trait]
trait DynDatabase {
    async fn fetch_user(&self, id: u64) -> Option<String>;
}

// Now `Arc<dyn DynDatabase>` / `Box<dyn DynDatabase>` work.
```

### 11. Cancellation

Three mechanisms to know:

1. **Drop-based cancellation**: dropping a future cancels it. Cancellation takes effect at the next `.await` yield
   point — CPU-bound code that never `.await`s cannot be cancelled until it yields. Note: dropping a `JoinHandle` only
   *detaches* the task (it keeps running); use `JoinHandle::abort()` to cancel a spawned task.
2. **`select!` races**: the losing branch's future is dropped wherever it was suspended. That is only safe if
   dropping it loses no state — "cancel safety". `tokio::sync::mpsc::Receiver::recv` and `AsyncReadExt::read` are
   cancel-safe; a hand-rolled future that has partially consumed input may drop that input. Check the "Cancel safety"
   section of each tokio API's docs before using it in `select!`.
3. **`CancellationToken`** (from `tokio-util`): explicit, cooperative shutdown for many tasks at once.

```toml
[dependencies]
tokio-util = "0.7"
```

```rust
use std::time::Duration;
use tokio_util::sync::CancellationToken;

async fn worker(token: CancellationToken) {
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("Shutting down");
                return;
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                println!("Working...");
            }
        }
    }
}
```

### 12. Best Practices

1. **Don't block in async code** — `std::thread::sleep`, sync file I/O, and blocking DB drivers stall the executor.
   Wrap blocking calls in `tokio::task::spawn_blocking`. For CPU-bound *parallelism* use rayon instead (see the
   rust-rayon skill).
2. **Use `Arc` for shared state** across tasks.
3. **Pick the right mutex** — `std::sync::Mutex` for short sections never held across `.await`;
   `tokio::sync::Mutex` only when the guard must live across `.await` (see §7).
4. **Use `?` for error propagation** in async functions.
5. **Task granularity is a trade-off** — every task pays scheduling and memory overhead, so don't spawn one task
   per tiny item; but too few coarse tasks waste parallelism. Batch small units; split when units are large or must make
   progress independently.
6. **Use `JoinSet`** for managing many tasks.
7. **Backpressure** — use bounded channels (`mpsc::channel(n)`) so fast producers can't exhaust memory.
8. **Graceful shutdown** — use `CancellationToken` or `tokio::signal`.

### 13. Common Patterns

```rust
use std::sync::Arc;
use std::time::Duration;

async fn do_work(i: i32) -> i32 {
    tokio::time::sleep(Duration::from_millis(10)).await;
    i * 2
}

fn blocking_call() -> i32 {
    // e.g. a synchronous database driver or C FFI call
    42
}

#[tokio::main]
async fn main() {
    // spawn_blocking: run BLOCKING (synchronous) code on a dedicated thread pool.
    // For CPU-bound parallelism, use rayon instead.
    let result = tokio::task::spawn_blocking(blocking_call).await.unwrap();
    println!("blocking result: {result}");

    // JoinSet: manage a dynamic set of tasks
    let mut set = tokio::task::JoinSet::new();
    for i in 0..10 {
        set.spawn(do_work(i));
    }
    while let Some(res) = set.join_next().await {
        println!("completed: {:?}", res);
    }

    // Semaphore: limit concurrency
    let sem = Arc::new(tokio::sync::Semaphore::new(3));
    let permit = sem.clone().acquire_owned().await.unwrap();
    drop(permit); // released back to the semaphore
}
```

### 14. Feature Flags

```toml
# Minimal
tokio = { version = "1", features = ["rt"] }

# Common
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time", "net", "io-util"] }

# Everything
tokio = { version = "1", features = ["full"] }
```

## When to Use Tokio

- Web servers (with axum, actix, etc.)
- Database connection pooling
- File I/O
- Network protocols
- Background task processing
- Any I/O-bound concurrent workload
