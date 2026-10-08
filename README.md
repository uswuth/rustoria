# rustoria

<p align="center">
  <img src="assets/logo.svg" alt="rustoria logo" width="180">
</p>

Production-grade Rust engineering guidance for AI coding agents. A curated collection of **14 skills** covering the Rust
language, standard library, Cargo/toolchain, production engineering, and key ecosystem libraries — with a strong
native-Rust foundation and third-party crates as extensions.

## What this is

A skill system for AI agents that write Rust. Each skill is a concise, reference-oriented `SKILL.md` that helps an agent
make correct engineering decisions. The root `SKILL.md` is a router; skills point to deep-dive references; examples
prove the guidance compiles and tests green.

**Native Rust first.** The Rust language, standard library, and Cargo/toolchain are the foundation. Library skills
(tokio, axum, serde, diesel, rayon, clap, ratatui) are extensions that build on that foundation.

## Skill catalog

### Native core (6)

| Skill | Use when |
| ------- | ---------- |
| [`rust-fundamentals`](skills/fundamentals/rust-fundamentals/SKILL.md) | Learning, teaching, or reviewing Rust language basics |
| [`rust-std`](skills/std/rust-std/SKILL.md) | Choosing or using standard-library APIs |
| [`rust-production`](skills/production/rust-production/SKILL.md) | Designing, reviewing, or shipping production Rust |
| [`rust-debugging`](skills/debugging/rust-debugging/SKILL.md) | Debugging Rust code or reading compiler errors |
| [`rust-unsafe-checker`](skills/unsafe-checker/rust-unsafe-checker/SKILL.md) | Writing or reviewing unsafe Rust and FFI |
| [`rust-dependencies`](skills/dependencies/rust-dependencies/SKILL.md) | Managing dependencies, versions, SemVer, MSRV |

### Security (1)

| Skill | Responsibility |
| ------- | --------------- |
| [`rust-code-audit`](skills/security/rust-code-audit/SKILL.md) | Repository-wide security audit, secret detection, supply-chain analysis |

### Library extensions (7)

| Skill | Use when |
| ------- | ---------- |
| [`rust-tokio`](skills/libraries/rust-tokio/SKILL.md) | Async runtimes, tasks, channels, cancellation |
| [`rust-axum`](skills/libraries/rust-axum/SKILL.md) | Building HTTP APIs with axum |
| [`rust-serde`](skills/libraries/rust-serde/SKILL.md) | Serialization and deserialization |
| [`rust-diesel`](skills/libraries/rust-diesel/SKILL.md) | Databases with Diesel ORM |
| [`rust-rayon`](skills/libraries/rust-rayon/SKILL.md) | Data parallelism and CPU-bound work |
| [`rust-clap`](skills/libraries/rust-clap/SKILL.md) | Command-line interfaces and argument parsing |
| [`rust-ratatui`](skills/libraries/rust-ratatui/SKILL.md) | Terminal user interfaces |

## Installation

### As a skill collection

#### One-line install (npx)

