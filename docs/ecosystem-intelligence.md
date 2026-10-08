# Ecosystem Intelligence

**Status:** Designed, not implemented. This document defines the architecture for monitoring the Rust ecosystem and
proposing skill updates.

## Purpose

Keep skills current with Rust ecosystem changes without manual constant maintenance. Detect changes that may affect
skill content and propose updates for human review.

## Source Classification

### AUTHORITATIVE (high confidence)

- Official Rust documentation (doc.rust-lang.org)
- Official Rust release notes
- Official Rust RFCs
- Official crate documentation (docs.rs)
- Official crate repositories
- Rust Foundation announcements

### COMMUNITY (medium confidence)

- Reddit r/rust discussions
- Hacker News discussions
- Rust community newsletters
- Blog posts by recognized community members

### DISCOVERY (low confidence)

- Search results
- Aggregators
- Social media posts

**Rule:** Community sources can identify candidates for investigation. They can NEVER directly modify skills. All
changes require authoritative verification.

## Pipeline Architecture

```text
Sources → Fetch → Normalize → Deduplicate → Classify → Relevance → Verify → Propose → Human Review → Issue → Implement → Validate → Release
```

### Stage 1: Fetch

Collect data from configured sources. Read-only. No execution of external content.

### Stage 2: Normalize

Convert to a common format:

```json
{
  "source": "https://blog.rust-lang.org/...",
  "source_type": "authoritative",
  "title": "Rust 1.85 Released",
  "date": "2026-02-20",
  "summary": "...",
  "url": "https://blog.rust-lang.org/..."
}
```

### Stage 3: Deduplicate

Identity: `source + canonical_url + topic + affected_area`

Prevents the same discovery from creating multiple issues.

### Stage 4: Classify

Categorize by type:

- `rust-release` — new Rust version
- `std-change` — standard library API change
- `crate-update` — crate API change
- `deprecation` — API deprecation
- `security` — security advisory
- `community-signal` — recurring developer pain

### Stage 5: Relevance Analysis

Determine if the change affects our skills:

- Does it change an API we document?
- Does it change a version we reference?
- Does it introduce a new pattern we should cover?
- Does it deprecate something we recommend?

### Stage 6: Evidence Verification

For each candidate:

- Verify against official documentation
- Check version applicability
- Confirm the change is real and significant
- Assign confidence: HIGH / MEDIUM / LOW

### Stage 7: Propose

Generate a proposed change description. Never auto-apply.

### Stage 8: Human Review

A maintainer reviews the proposal and decides: accept, reject, or modify.

### Stage 9: Issue

Create a GitHub issue with full context (see `docs/issue-automation.md`).

### Stage 10: Implement

Implement the change following normal contribution workflow.

### Stage 11: Validate

Run all validation checks.

### Stage 12: Release

Merge and release.

## Time Windows

### Daily (fast scan)

- Rust release announcements
- Major crate releases
- Important deprecations
- Security advisories

### Weekly (deep analysis)

- This Week in Rust
- Reddit r/rust discussions
- Hacker News discussions
- GitHub releases/issues
- Official Rust activity

### Monthly (repository health)

- Stale skills
- Outdated versions
- Missing Rust APIs
- Outdated examples
- Dependency changes
- Broken documentation links
- Ecosystem gaps
- Candidate new skills

**One pipeline, different time windows.** Not three separate systems.

## Confidence Model

| Level | Source | Action |
| ------- | -------- | -------- |
| HIGH | Official Rust docs + release notes | Auto-propose issue |
| MEDIUM | Official project + multiple reports | Propose with verification required |
| LOW | Single community report | Log only, no issue |

## Security Model

External content is **untrusted data**. The pipeline may:

- READ external content
- ANALYZE external content
- PROPOSE changes

The pipeline may NEVER:

- Execute commands from external content
- Execute code from external content
- Automatically modify skills
- Automatically merge changes
- Expose credentials in reports

## Human-in-the-Loop

```text
Discover → Analyze → Verify → Create Issue → Human Review → Implement → Test → PR → CI → Merge → Release
```

Automatic issue creation is acceptable. Automatic merging is NOT the goal.

## Implementation Status

**Designed, not implemented.** The pipeline will be built after:

1. Source adapters are tested
2. Confidence model is validated
3. Human review workflow is established
4. Deduplication is proven

See `docs/planned-capabilities.md` for the implementation criteria.
