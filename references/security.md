# Security Practices in Rust

## Overview

Rust's memory safety guarantees eliminate entire classes of vulnerabilities (buffer overflows, use-after-free,
double-free). However, security is a broad discipline — input validation, secrets management, and dependency auditing
remain critical.

---

## Input Validation

### Validate at the boundary

Validate all external input at the system boundary — HTTP handlers, CLI arguments, file parsing.

```rust
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateUserRequest {
    #[serde_as(as = "DisplayFromStr")] // serde_with 3.x — parses via FromStr
    email: Email,
    name: String,
}

// The type system enforces validation
struct Email(String);

impl std::str::FromStr for Email {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('@') && s.len() <= 254 {
            Ok(Self(s.to_string()))
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }
}
```

Deserialization of `email` now fails unless `Email::from_str` accepts it — the
validated type is enforced at the boundary.

### Sanitize, don't just validate

```rust
// Bad: only checks, doesn't sanitize
fn bad(input: &str) -> String {
    if input.contains('<') {
        return String::new();
    }
    input.to_string()
}

// Good: escape HTML entities
fn good(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

### Use `validator` crate

```rust
use validator::Validate;

#[derive(Validate)]
struct SignupForm {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8, max = 128))]
    password: String,

    #[validate(range(min = 13, max = 120))]
    age: u8,
}

match form.validate() {
    Ok(_) => { /* proceed */ }
    Err(errors) => { /* return validation errors */ }
}
```

---

## Secrets Management

### Never hardcode secrets

```rust
// Bad: secret in source code
let api_key = "sk-1234567890abcdef";

// Good: load from environment
let api_key = std::env::var("API_KEY")
    .expect("API_KEY environment variable not set");
```

### Use `secrecy` for zeroization

```rust
use secrecy::{ExposeSecret, SecretString};

let api_key = SecretString::new("my-api-key".into());
// Debug output is redacted ("[REDACTED]") and the memory is zeroized on drop.
// Access the plaintext only where it's actually needed:
let header = format!("Bearer {}", api_key.expose_secret());
```

### Use `dotenvy` for local development

```rust
dotenvy::dotenv().ok(); // loads .env file — dev convenience only, never in prod

let db_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL not set");
```

### Environment-variable caveats

Environment variables are the simplest secret channel, but they leak easily: a
process's environment is inherited by its children, is often readable by
same-user processes (`/proc/<pid>/environ` on Linux), and frequently ends up in
crash reports and logs. Prefer secrets delivered as **files** mounted by your
orchestrator (Kubernetes secrets, Docker secrets) or fetched from a secret
manager, and never log secret values or include them in error messages.

### Secret management in production

- **Environment variables** — simplest, works with containers.
- **Secret managers** — AWS Secrets Manager, HashiCorp Vault, Azure Key Vault.
- **Encrypted config files** — `age`, `sops`.

```rust
// Example: loading from AWS Secrets Manager
use aws_sdk_secretsmanager::Client;

async fn get_secret(client: &Client, name: &str) -> Result<String, Error> {
    let resp = client
        .get_secret_value()
        .secret_id(name)
        .send()
        .await?;

    Ok(resp.secret_string().unwrap_or_default().to_string())
}
```

---

## Dependency Auditing

### `cargo audit`

```bash
cargo install cargo-audit
cargo audit
```

This checks your dependencies against the RustSec Advisory Database.

### `cargo deny`

```bash
cargo install cargo-deny
cargo deny check
```

`cargo deny` provides more comprehensive checks:

- License compliance
- Security advisories
- Banned crates
- Duplicate dependencies

```toml
# deny.toml - keys valid for cargo-deny 0.20.x (verify against the cargo-deny book
# for your pinned version; the advisories section has churned across releases)
[advisories]
unmaintained = "all"     # report unmaintained crates: all | workspace | transitive | none
yanked = "warn"
# ignore = [{ advisory-id = "...", reason = "..." }]   # with an explicit reason

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC", "Zlib"]
# unlicensed/confidence thresholds are not valid keys - unknown keys are rejected

[bans]
multiple-versions = "warn"
```

Failure severity is controlled by CLI flags (`cargo-deny check --deny warnings`), not by removed config keys.

### Keep dependencies up to date

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update
```

### Minimize dependencies

- Audit your `Cargo.toml` — remove unused dependencies.
- Prefer small, focused crates over large frameworks.
- Use `cargo tree` to understand your dependency graph.

```bash
cargo tree
cargo tree -d  # show duplicates
```

---

## SQL Injection Prevention

### Use parameterized queries

```rust
// Bad: string formatting
let query = format!("SELECT * FROM users WHERE id = {}", user_id);

// Good: parameterized query
let user = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE id = $1"
)
.bind(user_id)
.fetch_one(&pool)
.await?;
```

### Use an ORM or query builder

```rust
// Diesel — compile-time checked queries
use diesel::prelude::*;

let user = users::table
    .filter(users::id.eq(user_id))
    .first::<User>(&conn)?;

// SeaORM — async ORM
let user = User::find_by_id(user_id)
    .one(&db)
    .await?;
```

### When NOT to use string concatenation

Never build SQL queries with user input. Even "internal" data can be an attack vector if it originated from user input.

---

## Path Traversal Prevention

### Validate file paths

```rust
use std::path::{Path, PathBuf};

fn safe_path(base: &Path, user_input: &str) -> Result<PathBuf, Error> {
    let path = base.join(user_input);

    // Canonicalize and verify the path is under base
    let canonical = path.canonicalize()?;
    let canonical_base = base.canonicalize()?;

    if !canonical.starts_with(&canonical_base) {
        return Err(Error::PathTraversal);
    }

    Ok(canonical)
}
```

