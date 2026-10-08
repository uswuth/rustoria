# Async Rust Patterns

## Overview

Async Rust is built on **futures** — zero-cost state machines that are polled by an **executor**. The most common
runtime is **tokio**.

---

## Futures

A `Future` is a value that will eventually produce a value (or an error). Futures are lazy — they do nothing until polled.

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

struct Sleep {
    deadline: Instant,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.deadline {
            Poll::Ready(())
        } else {
            // Not ready yet: arrange for the executor to poll us again later.
            let waker = cx.waker().clone();
            let remaining = self.deadline - Instant::now();
            std::thread::spawn(move || {
                std::thread::sleep(remaining);
                waker.wake();
            });
            Poll::Pending
        }
    }
}
```

**The `Pending` contract:** a future that returns `Poll::Pending` must arrange for
`cx.waker()` to be woken when it can make progress. Returning `Pending` without
waking (or storing the waker for later) is the *lost-waker bug* — the executor
parks the task and never polls it again, so the program hangs silently. (The
thread-per-timer above illustrates the mechanics; real executors like tokio
integrate timers and I/O with their reactor instead.)

### `async fn` desugars to a Future

```rust
async fn fetch_data() -> String {
    // ...
    "data".to_string()
}

// Desugars to:
fn fetch_data() -> impl Future<Output = String> {
    async { "data".to_string() }
}
```

### `Pin`

Futures may be self-referential. `Pin` guarantees the future won't be moved in memory. You rarely need to use `Pin`
directly — `async fn` and combinators handle it for you.

---

## Tokio Runtime

Tokio is the most widely used async runtime. It provides:

- **Executor** — polls futures.
- **Reactor** — handles I/O events (epoll, kqueue, IOCP).
- **Timer** — time-based operations.

### Creating a runtime

```rust
#[tokio::main]
async fn main() {
    // tokio::main creates a multi-threaded runtime
    println!("Hello from async!");
}
```

### Multi-threaded vs current-thread

```rust
// Multi-threaded (default) — work-stealing across CPU cores
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() { /* ... */ }

// Current-thread — single-threaded, lower overhead
#[tokio::main(flavor = "current_thread")]
async fn main() { /* ... */ }
```

### When to use current-thread

- Low-latency applications where you want to avoid synchronization.
- Embedded or resource-constrained environments.
- Hosting a `LocalSet` for `!Send` work (see below).

**`!Send` types still can't be spawned normally.** `tokio::spawn` requires `Send`
even on a current-thread runtime — the bound is part of its signature regardless
of flavor. For `!Send` futures (e.g., code using `Rc` or `RefCell` across
`.await`), use a `LocalSet` and `tokio::task::spawn_local`:

```rust
use tokio::task::LocalSet;
use std::rc::Rc;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let local = LocalSet::new();
    local.run_until(async {
        let rc = Rc::new(42);
        tokio::task::spawn_local(async move {
            println!("{rc}"); // !Send is fine inside spawn_local
        })
        .await
        .unwrap();
    })
    .await;
}
```

---

## Spawning

`tokio::spawn` runs a future concurrently on the runtime.

```rust
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        do_work().await
    });

    // Do other work concurrently
    do_other_work().await;

    // Await the spawned task
    let result = handle.await.unwrap();
}
```

`tokio::spawn` requires the future to be `Send + 'static`:

- **`Send`** — on a multi-threaded runtime the task may migrate between worker
  threads at every `.await`, so every value held across an `.await` must be safe
  to move between threads.
- **`'static`** — the spawned task may outlive the function that spawned it, so
  it cannot borrow from the caller's stack frame. Move owned values in
  (`async move`) or share with `Arc`.

This is why these bounds exist: **values held across an `.await` become fields of
the future's state machine**, so the future's auto-trait impls are determined by
everything alive at each suspend point.

### `spawn` vs `spawn_blocking`

```rust
// spawn — for async work (must be Send)
tokio::spawn(async { /* async work */ });

// spawn_blocking — for CPU-intensive or blocking work
tokio::spawn_blocking(|| {
    // This runs on a separate thread pool
    std::thread::sleep(Duration::from_secs(1));
    42
}).await.unwrap();
```

