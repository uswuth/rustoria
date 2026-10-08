# Agent Instructions

This repository is a Rust engineering skill system for AI coding agents.

## Quick start

1. Load the root router (`SKILL.md`) for any Rust task.
2. Route to the appropriate skill based on the task.
3. Load references when deep dives are needed.
4. Consult examples for compile-tested patterns.

## Skill catalog

### Native core

- `rust-fundamentals` — language basics
- `rust-std` — standard library API discovery
- `rust-production` — production engineering
- `rust-debugging` — debugging
- `rust-unsafe-checker` — unsafe Rust review
- `rust-dependencies` — dependency management

### Security

- `rust-code-audit` — repository security audit

### Library extensions

- `rust-tokio` — async runtime
- `rust-axum` — HTTP
- `rust-serde` — serialization
- `rust-diesel` — databases
- `rust-rayon` — parallelism
- `rust-clap` — CLI
- `rust-ratatui` — TUI

## Principles

- **Native Rust first.** Prefer `std` over third-party crates.
- **No blocking in async.** Use `spawn_blocking` or async-native crates.
- **No casual `unwrap()` in production paths.**
- **Version-sensitive advice names its version.**
- **Examples use edition 2021.**

## Agent execution safety

- **Read-only actions** (inspect, search, analyze, explain, review) are always safe.
- **Mutating actions** (create, modify, delete, install, commit, push, execute) require user awareness.
- **Destructive operations** require explicit user confirmation.
- Always: analyze first, plan second, modify third, validate fourth.

## Validation

```bash
bash scripts/validate.sh
bash scripts/test-triggers.sh
```
