---
name: rustoria
description: Production-grade Rust engineering guidance for AI agents. Use for designing, implementing, reviewing, or debugging Rust code.
---

# rustoria

Router and entry point for the rustoria skill system. Routes to the right skill, then to the right reference.

## How to use

1. Load this router for any Rust task.
2. Route to the skill matching the task (catalog below).
3. Skills point to `references/` for deep dives; load the reference when the task needs it.
4. For library work, load the library skill **and** the native skills it builds on (e.g. axum builds on tokio and serde).

## Native Rust first

Rust language, standard library, and Cargo/toolchain knowledge is the foundation. Third-party crates are extensions.
Before recommending any dependency, check whether `std` already provides the capability — see `rust-std` and the
native-first rule in `rust-production`.

## Skill catalog (14 skills)

### Native core

| Skill | Use when |
| ------- | ---------- |
| `rust-fundamentals` | Learning or reviewing Rust language basics: ownership, borrowing, lifetimes, types, traits, generics, macros. |
| `rust-std` | Choosing or using standard-library APIs: collections, iterators, strings, fs/io, process, net, sync, conversion traits. |
| `rust-production` | Designing, reviewing, or shipping production Rust: architecture, errors, concurrency, performance, testing, observability. |
| `rust-debugging` | Debugging Rust code, reading compiler errors, or investigating runtime failures. |
| `rust-unsafe-checker` | Writing or reviewing `unsafe` Rust, FFI, or raw-pointer code. |
| `rust-dependencies` | Managing dependencies, versions, SemVer, Cargo.lock, MSRV, and feature flags. |

### Security

| Skill | Use when |
| ------- | ---------- |
| `rust-code-audit` | Auditing a repository for security vulnerabilities, secret exposure, or supply-chain risks. |

### Library extensions

| Skill | Use when |
| ------- | ---------- |
| `rust-tokio` | Async runtimes, tasks, channels, timers, cancellation. |
| `rust-axum` | Building HTTP APIs and services with axum. |
| `rust-serde` | Serialization and deserialization with serde. |
| `rust-diesel` | Databases with Diesel ORM (schema, queries, migrations, pooling). |
| `rust-rayon` | Data parallelism and CPU-bound work. |
| `rust-clap` | Command-line interfaces and argument parsing. |
| `rust-ratatui` | Terminal user interfaces (TUI). |

## Ecosystem orientation

| Layer | Crate | Skill |
| ------- | ------- | ------- |
| Async runtime | tokio 1.x | `rust-tokio` |
| HTTP | axum 0.8 | `rust-axum` |
| Serialization | serde 1.x | `rust-serde` |
| Database | diesel 2.3 | `rust-diesel` |
| Parallelism | rayon 1.12 | `rust-rayon` |
| CLI | clap 4.x | `rust-clap` |
| TUI | ratatui 0.30 | `rust-ratatui` |

How the layers connect:

- **axum runs on tokio.** `axum::serve` takes a `tokio::net::TcpListener`; `#[tokio::main]` sets up the runtime.
- **diesel is synchronous.** In async servers, use a connection pool (r2d2) with `spawn_blocking`, or diesel-async.
- **rayon is for CPU-bound work.** Call it from `spawn_blocking`, never directly inside an `async fn`.
- **serde is the data layer.** axum's `Json` extractor/response is serde; diesel models derive serde traits when needed.

## References

Deep-dive documents, reachable from the skills that need them:

- `references/ownership.md` — moves, Copy vs Clone, drop order, Box/Rc/Arc, Weak, leaks
- `references/borrowing.md` — borrow rules, lifetimes, elision, RefCell/Cell/OnceLock/LazyLock
- `references/type-design.md` — newtypes, enums, typestate, PhantomData, Deref guidance
- `references/error-handling.md` — Result/Option, thiserror vs anyhow, `?`, error design
- `references/concurrency.md` — threads, channels, Mutex/RwLock, atomics, Send/Sync, poisoning
- `references/async.md` — Future/Pin, tokio runtime, spawn, cancellation, backpressure
- `references/performance.md` — allocations, clones, iterators, profiling, release profiles
- `references/testing.md` — unit/integration/doc tests, property tests, async tests, coverage
- `references/security.md` — input validation, secrets, injection, overflow, unsafe policy
- `references/production.md` — observability, config, graceful shutdown, deployment
- `references/architecture.md` — module boundaries, workspaces, feature-oriented structure
- `references/toolchain.md` — cargo, rustfmt, clippy, rustdoc, features, editions, MSRV

## Examples

Runnable, tested projects under `examples/`:

- `std-file-processor` — zero-dependency std-only tool (fs, io, iterators, custom errors)
- `minimal-api` — axum HTTP API
- `modular-monolith` — feature-oriented module boundaries
- `async-service` — tokio workers, scheduler, graceful shutdown
- `cli` — clap subcommands with tests
- `library` — validated newtypes with thiserror

## House rules

- **Native first.** Prefer `std` when it provides the capability; add dependencies intentionally.
- **No blocking in async.** Use `spawn_blocking` or async-native crates.
- **No casual `unwrap()`/`expect()` in production paths.** Examples may use them only where clearly demo-grade.
- **Version-sensitive advice names its version.** Skills state which Rust version or crate version a recommendation
  applies to.
- **Editions.** Examples use edition 2021. Skills note when guidance is edition-specific.