### Use `camino` for UTF-8 paths

```rust
use camino::Utf8Path;

fn process(path: &Utf8Path) -> Result<(), Error> {
    if path.as_str().contains("..") {
        return Err(Error::InvalidPath);
    }
    // ...
}
```

### Common pitfalls

```rust
// Bad: user input directly in path
let path = format!("/uploads/{}", user_input);

// Bad: only checking for ".."
if user_input.contains("..") {
    return Err(Error::InvalidPath);
}
// Real bypass classes for a contains("..") check:
//  - URL percent-encoding: "%2e%2e%2f%2e%2e%2fetc/passwd" — decode BEFORE validating
//  - absolute paths: "/etc/passwd" contains no "..", and
//    Path::join("/uploads", "/etc/passwd") yields "/etc/passwd" — an absolute
//    argument replaces the base entirely!
//  - symlinks: /uploads/link -> /etc passes every string check; only
//    canonicalization reveals the real location

// Good: canonicalize (resolves symlinks and "..") and verify the resolved path
// stays under the canonicalized base, as in safe_path above
let path = safe_path(Path::new("/uploads"), user_input)?;
```

---

## Additional Security Practices

### Use `rustls` instead of `native-tls`

```toml
# Prefer rustls — pure Rust, no OpenSSL dependency
reqwest = { version = "0.13", default-features = false, features = ["rustls-tls"] }
```

### Constant-time comparison for secrets

```rust
use subtle::ConstantTimeEq;

fn verify_token(provided: &str, expected: &str) -> bool {
    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}
```

### Rate limiting

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

let limiter = RateLimiter::direct(
    Quota::per_second(NonZeroU32::new(10).unwrap())
);

if limiter.check().is_err() {
    return Err(Error::RateLimited);
}
```

### Sandboxing and syscall filtering

For high-security applications, restrict what the process can do at the OS level:
`seccompiler` (seccomp-bpf syscall filters, Linux only), `landlock` (filesystem
scoping, Linux 5.13+), `sandbox-exec` (macOS), or AppContainer (Windows). These
APIs are platform-specific, must usually be applied after initialization, and
complement — not replace — input validation: they limit the blast radius of a
compromise.

---

## Integer Overflow Semantics

- **Debug builds:** arithmetic overflow **panics** (overflow checks on).
- **Release builds:** overflow **wraps** silently (two's complement), unless
  `overflow-checks = true` is set in the profile.

Silent wrapping is a real vulnerability class (think balance calculations, buffer
sizes). Pick the explicit operation instead of relying on build-mode behavior:

```rust
let x = 250u8;

x.checked_add(10)      // None — overflow reported, nothing wrapped
x.wrapping_add(10)     // 4 — deliberate wraparound (hashing, protocols)
x.saturating_add(10)   // 255 — clamps at the type's bound
x.overflowing_add(10)  // (4, true) — value plus overflow flag
```

If wrapping in release is never acceptable for your domain, turn the checks on:

```toml
[profile.release]
overflow-checks = true
```

Also beware `as` casts: `300u32 as u8` truncates silently to 44. Prefer
`u8::try_from(x)` where a lossy conversion would be a bug.

---

## Unsafe Code Policy

Safe Rust's guarantees end at `unsafe`. For application code, forbid it outright
at the crate root:

```rust
#![forbid(unsafe_code)] // lib.rs or main.rs — any unsafe block is a compile error
```

Libraries that genuinely need `unsafe`: isolate it in a small module, document
the invariants that make each block sound, and audit unsafe across the dependency
tree with `cargo geiger`.

---

## Command Argument Injection

Never assemble shell strings from user input — and don't spawn a shell at all
unless you must:

```rust
use std::process::Command;

// Bad: user input inside a shell string — `x; rm -rf /` runs BOTH commands
Command::new("sh")
    .arg("-c")
    .arg(format!("convert {} out.png", user_filename));

// Good: no shell; arguments passed as an array. The OS hands each argument
// to the program literally — it can never be parsed as shell syntax.
Command::new("convert")
    .arg(user_filename)
    .arg("out.png")
    .status()?;
```

---

## Security Checklist

- [ ] Validate all external input at the boundary.
- [ ] Use parameterized queries or an ORM.
- [ ] Never hardcode secrets — use files or a secret manager; treat env vars as a fallback.
- [ ] Zeroize secrets in memory (`secrecy`) and never log them.
- [ ] Run `cargo audit` regularly.
- [ ] Use `cargo deny` for license and dependency checks.
- [ ] Canonicalize and verify file paths (prefix check after canonicalization).
- [ ] Use constant-time comparison for secrets.
- [ ] Pass command arguments as arrays, never as shell strings.
- [ ] Use checked/saturating arithmetic where silent wrap would be a vulnerability.
- [ ] Implement rate limiting for public APIs.
- [ ] Keep dependencies up to date.
- [ ] Minimize dependencies — audit `Cargo.toml`.

---

## Summary

| Practice | Tool/Crate | When |
| ---------- | ------------ | ------ |
| Input validation | `validator`, custom types | All external input |
| Secrets management | `secrecy`, `dotenvy` (dev only) | All environments |
| Dependency auditing | `cargo audit`, `cargo deny` | CI/CD, regularly |
| SQL injection prevention | `sqlx`, `diesel`, `sea-orm` | All database access |
| Path traversal prevention | `camino`, canonicalization | All file operations |
| Constant-time comparison | `subtle` | Token/password verification |
| Rate limiting | `governor` | Public APIs |
| Unsafe audit | `cargo geiger`, `#![forbid(unsafe_code)]` | CI/CD |
