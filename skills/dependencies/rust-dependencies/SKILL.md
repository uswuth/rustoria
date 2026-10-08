---
name: rust-dependencies
description: Use when managing Rust dependencies, versions, and semver.
---

# Rust Dependencies and SemVer

Managing crates, version requirements, and the lockfile. (This skill is about Cargo dependency/version management, not git.)

## Declaring Dependencies (Cargo.toml)

```toml
[dependencies]
# Caret (the default) — see the caret table below
serde = "1.0.219"    # >=1.0.219, <2.0.0
serde = "1.0"        # >=1.0.0,  <2.0.0
serde = "1"          # >=1.0.0,  <2.0.0

# Exact version
serde = "=1.0.219"

# Tilde — no major/minor bumps
serde = "~1.0.3"     # >=1.0.3, <1.1.0
serde = "~1.0"       # >=1.0.0, <1.1.0

# Wildcard
serde = "1.*"        # >=1.0.0, <2.0.0
serde = "*"          # any version (not recommended)

# Comparison requirements
serde = ">=1.0.200, <2.0"

# Git dependency
serde = { git = "https://github.com/serde-rs/serde" }

# Path dependency
serde = { path = "../serde" }

# Features
serde = { version = "1.0", features = ["derive"] }

# Optional dependency
serde = { version = "1.0", optional = true }

# Renaming
serde1 = { package = "serde", version = "1.0" }
```

## SemVer and Caret Requirements

SemVer: **major** = breaking changes, **minor** = backward-compatible features, **patch** = backward-compatible fixes.
Cargo's *caret* requirements encode how "compatible" is interpreted — and the 0.x rule is the part people get wrong:

| Requirement | Allows | Why |
| ------------- | -------- | ----- |
| `^1.2.3` (= `"1.2.3"`) | `>=1.2.3, <2.0.0` | Minor/patch bumps are compatible |
| `^0.2.3` (= `"0.2.3"`) | `>=0.2.3, <0.3.0` | For 0.x, the **left-most non-zero digit** is the compatibility boundary — a minor bump (0.2 → 0.3) may break |
| `^0.0.3` (= `"0.0.3"`) | `=0.0.3` exactly | With 0.0.x, even the patch release may break |

So `anyhow = "1"` floats across all 1.x, but `ratatui = "0.30"` will NOT auto-upgrade to 0.31 — you must bump 0.x
minors deliberately and read the changelog.

Which bump to make when publishing: fixes only → patch; new backward-compatible API → minor; any breaking change →
major (or the left-most non-zero digit, pre-1.0).

## Cargo.lock

Version requirements in Cargo.toml are **ranges**, not pins — they never pin transitive dependencies. `Cargo.lock`
records the exact resolved version of every crate in the graph, direct and transitive.

```bash
# (Re)generate Cargo.lock from scratch without building
cargo generate-lockfile

# Update all dependencies, respecting Cargo.toml requirements
cargo update

# Update one crate (and what depends on it)
cargo update -p serde

# Update one crate to a specific version
cargo update -p serde --precise 1.0.219

# Check for outdated direct dependencies
cargo install cargo-outdated
cargo outdated
```

**Should you commit Cargo.lock?** The old advice — binaries yes, libraries no — is outdated. Modern practice:
**commit it for binaries AND libraries**. `cargo publish` ignores the lockfile, so committing it never constrains your
library's dependents; what it buys you is reproducible CI and bisectable history. The trade-off: without a committed
lockfile, CI always tests against the latest compatible deps, which catches upstream breakage early. Many projects
commit the lockfile and add a scheduled CI job that runs `cargo update` first to get both.

## MSRV-Aware Resolution (Rust 1.84+)

Declare your minimum supported Rust version, then make the resolver prefer dependency versions compatible with it:

```toml
# Cargo.toml (per package)
[package]
rust-version = "1.85"
```

```toml
# .cargo/config.toml
[resolver]
incompatible-rust-versions = "fallback"
```

