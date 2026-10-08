# Planned Capabilities

Capabilities that are **intentionally not implemented** in the current version. Each entry documents the purpose,
criteria for implementation, and reason for deferral.

## Deferred Skills

### rust-topcoat

- **Purpose:** Full-stack web framework guidance
- **Proposed scope:** Routing, views, state management, deployment
- **Dependencies:** topcoat crate with stable API
- **Official sources:** https://github.com/topcoat-rs/topcoat
- **Status:** Deferred
- **Reason:** Crate is early-stage; API not stable enough for accurate guidance
- **Criteria:** Stable release, verified API, compile-tested examples

### rust-gpui-kit

- **Purpose:** Desktop GUI application guidance
- **Proposed scope:** Window management, widgets, event handling, layout
- **Dependencies:** gpui-kit crate with stable API
- **Official sources:** https://github.com/longbridge/gpui-kit
- **Status:** Deferred
- **Reason:** API could not be verified against current documentation
- **Criteria:** Stable release, verified API, compile-tested examples

## Deferred Capabilities

### Ecosystem Intelligence Pipeline

- **Purpose:** Automated monitoring of Rust ecosystem changes
- **Proposed scope:** Source classification, evidence verification, issue generation
- **Status:** Designed (see `docs/ecosystem-intelligence.md`), not implemented
- **Reason:** Requires reliable source adapters and verification before automation
- **Criteria:** Source adapters tested, confidence model validated, human review workflow established

### Dynamic Skill Generation

- **Purpose:** Generate crate-specific skills from Cargo.toml dependencies
- **Proposed scope:** Parse dependencies, fetch docs, generate version-aware skill
- **Status:** Rejected for now
- **Reason:** High risk of hallucinated APIs; bypasses human review
- **Criteria:** Source verification, version awareness, human review, validation — all mandatory

### Code Navigation Skill

- **Purpose:** Rust code analysis — symbol search, call graphs, trait exploration
- **Status:** Rejected
- **Reason:** Agent built-in capability; low distinct value
- **Criteria:** N/A

### Symbol Analysis Skill

- **Purpose:** Deep Rust symbol and type analysis
- **Status:** Rejected
- **Reason:** Agent built-in capability; low distinct value
- **Criteria:** N/A

### Deferred Agent Adapters

- **Purpose:** Additional agent integration directories (Cursor, GitHub Copilot, VS Code, Amp, Goose, ...)
- **Status:** Deferred
- **Research:** Investigated against the [Agent Skills client showcase](https://agentskills.io/clients.md).
  The Agent Skills standard is supported by a long tail of editors and agents; the six adapters in this
  repository (Claude Code, Codex, Gemini CLI, Hermes, Antigravity, OpenCode) are the ones with a verified,
  cited discovery path in their `INSTALL.md`. The remaining standard-supporting tools would be mechanical
  copies of the same `<name>/SKILL.md` bundle instructions.
- **Reason:** Each adapter must be verified against that agent's current official documentation (path and
  directory structure differ per agent, and unverified paths silently produce undiscoverable installs).
  Adapters for tools this repository's maintainers do not use cannot be kept verified as those tools
  evolve, so they would rot into false claims - the failure mode found and fixed in the six existing
  adapters during the hardening pass.
- **Criteria:** Add an adapter only when (1) the agent is genuinely used by a maintainer, (2) its current
  official docs are checked and the exact discovery path/structure is cited in `INSTALL.md`, (3) the file
  stays a thin install guide, and (4) its discovery path is re-verified whenever that agent's docs change.

## Evaluation Criteria for Future Skills

Any new skill must score at least 20/25 on:

| Criterion | Weight |
| ----------- | -------- |
| User value | 1-5 |
| Rust relevance | 1-5 |
| Distinct responsibility | 1-5 |
| Evidence availability | 1-5 |
| Maintenance cost (inverse) | 1-5 |
| Duplication risk (inverse) | 1-5 |

Skills scoring below 20 are not added. Skills with high duplication risk are rejected regardless of total score.
