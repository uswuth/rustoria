---
name: rust-production
description: Use when designing, reviewing, or shipping production Rust.
---

# rust-production — Engineering for Production

Principles and decision guidance for production Rust. Reference-oriented; deep dives live in `references/production.md`,
`references/architecture.md`, `references/security.md`, `references/testing.md`, `references/performance.md`.

## When to use

- Designing or reviewing Rust architecture, APIs, error handling, concurrency, or deployment.
- Shipping or operating Rust services.
- Reviewing code for production readiness.

## Native-first dependency rule

The decision chain: **NATIVE FIRST -> VERIFY NEED -> CHOOSE DEPENDENCY -> VERIFY MAINTENANCE.**

Before adding any third-party dependency:

1. Does `std` already provide this capability?
2. Does `std`'s semantics and performance suffice for the requirement?
3. What does the crate provide beyond `std`?
4. Weigh maintenance, security, MSRV, compile time, binary size, ecosystem maturity, and operational cost.
5. Add the dependency when it provides justified value: materially more capability, ergonomics,
   portability, performance, ecosystem compatibility, or domain functionality that std does not
   reasonably provide.

This is not "never use dependencies." It is: understand native capabilities first, then introduce dependencies
intentionally. A crate that wraps a `std` API with no added value is a liability; a crate that provides real capability
(async runtime, serialization, database drivers) is justified. For the per-capability breakdown (what std covers and
when a dependency wins), see the capability decision table in `rust-std`.

## Engineering decision checklist

For any non-trivial design decision, ask:

- **Ownership:** Who owns this data? Who can mutate it? When is it dropped? Is the ownership model idiomatic?
- **Error paths:** Is this recoverable or fatal? What context should the error carry? `?` or `match`? What does the
  caller need to know?
- **Concurrency:** Is this `Send`? Is this `Sync`? Do I need `Arc`, `Mutex`, or channels? What are the failure modes
  (poisoning, deadlock, starvation)?
- **Async:** Is this I/O-bound or CPU-bound? How is cancellation handled? What is the backpressure strategy?
- **Performance:** Where are the allocations? Where are the clones? Is this cache-friendly? Have I measured, or am I guessing?
- **Dependencies:** Is the crate maintained? What is the license? What is the compile-time cost? Does `std` already do this?
- **API design:** Is this minimal? Is it difficult to misuse? Is it documented? Does it follow Rust API guidelines?
- **Testing:** What are the edge cases? How do I test concurrency? How do I test async code? Are there doc-tests?

## Architecture

- **Feature-oriented structure** for domain code: `features/users/`, `features/orders/` — each feature owns its
  models, logic, and tests. Prefer this over rigid `controllers/services/repositories` layering when the domain is the
  primary axis of change.
- **Module boundaries:** expose only what the module's facade needs. Adding a feature must not require editing a
  shared god-module. Dependencies point inward: features depend on `common`, never the reverse.
- **Workspaces** for multi-crate projects: `[workspace.dependencies]` for version inheritance, shared lints, one
  lockfile. See `references/toolchain.md`.
- **Small project ≠ toy architecture; production ≠ enterprise overengineering.** No microservices, CQRS, event
  sourcing, or DI containers unless the problem demands them.
- **Growth test:** a 5-feature project must grow to 50 features without touching unrelated modules. If adding a
  feature edits `common/`, the boundary is wrong.

## Error handling

- Libraries return concrete error types implementing `std::error::Error` (+ `Send + Sync + 'static` for `anyhow`
  interop). Use `thiserror` for derive-based error enums.
- Applications may use `anyhow` for ergonomic error propagation; `Box<dyn Error>` only at the outermost boundary.
- `?` propagates; `map_err` adds context; `ok_or`/`ok_or_else` converts Option to Result.
- Never silently discard errors: propagate, handle, or log. `.ok()` is not error handling.
- `unwrap()`/`expect()` only in tests, demos, or provably-infallible cases — never in production paths.

## Concurrency

- Prefer message passing (channels) over shared memory; use `Arc<Mutex<T>>`/`Arc<RwLock<T>>` when shared state is necessary.
- No blocking calls in async contexts: `spawn_blocking` for blocking work, async-native crates for I/O.
- `tokio::spawn` requires `Send + 'static` — values alive across `.await` become part of the future.
- Mutex poisoning: `lock()` returns `LockResult`; recover with `into_inner()` only when safe.
- Atomics: default `SeqCst`; weaken only with a correctness argument. See `references/concurrency.md`.

## Performance

- Measure first (criterion, flamegraph on Linux, `std::hint::black_box` discipline so the optimizer cannot elide
  measured work). Optimize only measured hot paths.
- Pre-allocate with `Vec::with_capacity`/`String::with_capacity` when size is known.
- Avoid unnecessary clones: borrow (`&T`, `&str`) instead of cloning; `Cow` for clone-on-write.
- Iterators are lazy — chain without intermediate `collect()`.
- Release profiles: `lto`, `codegen-units = 1`, `strip`, `panic = "abort"` where appropriate. See `references/performance.md`.

## Testing

- Unit tests for pure functions; integration tests via `tests/`; doc-tests for examples that must stay correct.
- Test edge cases, error paths, and concurrency (not just the happy path).
- `cargo test` runs unit + integration + doc tests; tests within a binary already run in parallel. Use
  `cargo nextest` for process-per-test isolation (a crashing test cannot take down siblings) and faster scheduling.
- Property tests (proptest) for invariants; snapshot tests (insta) for output stability.
- Coverage: `cargo tarpaulin` or `llvm-cov` — aim for meaningful coverage, not a number.

## Observability

- Structured logging with `tracing` (spans, events, propagation). Never hold a span guard across `.await`.
- Metrics: `metrics` crate 0.21+ handle API (`counter!("x").increment(1)`, `histogram!("x").record(v)`).
- Health checks, graceful shutdown (signal handling, JoinSet, CancellationToken), correlation IDs.
- Config: layered (defaults → file → env), secrets via `secrecy::SecretString`, never in code or logs.

## Security

- Validate all untrusted input at the boundary; canonicalize paths before use (`fs::canonicalize` + prefix check).
- No SQL string interpolation — use parameterized queries.
- No shell-string construction — pass `Command` args as an array.
- Integer overflow: debug builds panic, release wraps — use `checked_*`/`saturating_*`/`wrapping_*` explicitly.
- `cargo audit` + `cargo deny` in CI for dependency hygiene. See `references/security.md`.

## Production readiness checklist

- [ ] Compiles with `cargo clippy -- -D warnings` and `cargo fmt --check`
- [ ] `cargo test` green (unit + integration + doc)
- [ ] No `unwrap()`/`expect()` in production paths
- [ ] Errors are typed, contextual, and propagated or handled
- [ ] Concurrency primitives chosen deliberately; poisoning handled
- [ ] No blocking in async; cancellation is safe
- [ ] Dependencies justified (native-first rule applied)
- [ ] Observability: logs, metrics, health checks, graceful shutdown
- [ ] Security: input validation, no injection, secrets managed
- [ ] MSRV declared and CI-tested; edition appropriate
- [ ] Documentation: README, doc comments, examples

## House rules

- Native first: `std` before crates.
- No blocking in async; no casual `unwrap()` in production paths.
- Version-sensitive advice names its version.
- Examples use edition 2021.
