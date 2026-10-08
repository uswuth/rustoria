# Trigger Tests

These tests verify that the skill system routes developer requests to the correct skills. The canonical harness is
`scripts/test-triggers.sh`, which **parses the tables in this file directly** — documentation and executed cases
cannot drift apart. Skill descriptions are read from the real `skills/**/SKILL.md` files at run time.

## Positive cases (should activate)

Each positive case lists **routing keywords**: at least one must appear in the expected skill's description
(case-insensitive). The keyword bridges the prompt (semantic) to the description (literal).

| ID | Prompt | Expected skill | Routing keywords |
| ---- | -------- | ---------------- | ---------------- |
| TC-001 | Explain ownership in Rust | `rust-fundamentals` | learning, language |
| TC-002 | Which collection should I use for a FIFO queue? | `rust-std` | standard-library |
| TC-003 | How do I choose between HashMap and BTreeMap? | `rust-std` | standard-library |
| TC-004 | Should I use Rc or Arc here? | `rust-std` | standard-library |
| TC-005 | Does std have an API for spawning a process? | `rust-std` | standard-library |
| TC-006 | How do I design a Rust error type? | `rust-production` | designing, production |
| TC-007 | Review my async code for cancellation safety | `rust-tokio` | async, tokio |
| TC-008 | Build an HTTP API with axum | `rust-axum` | http, axum |
| TC-009 | Serialize this struct to JSON | `rust-serde` | serializ |
| TC-010 | Set up Diesel with migrations | `rust-diesel` | diesel, databases |
| TC-011 | Parallelize this CPU-bound loop | `rust-rayon` | parallelism, rayon |
| TC-012 | Parse command-line arguments with clap | `rust-clap` | clap, cli |
| TC-013 | Build a terminal UI dashboard | `rust-ratatui` | terminal, ratatui |
| TC-014 | Debug this borrow checker error | `rust-debugging` | debugging, errors |
| TC-015 | Review this unsafe block | `rust-unsafe-checker` | unsafe |
| TC-016 | How do I manage Cargo.lock and MSRV? | `rust-dependencies` | dependencies, semver |
| TC-017 | Which conversion trait should I implement? | `rust-std` | standard-library |
| TC-018 | How do I use MaybeUninit safely? | `rust-unsafe-checker` | unsafe |
| TC-019 | How do I inspect Cargo feature resolution? | `rust-dependencies` | dependencies |
| TC-020 | What is the correct MSRV strategy? | `rust-dependencies` | versions, dependencies |
| TC-021 | How do I handle PathBuf and File? | `rust-std` | standard-library |
| TC-022 | Design a feature-oriented module structure | `rust-production` | designing, production |
| TC-023 | How do I test async code? | `rust-production` | production, designing |
| TC-024 | Choose between Cell and RefCell | `rust-std` | standard-library |
| TC-025 | How do I use LazyLock? | `rust-std` | standard-library |
| TC-026 | Audit this repository for security vulnerabilities | `rust-code-audit` | audit, security |
| TC-027 | Scan for exposed API keys and secrets | `rust-code-audit` | secret |
| TC-028 | Check for dependency vulnerabilities | `rust-code-audit` | dependency, security |
| TC-029 | Review this PR for security regressions | `rust-code-audit` | security, audit |
| TC-030 | Audit CI/CD pipeline for secret exposure | `rust-code-audit` | ci, secret |

## Negative cases (should NOT activate)

Each negative case lists keywords that must appear in **no** skill description — if a description ever starts
claiming one of these, the case fails.

| ID | Prompt | Must-not-match keywords | Why |
| ---- | -------- | ------------------------ | -------- |
| TC-N1 | Build a React component | react | Not Rust |
| TC-N2 | Write a Python script | python | Not Rust |
| TC-N3 | Configure nginx reverse proxy | nginx | Not Rust |
| TC-N4 | How do I center a div in CSS? | css | Not Rust |
| TC-N5 | How do I manage git branches? | git | `rust-dependencies` covers crate versions, not git |
| TC-N6 | Serialize data in Python | python | `rust-serde` is Rust-only |
| TC-N7 | Build a CLI in Go | golang | `rust-clap` is Rust-only |
| TC-N8 | Debug a JavaScript promise | javascript | `rust-debugging` is Rust-only |
| TC-N9 | Format this Rust code | format | Formatting is toolchain guidance, not a skill trigger |
| TC-N10 | Refactor this function for readability | refactor, readability | Generic refactoring is not a skill trigger |

## Composition cases (multiple skills)

Each composition case lists `skill:keyword` pairs; every pair must match that skill's description.

| ID | Prompt | Expected skill:keyword pairs |
| ---- | -------- | ----------------- |
| TC-C1 | Build a production axum API with diesel and serde | `rust-axum`:axum, `rust-diesel`:diesel, `rust-serde`:serializ, `rust-production`:production |
| TC-C2 | Review this tokio and rayon code for blocking in async | `rust-tokio`:tokio, `rust-rayon`:rayon, `rust-production`:production |
| TC-C3 | Audit this axum API for SQL injection and auth bypass | `rust-code-audit`:audit, `rust-axum`:axum, `rust-production`:production |
| TC-C4 | Review this unsafe FFI code for memory safety | `rust-code-audit`:audit, `rust-unsafe-checker`:unsafe, `rust-production`:production |

## How the harness works

`scripts/test-triggers.sh`:

1. Reads every `skills/**/SKILL.md` and extracts `name` and `description` (real files, not copies).
2. Parses the three tables above from this file (fails if a table is missing or unparseable).
3. Positive: the expected skill exists and its description contains at least one routing keyword.
4. Negative: no skill description contains any of the must-not-match keywords.
5. Composition: every `skill:keyword` pair matches.
6. Router: the root `SKILL.md` names `rustoria`, references every skill, and references no non-existent skill.
7. Exits non-zero if any case fails.

This tests that skill descriptions are discriminative — that an agent routing on descriptions would activate the
right skills and not the wrong ones — and that the router stays consistent with the skill set.
