# Repository Integrity

**Status:** Partially implemented. `scripts/validate.sh` covers many of these checks. This document defines the full
integrity model.

## Integrity Checks

### Skill Integrity

- [x] Advertised skill exists in filesystem
- [x] Skill exists but is indexed in README
- [x] Skill name matches directory name
- [x] No duplicate skill names
- [x] No orphan SKILL.md (all routed from root)
- [x] No phantom routes in root SKILL.md
- [x] Description length ≤ 100 chars
- [x] Description ends with period

### Content Integrity

- [x] No placeholders (org-name, contact-method, task-tracker markers)
- [x] No personal filesystem paths
- [x] No secrets (.env, .key, .pem)
- [x] No build artifacts (target/)
- [x] Internal links valid
- [x] No phantom skill references in catalogs

### Example Integrity

- [x] All examples have Cargo.toml
- [ ] All examples compile (`cargo check`)
- [ ] All examples pass tests (`cargo test`)
- [ ] All examples pass clippy (`cargo clippy -- -D warnings`)
- [ ] All examples pass fmt (`cargo fmt --check`)

### Documentation Integrity

- [x] README skill count matches filesystem
- [x] CHANGELOG describes actual changes
- [x] CONTRIBUTING describes actual process
- [x] No stale references to removed skills

### Trigger Integrity

- [x] Trigger tests pass (44/44 case tests + router assertions)
- [x] No phantom skills in trigger tests
- [x] Ambiguous near-miss cases exist
- [x] Cases parsed from `tests/trigger-tests.md` (docs cannot drift from executed tests)

### Agent Integrity

- [x] All agent INSTALL.md files are valid (paths verified against official docs:
  code.claude.com/docs/en/skills, developers.openai.com/codex/skills,
  geminicli.com/docs/cli/skills, hermes-agent.nousresearch.com docs,
  antigravity.google/docs/skills, opencode.ai/docs/skills)
- [x] Agent directories don't duplicate skill content (INSTALL.md files contain
  install steps only; skills live solely in `skills/`)
- [x] Agent adapters point to canonical source (each INSTALL.md cites its official docs URL)
- [x] Install layout matches discovery rules (directory bundle `<name>/SKILL.md`,
  the Agent Skills standard - verified by running `scripts/setup.sh`; CI-verified)

## Automated Checks

### Currently Automated (`scripts/validate.sh`)

- Skill count, frontmatter, name↔dir, duplicates, orphans, phantoms (frontmatter also gets strict-YAML
  validation: quote-aware description checks plus a PyYAML parse of every frontmatter when python3 is
  available - the exact rejection an Agent Skills consumer would perform)
- README and AGENTS.md catalog consistency
- Router coverage (both directions: missing routes and dead entries)
- Internal links (file existence, including anchored links)
- Placeholders, secrets, personal paths
- Example manifests
- Reference count
- Workflow validity (explicit `permissions:`, SHA-pinned actions,
  no untrusted `${{ }}` interpolation inside `run:` blocks)

### Currently Automated (`scripts/test-triggers.sh`)

- Positive trigger cases (30)
- Negative trigger cases (10)
- Composition cases (4)
- Root router assertions (router names `rustoria`, routes every skill, no dead entries)

### Currently Automated (CI)

- Per-example cargo check/test/clippy/fmt
- Markdown lint
- Link checking
- Skill install layout (`setup.sh` into a temp dir; expected bundle count derived from the
  skill inventory; fails on flat `*-SKILL.md` installs)

### Not Yet Automated

- Agent adapter validation
- Stale version detection
- Ecosystem change detection
- Duplicate responsibility detection

## Future: Repository Monitor

A monthly integrity report should detect:

- Skills not updated in 6+ months
- Examples with outdated dependencies
- References with broken links
- Version claims that may be stale
- Skills with overlapping descriptions
- Missing trigger tests for skills

## Design Principles

1. **Every advertised skill must exist.** Caught the original 25-vs-18 problem.
2. **Every skill must be reachable.** No orphans.
3. **Every claim must be verifiable.** Version-sensitive advice names its version.
4. **Every example must compile.** No exceptions.
5. **Every change must be traceable.** CHANGELOG + commit convention.
