# Changelog

All notable changes to this repository are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`rust-code-audit` skill**: layered repository security audit (reconnaissance, secret/credential detection with
  redaction, Git history, dependency and supply-chain review, unsafe integration, data-flow reasoning, CI/CD and
  build-script review, severity/confidence finding model, audit modes, false-positive control)
- Trigger tests for the audit skill: 5 positive, 2 negative, 2 composition cases
- Native-first capability decision table (language/std/Cargo vs third-party crates, 21 areas) in `rust-std`,
  cross-referenced from `rust-production`
- Supply-chain considerations section in `rust-dependencies` (yanked releases, sources, feature activation,
  transitive depth, source replacement, reproducibility)
- Verified discovery-path citations in all six agent `INSTALL.md` files (official docs URLs per agent)
- Release workflow gate: per-example `cargo check` matrix before publishing a release
- Exit-code contract tests for `examples/std-file-processor` (help=0 on stdout, usage=2 on stderr)
- README header logo (`assets/logo.svg`, previously unreferenced) and a one-line install path via the
  `skills` CLI: `npx skills add uswuth/rustoria --full-depth` (verified against the CLI's discovery rules:
  `--full-depth` is required for category-nested skills)

### Changed

- Skill catalog updated to **14 skills in 7 categories** (new `security` category)
- `scripts/test-triggers.sh` now parses `tests/trigger-tests.md` as the single source of truth
  (documentation and executed cases cannot drift); router assertions added
- `scripts/validate.sh` hardened: section-scoped scanning, `*/target/*` excluded, anchor-stripped link checks,
  workflow integrity checks (explicit `permissions:`, SHA-pinned actions, no untrusted `${{ }}` interpolation
  in `run:` blocks), AGENTS.md catalog gate
- `scripts/setup.sh` installs directory bundles (`<skill-name>/SKILL.md`) matching every supported agent's
  documented discovery rules; fails loudly on copy errors
- Audit skill intro, data-flow chain (normalization and authorization stages), and tool table aligned with
  the skill's actual structure; "signals are not verdicts" guidance added
- CI workflows pinned to verified commit SHAs with explicit `permissions` blocks
- First real-runner CI fixes: dropped lychee `--exclude-mail` (removed in lychee 0.24.x)
  and bumped `actions/checkout` to v7.0.1 (Node 24 native), SHA-pinned
- `scripts/validate.sh` frontmatter gate extended: quote-aware description checks (a plain scalar containing
  an unquoted `repos: security`-style colon is rejected) plus a PyYAML parse of every frontmatter when
  python3 is available - both mutation-tested

### Fixed

- Skill and reference accuracy corrections verified against current official docs (Rust 1.99 stable window):
  refutable `let` patterns, `static_mut_refs` edition behavior, `LazyLock` availability, `black_box` guidance,
  `ratatui` 0.3, `cargo-deny` 0.20 configuration keys, `reqwest` 0.13, deprecated `serde_yaml` guidance,
  channel/hasher API claims, tracing span guard `Send` nuance, and related items
- Agent adapter install paths corrected to officially documented locations (for example Codex discovers
  skills from `.agents/skills`, not `~/.codex/skills`) with per-agent source citations
- `examples/std-file-processor` help/usage exit codes now follow the conventional 0/2 contract
- Fabricated source URL in `docs/planned-capabilities.md` (`topcoat-rs/topcoat` 404) replaced with the
  verified repository (`tokio-rs/topcoat`); entry now names the crate version and states the pre-1.0
  deferral reason
- `rust-code-audit` frontmatter description is now quoted: the unquoted colon in `...repos: security audit...`
  parsed as a nested mapping and strict YAML parsers (e.g. the `skills` CLI) skipped the skill entirely

### Security

- Issue-form templates no longer interpolate untrusted `${{ }}` input into `run:` blocks (release workflow
  validates and passes versions through `env:` instead)
- Legacy Markdown issue templates removed (kept the `.yml` forms); repository sensitive-file list updated

### Removed

- Redundant/obsolete documentation: development-session reports, `docs/planned-skills.md`
  (content consolidated in `docs/planned-capabilities.md`), `docs/repository-naming.md`
  (naming decision finalized)
- Legacy `.github/ISSUE_TEMPLATE/bug_report.md` and `feature_request.md` (superseded by `.yml` issue forms)

## [1.0.0] - 2026-10-08

### Added

- **13 canonical skills** across 6 categories:
  - Native core: `rust-fundamentals`, `rust-std`, `rust-production`, `rust-debugging`, `rust-unsafe-checker`, `rust-dependencies`
  - Library extensions: `rust-tokio`, `rust-axum`, `rust-serde`, `rust-diesel`, `rust-rayon`, `rust-clap`, `rust-ratatui`
- **12 reference documents**: ownership, borrowing, type-design, error-handling, concurrency, async, performance,
  testing, security, production, architecture, toolchain
- **6 examples**, all compile-tested: `std-file-processor` (zero-dependency), `minimal-api`, `modular-monolith`,
  `async-service`, `cli`, `library`
- **Validation harness**: `scripts/validate.sh` (structure, frontmatter, catalog, links, secrets, personal paths) and
  `scripts/test-triggers.sh` (trigger routing with positive, negative, ambiguous, and composition cases)
- **CI**: structure/trigger validation, per-example Rust checks (check/test/clippy/fmt), markdown lint, link checking
- **Native-first dependency rule** established as a repository-wide engineering principle
- **Planned skills** process documented in `docs/planned-capabilities.md`

### Changed

- Consolidated overlapping skills: `rust-teacher` merged into `rust-fundamentals`; `rust-meta-cognition` decision
  questions folded into `rust-production`
- Renamed `rust-version-control` to `rust-dependencies` (it covers dependency management, not git)
- Renamed `libraries/clap` → `libraries/rust-clap`, `libraries/ratatui` → `libraries/rust-ratatui` for name-directory consistency
- All examples upgraded to axum 0.8 where applicable; edition 2021 declared explicitly
- Root `SKILL.md` rewritten as a concise router with ecosystem orientation

### Removed

- 7 domain skills (`rust-cli`, `rust-cloud`, `rust-embedded`, `rust-fintech`, `rust-iot`, `rust-ml`, `rust-web`) —
  removed per project decision; not part of v1
- `rust-topcoat`, `rust-gpui-kit` — deferred; see `docs/planned-capabilities.md`
- `rust-dynamic-skills` — Hermes workflow helper, not user-facing Rust guidance
- `rust-meta-cognition` — folded into `rust-production`
- `rust-teacher` — merged into `rust-fundamentals`
- `skills/mastery/rustoria` — duplicate name, orphaned, contradictory; ecosystem map salvaged into root router
- Broken duplicate trigger harness (`tests/trigger-tests.sh`) — replaced by canonical `scripts/test-triggers.sh`
