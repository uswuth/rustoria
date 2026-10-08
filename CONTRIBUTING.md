# Contributing to rustoria

Thank you for your interest in contributing. This repository is a curated collection of Rust engineering knowledge —
**quality over quantity**.

## Three concepts — not one

This repository has three distinct concepts. Do not conflate them.

1. **Skills** (`skills/`) — knowledge used by the agent during a task. Concise, reference-oriented, actionable.
2. **References** (`references/`) — deep technical source material. Reachable from skills. Authoritative, version-aware.
3. **Intelligence/Maintenance** (`docs/`, `scripts/`, `.github/`) — things that keep the repository current.
   Ecosystem monitoring, validation, CI/CD.

Do not turn ecosystem monitoring into another skill. That would recreate the skill inflation this repository
deliberately avoids.

## What belongs here

- **Skills**: concise, reference-oriented SKILL.md documents that help an AI agent write better Rust.
- **References**: deep-dive documents on specific Rust topics, reachable from skills.
- **Examples**: small, compile-tested, idiomatic Rust projects that demonstrate a pattern.

## What does not belong here

- Tutorials disguised as skills (a skill is a reference, not a 20-lesson course).
- Crate-specific skills for experimental or unmaintained crates.
- Content that has not been verified against current official documentation.
- AI-slop: filler, motivational writing, unsupported claims, repetitive slogans.
- Duplicate skills or overlapping responsibilities.

## Skill quality contract

Every skill must satisfy the [Skill Quality Contract](docs/skill-quality-contract.md). This is the anti-skill-slop gate.
Skills that do not pass are not accepted.

## Skill format

Every skill is a `SKILL.md` file at `skills/<category>/<skill-name>/SKILL.md` with YAML frontmatter:

```yaml
---
name: rust-something
description: One sentence, trigger-first, under 100 characters, ending with a period.
---
```

Rules:

- `name` matches the directory name.
- `description` is a precise activation trigger — an AI agent routes on it.
- Body is reference-oriented: rules, decision tables, small correct examples. Target under 500 lines.
- Code snippets must be valid Rust as presented.
- Version-sensitive advice names its version (e.g. "stable since Rust 1.75").
- Link to `references/` for deep dives; do not duplicate them.
- Every skill must be routed in the root `SKILL.md` and listed in the README catalog.

## Adding a reference

References live in `references/<topic>.md`. They must:

- Be technically accurate against current official Rust documentation.
- Contain valid Rust code snippets.
- Be linked from at least one skill.
- Cite first-party sources (std docs, Rust Book, Reference, Rustonomicon, Cargo Book) for native Rust claims.

## Adding an example

Examples live in `examples/<name>/` and must:

- Compile and pass `cargo test` on stable Rust.
- Use edition 2021 (unless there is a documented reason).
- Have zero or minimal, justified dependencies.
- Demonstrate a pattern taught by a skill.
- Pass `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.

## PR requirements

Every PR must:

- [ ] Explain **why** the change is needed
- [ ] Identify affected skills
- [ ] Avoid duplicate skill content
- [ ] Use official documentation for API claims
- [ ] Compile-test Rust examples
- [ ] Update trigger tests when routing changes
- [ ] Update README/catalog when skill inventory changes
- [ ] Update CHANGELOG for user-visible changes
- [ ] Pass CI

### For Rust API changes

Require:

- **Rust version:**
- **Edition:**
- **Dependency version:**
- **Official documentation checked:**
- **Example compile-tested:**

## Validation

Before submitting, run:

```bash
bash scripts/validate.sh       # structure, frontmatter, catalog, links, secrets
bash scripts/test-triggers.sh  # trigger routing tests
```

Both must pass. CI additionally runs markdown linting, link checking, and per-example `cargo check`/`test`/`clippy`/`fmt`.

## Commit convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat(scope):` — new skill, reference, or example
- `fix(scope):` — bug fix or accuracy correction
- `docs(scope):` — documentation change
- `refactor(scope):` — restructuring without content change
- `test(scope):` — test addition or fix
- `ci(scope):` — CI/workflow change
- `chore(scope):` — maintenance task

Examples:

```text
feat(std): add collection selection guidance
fix(axum): correct axum 0.8 path parameter example
fix(unsafe): correct MaybeUninit safety guidance
docs(readme): update skill catalog
test(triggers): add native API discovery cases
ci(validation): run example clippy checks
```

Keep commits **atomic, focused, explainable, independently reviewable**. Avoid: "update stuff", "fix things", "changes",
"final", "AI improvements".

## Planned skills

Skills and agent adapters that are intentionally deferred (e.g. `rust-topcoat`, `rust-gpui-kit`, additional
agent `INSTALL.md` directories) are listed in `docs/planned-capabilities.md`. Do not submit them until the
criteria there are met.

## Agent execution safety

When contributing skills that influence agent behavior:

- Distinguish **read-only** actions (inspect, search, analyze, explain, review) from **mutating** actions (create,
  modify, delete, install, commit, push, execute).
- Destructive operations should require explicit user confirmation.
- Skills should encourage: analyze first, plan second, modify third, validate fourth.