**Rule of thumb:** Use `spawn` for async I/O, `spawn_blocking` for CPU-bound or blocking operations.

### `JoinSet` for managing multiple tasks

```rust
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let mut set = JoinSet::new();

    for id in 0..10 {
        set.spawn(async move {
            process_item(id).await
        });
    }

    while let Some(result) = set.join_next().await {
        match result {
            Ok(val) => println!("completed: {val}"),
            Err(e) => eprintln!("task failed: {e}"),
        }
    }
}
```

---

## Cancellation

Cancellation is **drop-based**: dropping a future cancels it. There is no
preemption — the future is dropped at its current `.await` yield point, and each
pending inner future is dropped in turn (their destructors and `Drop` impls run
normally). Two consequences:

- Cancellation can only happen **at `.await` points**. Code between awaits runs
  to completion; CPU-bound code that never yields cannot be cancelled — offload
  it with `spawn_blocking` or call `tokio::task::yield_now().await` periodically.
- Your code must tolerate being dropped at any `.await` — keep shared state
  consistent across yield points (this is what "cancellation safety" is about).

Two things are *not* cancellation: dropping a `JoinHandle` merely **detaches** the
task (it keeps running), and dropping the runtime cancels and drops its remaining
tasks (their destructors run) - whereas process exit (e.g. `std::process::exit`)
tears the process down without running any destructors at all.

```rust
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("tick");
        }
    });

    tokio::time::sleep(Duration::from_secs(3)).await;
    handle.abort(); // request cancellation
    let err = handle.await.unwrap_err(); // awaiting an aborted task yields a JoinError
    assert!(err.is_cancelled());
}
```

### `tokio::select!` and cancellation safety

`select!` polls all branches, resolves the first one that's ready, and **drops the
losing futures**. A branch is *cancellation-safe* only if dropping it mid-operation
loses nothing: `mpsc::Receiver::recv()` is safe (no message is consumed until one
is returned), but a future that has already read a partial frame from a socket and
buffered it internally would lose that data when dropped. For non-cancellation-safe
work, keep the buffer/state outside the `select!` loop and pass it in by reference.
Tokio's `select!` docs list which combinators are cancellation-safe.

```rust
tokio::select! {
    _ = async_operation() => {
        println!("operation completed");
    }
    _ = tokio::signal::ctrl_c() => {
        println!("received ctrl-c, cancelling");
    }
}
```

### `CancellationToken`

```rust
use tokio_util::sync::CancellationToken;

async fn worker(token: CancellationToken) {
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("shutting down");
                return;
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // do work
            }
        }
    }
}
```

---

## Backpressure

Backpressure prevents a fast producer from overwhelming a slow consumer.

### Bounded channels

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100); // buffer of 100

tokio::spawn(async move {
    for i in 0..1000 {
        // send returns Err when the channel is closed
        // and applies backpressure when the buffer is full
        if tx.send(i).await.is_err() {
            break;
        }
    }
});

while let Some(val) = rx.recv().await {
    process(val).await;
}
```

### `tokio::sync::Semaphore`

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

let semaphore = Arc::new(Semaphore::new(10)); // max 10 concurrent

let permit = semaphore.clone().acquire_owned().await.unwrap();
tokio::spawn(async move {
    do_work().await;
    drop(permit); // release the permit
});
```

### `tokio_stream` with `buffer_unordered`

```rust
use tokio_stream::StreamExt;

let results: Vec<_> = stream::iter(urls)
    .map(|url| fetch(url))
    .buffer_unordered(10) // max 10 concurrent
    .collect()
    .await;
```

---

## Common Async Pitfalls

### 1. Blocking the executor

```rust
// Bad: blocks the executor thread
async fn bad() {
    std::thread::sleep(Duration::from_secs(1)); // blocks!
    let data = std::fs::read_to_string("file.txt").unwrap(); // blocks!
}

// Good: use async alternatives
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    let data = tokio::fs::read_to_string("file.txt").await.unwrap();
}
```

For CPU-bound work there is no async alternative — offload it with
`tokio::task::spawn_blocking` (see "Spawning" above).

### 2. Holding a `MutexGuard` across `.await`

