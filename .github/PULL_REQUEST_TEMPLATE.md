<!-- markdownlint-disable MD041 -->
## What changed?

<!-- Describe what this PR adds, changes, or removes. -->

## Why?

<!-- Explain the motivation. What problem does this solve? -->

## Skills affected

<!-- List any skills that are added, modified, or removed. -->

## Rust/API verification

<!-- For any Rust API changes, fill this out. Delete if not applicable. -->

- **Rust version:**
- **Edition:**
- **Dependency version:**
- **Official documentation checked:**
- **Example compile-tested:**

## Validation

There is no root `Cargo.toml` — each example is its own crate. Run these per example:

- [ ] `cargo fmt --check --manifest-path examples/<name>/Cargo.toml` passes
- [ ] `cargo check --manifest-path examples/<name>/Cargo.toml` passes
- [ ] `cargo test --manifest-path examples/<name>/Cargo.toml` passes
- [ ] `cargo clippy --manifest-path examples/<name>/Cargo.toml --all-targets -- -D warnings` passes
- [ ] `bash scripts/validate.sh` passes
- [ ] `bash scripts/test-triggers.sh` passes

## Documentation

- [ ] README updated if skill inventory changed
- [ ] CHANGELOG updated if user-visible
- [ ] Trigger tests updated if routing changed

## Breaking changes

<!-- None / describe -->