The [`skills` CLI](https://github.com/vercel-labs/skills) installs this repository's skills into any supported
agent's skills directory (requires Node.js 22.20+):

```bash
npx skills add uswuth/rustoria --full-depth
```

`--full-depth` is required so the root router and every category-nested skill are discovered. The CLI detects
your installed agents (or asks which to target); use `--list` to preview without installing, `-a <agent>` to
pick a specific agent, and `-g` for a user-wide install.

```bash
npx skills add uswuth/rustoria --full-depth --list
```

#### Manual install (Claude Code layout)

Copy the skills into your agent's skills directory. Agents discover skills as a
**directory bundle** (`<skill-name>/SKILL.md`), per the
[Agent Skills standard](https://agentskills.io):

```bash
# Copy the root router and all skills (Claude Code layout)
mkdir -p ~/.claude/skills/rustoria
cp SKILL.md ~/.claude/skills/rustoria/SKILL.md
for f in $(find skills -name SKILL.md); do
  name=$(grep '^name:' "$f" | head -1 | sed 's/name: *//')
  mkdir -p ~/.claude/skills/$name
  cp "$f" ~/.claude/skills/$name/SKILL.md
done
```

Or use the setup script:

```bash
bash scripts/setup.sh ~/.claude/skills
```

Other agents use different roots - see `.claude/INSTALL.md`, `.codex/INSTALL.md`,
`.gemini/INSTALL.md`, `.hermes/INSTALL.md`, `.antigravity/INSTALL.md`, and
`.opencode/INSTALL.md` for each agent's verified discovery path.

### References

References are deep-dive documents. Copy them alongside the skills:

```bash
mkdir -p ~/.claude/skills/references
cp references/*.md ~/.claude/skills/references/
```

## Usage

1. Load the root router (the `rustoria` skill, `SKILL.md`) for any Rust task.
2. Route to the skill matching the task.
3. Load the referenced deep-dive when the task needs it.

The router makes the priority explicit: **Rust language/std/Cargo fundamentals come first; libraries are specialized extensions.**

## Repository structure

```text
rustoria/
├── SKILL.md                    # Root router
├── skills/                     # 14 skills in 7 categories
│   ├── fundamentals/
│   ├── std/
│   ├── production/
│   ├── debugging/
│   ├── unsafe-checker/
│   ├── dependencies/
│   ├── security/           # rust-code-audit
│   ├── libraries/          # 7 library skills
├── references/                 # 12 deep-dive documents
├── examples/                   # 6 compile-tested examples
│   ├── std-file-processor/     # zero-dependency std-only
│   ├── minimal-api/
│   ├── modular-monolith/
│   ├── async-service/
│   ├── cli/
│   └── library/
├── scripts/
│   ├── validate.sh             # Structure/frontmatter/catalog/link/secret checks
│   ├── test-triggers.sh        # Trigger routing tests
│   └── setup.sh                # Installation helper
├── tests/
│   └── trigger-tests.md        # Trigger test case documentation
├── docs/
│   ├── architecture.md         # Skill system architecture
│   ├── functional-overview.md  # How the system works
│   └── planned-capabilities.md   # Deferred skills and capabilities
└── .github/workflows/          # CI: validate, markdown, links, release
```

## Examples

All examples compile and pass `cargo test` on stable Rust (edition 2021):

| Example | Demonstrates | Dependencies |
| --------- | ------------- | -------------- |
| `std-file-processor` | fs, io, iterators, custom errors | none |
| `minimal-api` | axum HTTP API | axum, tokio, serde |
| `modular-monolith` | feature-oriented module boundaries | axum, tokio, serde |
| `async-service` | tokio workers, scheduler, graceful shutdown | tokio, tokio-util |
| `cli` | clap subcommands with tests | clap, serde |
| `library` | validated newtypes with thiserror | thiserror, serde, regex |

## Validation

```bash
bash scripts/validate.sh       # structure, frontmatter, catalog, links, secrets, paths
bash scripts/test-triggers.sh  # trigger routing (positive/negative/ambiguous/composition)
```

CI runs these plus markdown linting, link checking, and per-example `cargo check`/`test`/`clippy`/`fmt`.

## Planned skills

The following skills are **intentionally not part of v1** and may be reintroduced later once their underlying crates
mature and their APIs can be verified against current official documentation:

- rust-topcoat — deferred until topcoat has stable releases and verified APIs
- rust-gpui-kit — deferred until gpui/gpui-kit have stable releases and verified APIs

See [`docs/planned-capabilities.md`](docs/planned-capabilities.md) for the full criteria.

## Agent integrations

This repository supports multiple AI coding agents through thin adapters. The canonical skill source is always `skills/`
— agent directories contain only integration glue.

| Agent | Installation |
| ------- | ------------- |
| Claude Code | See [.claude/INSTALL.md](.claude/INSTALL.md) |
| Codex | See [.codex/INSTALL.md](.codex/INSTALL.md) |
| Gemini CLI | See [.gemini/INSTALL.md](.gemini/INSTALL.md) |
| Hermes | See [.hermes/INSTALL.md](.hermes/INSTALL.md) |
| Antigravity | See [.antigravity/INSTALL.md](.antigravity/INSTALL.md) |
| OpenCode | See [.opencode/INSTALL.md](.opencode/INSTALL.md) |

See [AGENTS.md](AGENTS.md) for agent execution safety principles.

## Governance

- [DISCLAIMER.md](DISCLAIMER.md) — origin, no-guarantee, user responsibility
- [THIRD-PARTY-NOTICE.md](THIRD-PARTY-NOTICE.md) — referenced projects, trademark separation
- [CONTRIBUTING.md](CONTRIBUTING.md) — contribution rules, commit convention, PR requirements
- [CHANGELOG.md](CHANGELOG.md) — version history
- [docs/skill-quality-contract.md](docs/skill-quality-contract.md) — anti-skill-slop gate for all skills
- [docs/labels.md](docs/labels.md) — issue label taxonomy
- [docs/planned-capabilities.md](docs/planned-capabilities.md) — deferred skills and capabilities

## Ecosystem intelligence

This repository is designed to eventually monitor the Rust ecosystem and propose skill updates. See:

- [docs/ecosystem-intelligence.md](docs/ecosystem-intelligence.md) — pipeline architecture
- [docs/issue-automation.md](docs/issue-automation.md) — automatic issue generation design
- [docs/repository-integrity.md](docs/repository-integrity.md) — integrity checker design

## License

[MIT](LICENSE)
