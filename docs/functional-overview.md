# rustoria Functional Overview

How the skill system works at runtime.

## The skill system

rustoria is a collection of Markdown files that AI coding agents can load to improve their Rust engineering. The system
has three layers:

1. **Router** (`SKILL.md`) — the entry point. Routes any Rust task to the right skill.
2. **Skills** (`skills/**/SKILL.md`) — concise, reference-oriented decision documents.
3. **References** (`references/*.md`) — deep-dive documents for topics that need more depth.

## How an agent uses it

### 1. Load the router

For any Rust task, the agent loads the root `SKILL.md`. The router contains:

- A skill catalog (14 skills in 7 categories)
- An ecosystem orientation table (which crates cover which layers)
- House rules (native-first, no blocking in async, no casual unwrap, version-aware advice)

### 2. Route to a skill

The agent matches the task to a skill using the catalog. Examples:

| Task | Skill |
| ------ | ------- |
| "Explain ownership" | `rust-fundamentals` |
| "Which collection for a FIFO queue?" | `rust-std` |
| "Design a production error type" | `rust-production` |
| "Build an HTTP API" | `rust-axum` |
| "Review this unsafe block" | `rust-unsafe-checker` |

### 3. Load references when needed

Skills point to references for deep dives. For example, `rust-std` links to `references/ownership.md` for ownership
details, and `rust-production` links to `references/architecture.md` for module boundary guidance.

### 4. Consult examples

When the task involves writing code, the agent can consult the relevant example under `examples/` for a compile-tested pattern.

## Skill format

Each skill is a `SKILL.md` file with YAML frontmatter:

```yaml
---
name: rust-std
description: Use when choosing or using Rust standard-library APIs, collections, iterators, or conversion traits.
---
```

- `name` matches the directory name.
- `description` is a precise activation trigger (one sentence, under 60 characters, ending with a period).
- The body is reference-oriented: rules, decision tables, small correct examples.

## Trigger routing

The trigger system ensures the right skill activates for the right request. `scripts/test-triggers.sh` verifies this by:

1. Reading every skill's `description` from the filesystem (real files, not copies).
2. Parsing the case tables from `tests/trigger-tests.md` (the single source of truth - documentation and
   executed cases cannot drift apart).
3. Checking positive cases: the expected skill exists and its description contains at least one of the
   case's routing keywords.
4. Checking negative cases: no skill description contains any of the case's must-not-match keywords.
5. Checking composition cases: every `skill:keyword` pair matches the corresponding skill.
6. Checking the router: root `SKILL.md` names `rustoria`, references every skill, and references no
   non-existent skill.

Test cases are documented in `tests/trigger-tests.md` and include:

- **Positive** — realistic Rust requests that should activate a specific skill.
- **Negative** — Non-Rust requests that should not activate any skill.
- **Ambiguous near-misses** — Rust-adjacent requests that should not activate the wrong skill (e.g. "manage git
  branches" should not activate `rust-dependencies`).
- **Composition** — Multi-skill requests (e.g. "build a production axum API with diesel and serde").

## Native-first principle

The system enforces a native-first dependency rule:

> Before adding a third-party dependency, check whether `std` already provides the capability. Understand native
> capabilities first, then introduce dependencies intentionally.

This is stated in the root router, `rust-production`, and `rust-std`. It is not "never use dependencies" — it is
"understand native capabilities first."

## Validation

Every push runs:

- `scripts/validate.sh` — structure, frontmatter, catalog, links, secrets, personal paths.
- `scripts/test-triggers.sh` — trigger routing.
- Markdown linting and link checking.
- Per-example `cargo check`/`test`/`clippy`/`fmt`.

This ensures the repository remains technically accurate and structurally consistent as it evolves.
