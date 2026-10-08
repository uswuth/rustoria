# rustoria Architecture

How the skill system is structured and why.

## Design goals

- **Native Rust first.** Language, std, and Cargo/toolchain knowledge is the foundation; crates are extensions.
- **Reference, not tutorial.** Skills are concise decision documents, not courses.
- **Discoverable.** Every skill is routed from the root and listed in the README; every reference is linked from a skill.
- **Validated.** Structure, frontmatter, catalog consistency, links, secrets, and trigger routing are machine-checked.
- **Bounded.** 14 skills, 12 references, 6 examples. Every component justifies its existence.
- **Three concepts, not one.** Skills (agent knowledge), references (deep technical material), and
  intelligence/maintenance (ecosystem monitoring, validation, CI/CD) are distinct. Do not conflate them.

## Structure

```text
rustoria/
├── SKILL.md                 # Router: routes to skills, orients on the ecosystem
├── skills/
│   ├── fundamentals/        # rust-fundamentals — language foundation
│   ├── std/                 # rust-std — std API discovery
│   ├── production/          # rust-production — production engineering
│   ├── debugging/           # rust-debugging — debugging
│   ├── unsafe-checker/      # rust-unsafe-checker — unsafe review
│   ├── dependencies/        # rust-dependencies — deps/semver/MSRV
│   └── libraries/           # 7 library skills (extensions)
├── references/              # 12 deep-dive documents
├── examples/                # 6 compile-tested examples
├── scripts/                 # validate.sh, test-triggers.sh, setup.sh
├── tests/                   # trigger-tests.md
└── docs/                    # architecture, functional-overview, planned-capabilities
```

## Skill categories

### Native core (6 skills)

The foundation. These skills make the repository useful even with zero third-party crates:

- `rust-fundamentals` — language: ownership, borrowing, lifetimes, types, traits, generics, macros
- `rust-std` — standard library: collections, iterators, strings, fs/io, process, net, sync, conversion traits
- `rust-production` — engineering: architecture, errors, concurrency, performance, testing, observability,
  native-first dependency rule
- `rust-debugging` — compiler errors, runtime debugging, common bugs
- `rust-unsafe-checker` — unsafe Rust, FFI, raw pointers, Miri
- `rust-dependencies` - Cargo, SemVer, MSRV, features, Cargo.lock policy

### Security (1 skill)

Repository-wide security analysis that complements (not duplicates) `rust-unsafe-checker`, `rust-dependencies`, and `rust-production`:

- `rust-code-audit` - layered repository audit: secret detection, Git/history exposure, dependency and supply-chain
  review, data-flow reasoning (source -> trust boundary -> validation -> sink), CI/CD and build-script review,
  severity/confidence finding model with secret redaction

### Library extensions (7 skills)

Specialized crates that build on the native foundation:

- `rust-tokio` — async runtime
- `rust-axum` — HTTP
- `rust-serde` — serialization
- `rust-diesel` — databases
- `rust-rayon` — parallelism
- `rust-clap` — CLI
- `rust-ratatui` — TUI

## Routing model

1. The root `SKILL.md` is the entry point for any Rust task.
2. It routes to the skill matching the task (catalog table).
3. Skills point to `references/` for deep dives.
4. The router states the priority: native fundamentals first, libraries as extensions.

## Reference model

References are deep-dive documents on specific topics. They are:

- **Reachable** — linked from the skills that need them.
- **Authoritative** — native Rust claims cite first-party sources (std docs, Rust Book, Reference, Rustonomicon, Cargo Book).
- **Bounded** — each covers one topic concisely; no filler.

## Example model

Examples are small, compile-tested Rust projects. They:

- Prove the guidance compiles and tests green.
- Demonstrate a pattern taught by a skill.
- Use edition 2021 and stable Rust.
- Have zero or minimal, justified dependencies.
- Include one zero-dependency std-only example (`std-file-processor`).

## Validation model

Two harnesses run on every push:

- `scripts/validate.sh` — structure, frontmatter, name↔directory consistency, skill count, README catalog
  consistency, router coverage, internal links, placeholders, secret files, personal paths, example manifests.
- `scripts/test-triggers.sh` — trigger routing: positive, negative, ambiguous near-miss, and composition cases
  against real SKILL.md descriptions.

CI additionally runs markdown linting, link checking, and per-example `cargo check`/`test`/`clippy`/`fmt`.

## Extension model

To add a skill:

1. Create `skills/<category>/<name>/SKILL.md` with valid frontmatter.
2. Route it in the root `SKILL.md` and list it in the README.
3. Link relevant references.
4. Run `scripts/validate.sh` and `scripts/test-triggers.sh` — both must pass.

To add a reference:

1. Create `references/<topic>.md`.
2. Link it from at least one skill.
3. Ensure it is technically accurate against current official documentation.

To add an example:

1. Create `examples/<name>/` with a `Cargo.toml`.
2. Ensure `cargo check`, `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` pass.
3. Link it from the root `SKILL.md` and README.

## Deferred skills

Skills that are intentionally not part of v1 (e.g. `rust-topcoat`, `rust-gpui-kit`) are documented in
`docs/planned-capabilities.md` with explicit reintroduction criteria.