The precise rule:

- `std::sync::MutexGuard` is `!Send` → holding it across `.await` makes the
  future `!Send` and `tokio::spawn` **rejects it at compile time** (see pitfall 5).
  Even unspawned it's wrong: the OS thread keeps the lock while the task is suspended.
- `tokio::sync::MutexGuard` is `Send` → holding it across `.await` **compiles**,
  but it is an anti-pattern: every other task wanting the lock stalls until the
  await completes, and if the awaited code needs the same lock you deadlock.

So: drop the guard before `.await`, whichever mutex you use.

```rust
use tokio::sync::Mutex;

// Compiles, but bad: the lock is held across an await point
async fn bad(mutex: &Mutex<Vec<u8>>) {
    let mut guard = mutex.lock().await;
    guard.push(1);
    some_async_op().await; // lock held here — other tasks blocked!
    guard.push(2);
}

// Good: scope the guard so it is dropped before awaiting
async fn good(mutex: &Mutex<Vec<u8>>) {
    {
        let mut guard = mutex.lock().await;
        guard.push(1);
    } // guard dropped here
    some_async_op().await;
    {
        let mut guard = mutex.lock().await;
        guard.push(2);
    }
}
```

### 3. `!Send` futures in `tokio::spawn`

Values created and used *between* awaits are fine — only values held *across* an
`.await` become part of the future's state and affect its `Send`-ness.

```rust
// Bad: `rc` is held across the await, so the future is !Send and
// tokio::spawn rejects it: "future cannot be sent between threads safely"
async fn bad() {
    let rc = Rc::new(42);
    some_async_op().await;
    println!("{rc}"); // rc is still alive here — it crossed the await
}

// Good: use Arc (Send if T: Send + Sync), or drop the Rc before awaiting
async fn good() {
    let arc = Arc::new(42);
    some_async_op().await;
    println!("{arc}");
}
```

### 4. Forgetting to await a spawned task

```rust
// Bad: dropping the JoinHandle DETACHES the task — it keeps running,
// but you lose its result, its panic, and any shutdown coordination;
// if main returns, the runtime shuts down and the task is dropped mid-flight.
tokio::spawn(async {
    do_work().await;
});

// Good: keep the handle and await it
let handle = tokio::spawn(async {
    do_work().await
});
handle.await.unwrap();
```

### 5. `std::sync::Mutex` in async code

```rust
// Compile error once spawned: the std MutexGuard is !Send, so a future holding
// it across .await is !Send, and tokio::spawn rejects it.
async fn bad(mutex: &std::sync::Mutex<u64>) {
    let mut guard = mutex.lock().unwrap();
    some_async_op().await;
    *guard += 1;
}
// tokio::spawn(bad(&mutex)); // ERROR: future cannot be sent between threads safely

// Good: scope the guard so it is dropped before the await
async fn good(mutex: &std::sync::Mutex<u64>) {
    {
        let mut guard = mutex.lock().unwrap();
        *guard += 1;
    } // guard dropped
    some_async_op().await;
}
```

`tokio::sync::Mutex` exists for cases where you *must* hold a lock across `.await`
(its guard is `Send`), but prefer restructuring to scope the guard — see pitfall 2.

---

## Structured Concurrency

Structured concurrency means tasks have a well-defined scope — a parent task waits for all its children.

```rust
use tokio::task::JoinSet;

async fn process_all(items: Vec<Item>) -> Vec<Result<Output, Error>> {
    let mut set = JoinSet::new();

    for item in items {
        set.spawn(async move {
            process(item).await
        });
    }

    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        results.push(res.unwrap());
    }
    results
}
```

---

## Summary

| Pattern | Use When | Crate |
| --------- | ---------- | ------- |
| `async fn` | Most async code | std |
| `tokio::spawn` | Concurrent tasks | tokio |
| `spawn_blocking` | CPU-bound or blocking work | tokio |
| `select!` | Cancellation, timeouts | tokio |
| Bounded channels | Backpressure | tokio |
| `Semaphore` | Limiting concurrency | tokio |
| `JoinSet` | Managing multiple tasks | tokio |
| `CancellationToken` | Graceful shutdown | tokio-util |
