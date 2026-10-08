# Security Policy

## Supported Versions

Only the latest `main` branch is supported with security fixes. Tagged releases, when made, receive fixes for the
release they were cut from.

## Reporting a Vulnerability

Please report security vulnerabilities privately:

1. Use the repository's **GitHub Security Advisories** feature (Security tab → Advisories → "Report a vulnerability").
2. Do **not** open a public issue for security reports.
3. Include: affected component, description of the issue, steps to reproduce, and potential impact.

You will receive an acknowledgment within 7 days. We will investigate, develop a fix, and coordinate disclosure timing
with you.

## Scope

This repository is a collection of Markdown skill documents and example Rust code. The primary security concerns are:

- Secrets or personal paths accidentally committed to the repository.
- Example code that teaches unsafe or insecure patterns.
- Malicious or vulnerable dependencies in example `Cargo.toml` files.

The `scripts/validate.sh` harness checks for secrets, personal paths, and placeholder content on every push.
