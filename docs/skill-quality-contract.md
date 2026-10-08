# Skill Quality Contract

Every skill in this repository must satisfy this contract. It is the anti-skill-slop gate.

## Skill Quality Gate

A skill is accepted only if it passes all checks:

- [ ] **Clear single responsibility** — one topic, one purpose
- [ ] **No duplicate existing skill** — distinct from all other skills
- [ ] **Native Rust considered first** — std before crates
- [ ] **Official documentation verified** — API claims cite official sources
- [ ] **Rust/edition/MSRV specified** where relevant
- [ ] **Dependency versions verified** where relevant
- [ ] **Code examples compile** when practical
- [ ] **No fabricated APIs** — every API call is real
- [ ] **No contradictory guidance** — consistent with other skills
- [ ] **No unnecessary abstraction** — KISS/DRY/YAGNI applied
- [ ] **Production implications explained** — not just "how" but "when/why"
- [ ] **Security implications considered** — input validation, unsafe, secrets
- [ ] **Trigger tests exist** — positive and negative cases
- [ ] **References are discoverable** — linked from the skill
- [ ] **No AI-generated filler** — no motivational writing, no empty claims
- [ ] **No unsupported claims** — every statement is verifiable
- [ ] **Maintainer can explain why this skill exists** — clear justification

## Three Concepts — Not One

This repository has three distinct concepts. Do not conflate them.

### 1. Skills (`skills/`)

Knowledge used by the agent during a task. Concise, reference-oriented, actionable.

### 2. References (`references/`)

Deep technical source material. Reachable from skills. Authoritative, version-aware.

### 3. Intelligence/Maintenance (`docs/`, `scripts/`, `.github/`)

Things that keep the repository current. Ecosystem monitoring, validation, CI/CD.

**Do not turn ecosystem monitoring into another skill.** That would recreate the skill inflation this repository
deliberately avoids.

## Scoring

| Criterion | Weight | Question |
| ----------- | -------- | ---------- |
| User value | 1-5 | Does this solve a real problem? |
| Rust relevance | 1-5 | Is this core Rust knowledge? |
| Distinct responsibility | 1-5 | Is this clearly separate from other skills? |
| Evidence availability | 1-5 | Can this be verified against official sources? |
| Maintenance cost | 1-5 | How much ongoing work to keep accurate? |
| Duplication risk | 1-5 | How likely to overlap with existing skills? |

**Minimum score: 20/30.** Skills below 20 are not added. High duplication risk is an automatic reject.

## The Maintainer Test

Before adding any skill, a maintainer must be able to answer:

> Why does this skill exist?

If the answer is "because another repository has it" or "because we might need it someday," the skill does not exist yet.
