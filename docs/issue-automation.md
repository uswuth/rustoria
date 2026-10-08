# Issue Automation

**Status:** Designed, not implemented. This document defines the architecture for automatic issue generation from
ecosystem intelligence.

## Purpose

Automatically create high-quality GitHub issues when the Rust ecosystem changes in ways that may affect our skills.

## Issue Quality Standard

### Bad issue

> Rust updated. Please check.

### Good issue

Every claim must be independently checkable. Example (a change this repository actually received):

```markdown
## Summary

`serde_yaml` is deprecated on crates.io (latest release `0.9.34+deprecated`), so YAML
guidance that recommends it is stale.

## Evidence

- Official source: https://crates.io/crates/serde_yaml (crate description marks it deprecated)
- Affected content: `skills/libraries/rust-serde/SKILL.md` (YAML warning section)

## Detected impact

Potentially affects:

- `skills/libraries/rust-serde` — recommends a deprecated crate

## Current repository state

- `rust-serde` warns about `serde_yaml` deprecation
- No example in this repository depends on `serde_yaml` or `serde_yml`
  (checked: no match in any `examples/*/Cargo.toml`, `Cargo.lock`, or source)

## Recommended investigation

- Verify the deprecation status against crates.io directly
- Update guidance to prefer JSON/TOML; note the successor crate only after
  checking its own maintenance status

## Confidence

HIGH (crates.io crate metadata, checked directly)

## Source type

Authoritative (crates.io registry)

## Proposed action

Update (rust-serde YAML guidance)
```

Rules the example demonstrates: cite the primary source (never a secondary summary), quote the exact
version or claim being verified, state the repository's actual current state (checked, not assumed), and
never invent version numbers, release dates, or URLs.

## Issue Types

| Type | Label | Example |
| ------ | ------- | --------- |
| Rust release | `type:maintenance`, `area:rust-core` | "Review Rust X.Y feature changes for skill coverage" |
| Std change | `type:skill`, `area:std` | "Review std API change: LazyLock stabilization" |
| Crate update | `type:skill`, `area:library` | "Review Axum X.Y API changes" |
| Deprecation | `type:maintenance`, `area:library` | "Review deprecation of serde_yaml" |
| Security | `type:security` | "Review security advisory for dependency X" |
| Community signal | `type:maintenance`, `area:rust-core` | "Recurring developer pain: async cancellation" |

## Deduplication

Identity: `source + canonical_url + topic + affected_area`

Before creating an issue, check if an issue with the same identity already exists. If so, update the existing issue
instead of creating a new one.

## Automatic vs Manual

| Action | Automatic | Manual |
| -------- | ----------- | -------- |
| Fetch external content | ✅ | — |
| Classify and analyze | ✅ | — |
| Verify against official docs | ✅ | — |
| Create issue | ✅ | — |
| Implement skill change | — | ✅ |
| Merge PR | — | ✅ |
| Release | — | ✅ |

**Automatic issue creation is acceptable. Automatic merging is NOT the goal.**

## Label Taxonomy

### Types

- `type:bug` — incorrect information
- `type:feature` — new capability
- `type:skill` — skill content change
- `type:documentation` — documentation change
- `type:maintenance` — maintenance task
- `type:security` — security issue

### Areas

- `area:rust-core` — Rust language/compiler
- `area:std` — standard library
- `area:unsafe` — unsafe Rust
- `area:cargo` — Cargo/toolchain
- `area:library` — third-party crates
- `area:agent` — agent integration
- `area:automation` — automation/CI
- `area:ci` — CI/CD
- `area:docs` — documentation

### Priority

- `priority:critical` — blocking issue
- `priority:high` — important
- `priority:medium` — normal
- `priority:low` — minor

### Status

- `status:triage` — needs review
- `status:planned` — accepted, not started
- `status:blocked` — blocked by dependency
- `status:in-progress` — being worked on

### Special

- `good-first-issue` — suitable for new contributors
- `help-wanted` — needs community help
- `breaking-change` — breaks compatibility
- `deferred` — intentionally postponed

## Implementation Status

**Designed, not implemented.** Will be built after the ecosystem intelligence pipeline is reliable.
