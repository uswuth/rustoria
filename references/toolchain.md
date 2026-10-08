# Toolchain and Cargo Reference

## Overview

The Rust toolchain is a set of coordinated tools managed by `rustup`, with
`cargo` as the daily driver. This reference maps the tools, then covers the
commands, configuration, and policies that matter in practice.

---

## The Tool Map

| Tool | Role | Key invocations |
| ------ | ------ | ----------------- |
| `rustup` | Installs and switches toolchains, components, targets | `rustup update`, `rustup component add`, `rustup target add` |
| `rustc` | The compiler — direct use is rare; cargo drives it | `rustc --version --verbose`, `rustc --print target-list` |
| `cargo` | Build system + package manager + task runner | `cargo build`, `cargo test`, ... |
| `rustfmt` | Code formatter | `cargo fmt` (`--check` in CI) |
| `clippy` | Linter (hundreds of lints beyond rustc's) | `cargo clippy` |
| `rustdoc` | Documentation generator | `cargo doc --open` |

---

## rustup: Managing Toolchains

```bash
rustup update                                  # update all installed toolchains
rustup default stable                          # set the default toolchain
rustup toolchain install nightly --profile minimal
rustup component add clippy rustfmt rust-analyzer
rustup target add x86_64-unknown-linux-musl    # cross-compilation target
rustup show                                    # what's installed and active
```

Override per invocation with `+toolchain`: `cargo +nightly build`,
`cargo +1.75 check`.

`rustc` directly is mostly for introspection:

```bash
rustc --version --verbose    # version, commit, host triple
rustc --print target-list    # all supported targets
rustc --print sysroot        # where the toolchain lives
```

---

## Daily Cargo Commands

| Command | Notes |
| --------- | ------- |
| `cargo check` | Fastest iteration: type-checks, no codegen. Use `--all-targets` to include tests/benches/examples |
| `cargo build` / `--release` | Debug (fast compile) vs release (optimized) |
| `cargo run [-- args...]` | Build + run the default binary; `--release` for optimized |
| `cargo test` | Unit + integration + doc tests. Filter: `cargo test name`. Test-binary flags: `-- --nocapture`, `-- --test-threads=1`, `-- --ignored` |
| `cargo bench` | Built-in harness is **nightly-only** (`#![feature(test)]`); on stable use `criterion` (see `performance.md`) |
| `cargo doc --open` | Build + open docs; `--no-deps` to skip dependencies |
| `cargo tree` | Dependency graph; `-d` shows duplicates, `-i foo` shows what depends on `foo` |
| `cargo metadata --format-version 1` | Machine-readable JSON of the workspace graph — the basis for tooling |
| `cargo update` | Bump the lockfile within semver ranges; `-p foo` for one crate, `-p foo --precise 1.2.3` to pin |
| `cargo clean` | Delete `target/` |
| `cargo new foo` / `cargo init` | New package (binary default, `--lib` for a library); `init` in an existing directory |
| `cargo add serde --features derive` / `cargo remove serde` | Edit `Cargo.toml` dependencies - `cargo add` built into cargo since 1.62, `cargo remove` (alias: `cargo rm`) since 1.66 (both formerly the `cargo-edit` plugin) |

---

## rustfmt and Clippy

```bash
cargo fmt                 # format the workspace
cargo fmt --check         # CI mode: fail if anything is unformatted
```

Configuration lives in `rustfmt.toml` (project root). Most stable options are
layout knobs; anything exotic tends to be nightly-only.

```bash
cargo clippy                                  # lint
cargo clippy --all-targets -- -D warnings     # CI mode: deny all warnings
cargo clippy --fix --allow-dirty              # auto-apply machine-applicable suggestions
```

Lint groups beyond the default correctness/style/complexity/perf sets:

- `clippy::pedantic` — stricter, opinionated; opt in per-crate
  (`#![warn(clippy::pedantic)]` at the crate root) and `#[allow]` the lints you
  disagree with. Expect noise.
- `clippy::nursery` — experimental lints; useful for occasional audits, too
  unstable/false-positive-prone for CI.

---

## Documentation (rustdoc)

```bash
cargo doc --open          # build and open your crate's docs
cargo doc --no-deps       # skip dependency docs (faster)
```

- Doc comments are `///` (items) and `//!` (modules/crates), written in Markdown.
- Use **intra-doc links** instead of bare names: ``[`crate::Config`]``,
  ``[`std::sync::Mutex`]`` — broken links are warnable and can be denied with
  `#![deny(rustdoc::broken_intra_doc_links)]`.
- Code blocks in doc comments run as tests — see `testing.md`, Documentation Tests.

---

## Workspaces

One repository, several crates, one lockfile and one `target/`:

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["crates/app", "crates/domain", "crates/infra"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used = "warn"
```

Member crates inherit:

```toml
# crates/domain/Cargo.toml
[dependencies]
serde = { workspace = true }              # version/features from the root

[lints]
workspace = true                          # opt into [workspace.lints]
```

- `resolver = "2"` is the current dependency resolver (per-target/build-dep aware
  feature resolution). It is the default when the root *package* uses edition
  2021+; a **virtual** workspace (root with no package) must set it explicitly.
- `[workspace.dependencies]` centralizes versions; members can still add
  features (`{ workspace = true, features = ["alloc"] }`).
- **When to split into multiple workspaces:** independent release cadence or
  ownership, compile-time isolation, or conflicting dependency constraints.
  Tightly coupled crates that change together belong in one workspace
  (see `architecture.md`).

---

## Features

```toml
[features]
default = ["json"]
json = ["dep:serde_json"]   # dep: syntax — no implicit "serde_json" feature
full = ["json", "dep:tokio"]

[dependencies]
serde_json = { version = "1", optional = true }
tokio = { version = "1", optional = true }
```

- An optional dependency implicitly creates a feature of the same name **unless**
  it is referenced with `dep:name` in some feature.
- **Unification:** features are additive — the build enables the union of all
  features requested anywhere in the dependency graph. Consequences:
  - Features must only *add* capability, never remove or change required
    behavior — a "negative" feature breaks as soon as two dependents disagree.
  - A library must compile with `--no-default-features` and with
    `--all-features`. Test both in CI; `cargo-hack --feature-powerset` tests
    every combination.
- Consumers select with `cargo build --features json` /
  `--no-default-features` / `--all-features`.

---

## Dependency Kinds and Build Scripts

| Section | Used by | Compiled for |
| --------- | --------- | -------------- |
| `[dependencies]` | the crate itself and its dependents | target |
| `[dev-dependencies]` | tests, benches, examples only — never shipped to dependents | target |
| `[build-dependencies]` | `build.rs` only | host |

A `build.rs` in the package root compiles and runs **before** the crate; it emits
`cargo::` directives on stdout:

```rust
// build.rs
fn main() {
    println!("cargo::rerun-if-changed=schema.sql"); // rebuild only when this changes
    println!("cargo::rustc-link-lib=sqlite3");
}
```

(`cargo::` syntax since 1.77; single-colon `cargo:` is the older form.) Common
directives: `rerun-if-changed` / `rerun-if-env-changed`, `rustc-link-lib` /
`rustc-link-search`, `rustc-cfg`. Generated code goes in `OUT_DIR` and is
included with `include!(concat!(env!("OUT_DIR"), "/gen.rs"))`.

**When to avoid build scripts:** when code generation could be a committed file
or a proc macro, and when wrapping a system C library a pure-Rust crate already
covers — build scripts slow every build and complicate cross-compilation.

---

## Cargo.lock Policy

- **Binaries / applications:** always commit `Cargo.lock` — it is what makes
  deployments reproducible.
- **Libraries:** the old rule ("never commit") is outdated. Dependents always
  ignore a library's lockfile (only the workspace *root's* lockfile governs a
  build), so committing costs nothing and buys reproducible CI and bisection.
  Current guidance leans toward committing it for libraries too.
- Either way, use `cargo build --locked` / `cargo test --locked` in CI so a
  lockfile drift fails loudly instead of silently resolving new versions.

---

## Editions

Editions are opt-in, per-package language changes. Crates on different editions
interoperate freely — the edition affects only the crate that declares it.

```toml
[package]
name = "my-crate"
edition = "2024"   # set at `cargo new` time; change deliberately, not casually
```

| Edition | Stabilized | Highlights |
| --------- | ------------ | ------------ |
| 2015 | Rust 1.0 | Original |
| 2018 | 1.31 | Module-system paths (`crate::`), `dyn Trait`, no `extern crate` needed |
| 2021 | 1.56 | Disjoint closure captures, `IntoIterator` for arrays, resolver v2 default, new prelude entries |
| 2024 | 1.85 | `gen` keyword reserved, `static_mut_refs` hard error, `#[unsafe(...)]` attributes, tighter temporary drop scopes, resolver v3 |

Migrating: `cargo fix --edition` applies machine-applicable changes, then bump
the `edition` field and address what remains; the *Edition Guide* documents each
edition's changes.

---

## MSRV (Minimum Supported Rust Version)

Declare it and enforce it:

```toml
[package]
rust-version = "1.75"   # cargo errors if the active toolchain is older
```

Verify in a stable/MSRV CI matrix:

```bash
rustup toolchain install 1.75 --profile minimal
cargo +1.75 check --workspace
```

Two dependency-resolution helpers:

- Edition 2024's resolver v3 already prefers dependency versions compatible with
  your `rust-version`.
- On 1.84+, the same behavior is available explicitly:

  ```toml
  # .cargo/config.toml
  [resolver]
  incompatible-rust-versions = "fallback"
  ```

(`cargo msrv`, third-party, finds the true minimum by bisecting toolchains.)

---

## rust-toolchain.toml

Pin the toolchain per repository — `rustup` reads it automatically:

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.85"          # "stable", "1.85", or "nightly-2026-01-15"
components = ["clippy", "rustfmt", "rust-analyzer"]
targets = ["x86_64-unknown-linux-musl"]
```

**Policy:** default to stable. Require nightly only for specific, named features
(e.g., docs.rs `doc_cfg`), gate nightly-only code behind `cfg`, and pin a dated
nightly so CI doesn't break when nightly changes.

---

## Build Profiles

Profiles control optimization and debug info per build kind (`dev`, `release`,
`test`, `bench`):

```toml
[profile.release]
opt-level = 3        # 0–3 speed; "s"/"z" optimize for size
lto = "thin"         # "off" (default), "thin", or true/"fat"
codegen-units = 16   # default; 1 = best optimization, slowest compile
panic = "unwind"     # "abort": smaller/faster binaries, no catch_unwind
strip = false        # true: strip symbols (smaller, poorer backtraces)
debug = false        # true: keep debug info in release (useful for profiling)

# Faster iteration: optimize dependencies, not your own crate
[profile.dev.package."*"]
opt-level = 2
```

`panic = "abort"` removes unwinding entirely — libraries must not set it (it's
only honored in the workspace root). See `performance.md` for when to tune.

---

## Reproducible Builds

The recipe is short:

1. Commit `Cargo.lock`; build with `--locked` in CI.
2. Pin the toolchain with `rust-toolchain.toml`.
3. Trim dependencies and `default-features = false` where you don't need them —
   fewer moving parts, less to pin.

That gets you byte-comparable builds across machines for practical purposes.
(Full bit-for-bit reproducibility additionally needs control of linker, sysroot,
and build paths — out of scope for most teams.)

---

## Dependency Hygiene

Wire into CI, fail on new findings:

- **`cargo audit`** — checks the lockfile against the RustSec Advisory Database.
- **`cargo deny`** — broader policy: advisories, licenses, banned crates,
  duplicate versions, and sources; configured in `deny.toml`.
- **`cargo outdated`** — shows dependencies behind their latest versions
  (third-party plugin; `cargo update` alone only moves within semver ranges).

Details and configuration: `security.md`, Dependency Auditing.