With `fallback`, when several versions of a dependency satisfy your requirement, Cargo picks one that works with your
`rust-version` instead of blindly picking the newest.

## Minimal-Versions Testing

If you publish a library, verify your stated lower bounds actually compile — otherwise you may claim `serde = "1"`
while secretly requiring 1.0.190:

```bash
# Nightly-only as of late 2026
cargo +nightly update -Z minimal-versions
cargo +nightly test
cargo +nightly update   # restore the normal lockfile afterwards
```

## Inspecting and Auditing

```bash
# View dependency tree
cargo tree

# View with features
cargo tree -e features

# Check for duplicate versions of the same crate
cargo tree -d

# Security advisories (RUSTSEC)
cargo install cargo-audit
cargo audit
```

For CI gating beyond vulnerabilities - license policy, banned/duplicate crates, source allow-lists - use
`cargo-deny` (`cargo deny check`). See the production/security tooling for policy setup.

## Supply-Chain Considerations

What to reason about beyond `cargo audit`:

- **Yanked versions.** A yanked release stays resolvable if already in `Cargo.lock` but cannot be newly selected.
  `cargo update` may move you off (or onto) yanked versions; treat unexpected lockfile churn as reviewable.
- **Dependency sources.** Prefer registry dependencies. Git dependencies pin a `rev` (never track a moving branch in
  released code); path dependencies must not survive into published crates (`cargo package` rejects them).
- **Build-dependencies and proc-macros** execute at compile time on your machine and in CI - they are code execution
  trust decisions, not just library choices. Review `[build-dependencies]` and derive-heavy trees accordingly.
- **Feature activation.** Optional dependencies turn on with a feature elsewhere in the graph; `cargo tree -e features`
  shows who activated what. Unexpected activation can pull in heavier or riskier transitive trees.
- **Transitive depth.** Direct dependencies are reviewed; transitive ones are where surprises live. `cargo tree -d`
  for duplicates, and check who pulls in unmaintained crates (`cargo deny check bans` surfaces this).
- **Source replacement** (`.cargo/config.toml` `[source]` replacement, vendoring) changes where code comes from -
  legitimate for reproducibility/offline builds, but it must itself be reviewed and pinned.
- **Reproducibility.** Commit `Cargo.lock` for binaries; `cargo install --locked` in CI images; consider `cargo vendor`
  plus a checksum-verified vendor directory for hermetic builds.

## Workspace Management

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/*",
]

[workspace.dependencies]
serde = "1.0"
tokio = { version = "1", features = ["full"] }
```

```toml
# crates/my-crate/Cargo.toml
[dependencies]
serde = { workspace = true }
```

## Publishing Crates

```bash
# 1. Bump version in Cargo.toml, update CHANGELOG.md
# 2. Inspect what will be packaged
cargo package --list
# 3. Dry-run the publish
cargo publish --dry-run
# 4. Publish
cargo publish
# 5. Verify
cargo search my-crate
```

## Best Practices

1. **Use caret requirements** (`"1.0"`) for normal dependencies; pin exact (`=1.0.219`) only for known-bad ranges or
   reproducible tooling.
2. **Commit Cargo.lock** for binaries, and prefer committing it for libraries too (reproducible CI; `cargo publish`
   ignores it) — keep a scheduled `cargo update` CI job to stay current.
3. **Bump 0.x minors deliberately** — `^0.2.3` won't float to 0.3; read changelogs before bumping.
4. **Declare `rust-version`** and enable MSRV-aware resolution (Rust 1.84+).
5. **Audit in CI** — `cargo audit` for RUSTSEC advisories, `cargo deny` for advisories + licenses + bans.
6. **Test minimal versions** before publishing a library.
7. **Use workspaces with `workspace.dependencies`** for multi-crate projects.

## When to Use

- Adding or updating dependencies
- Resolving version conflicts
- Publishing crates
- Managing workspaces
- Security auditing
