# Architecture Patterns in Rust

## Overview

Rust's module system and crate ecosystem enable clear separation of concerns. Good architecture makes code testable,
maintainable, and reusable.

---

## Feature-Based Organization

Organize code by feature rather than by layer. Each feature is a self-contained module with its own types, logic, and tests.

```text
src/
  lib.rs
  config/
    mod.rs
    loader.rs
    validation.rs
  auth/
    mod.rs
    middleware.rs
    token.rs
  users/
    mod.rs
    repository.rs
    service.rs
  orders/
    mod.rs
    repository.rs
    service.rs
```

### Benefits

- **High cohesion** — related code lives together.
- **Easy navigation** — find all user code in `users/`.
- **Clear dependencies** — features depend on each other through well-defined interfaces.

### When NOT to use feature-based organization

- Small projects (< 5 modules) — a flat structure is simpler.
- When features are deeply intertwined — consider domain-driven design instead.

---

## Domain Boundaries

Define clear boundaries between domains. Each domain exposes a public API and hides implementation details.

```rust
// src/users/mod.rs — facade: declares submodules and re-exports their public API.
// The types themselves are defined in the submodule files, not here.
mod repository;
mod service;

pub use repository::UserRepository;
pub use service::UserService;
```

```rust
// src/users/service.rs
use super::repository::UserRepository;

// Generic over the repository — static dispatch, easy to test with fakes.
// (For dynamic dispatch, `Box<dyn UserRepository>` requires a dyn-compatible
// trait — see "Plugin Architecture" below for why native `async fn` needs care.)
pub struct UserService<R: UserRepository> {
    repo: R, // private — not accessible outside this module
}
```

### Dependency direction

Dependencies should point inward:

```text
presentation (HTTP handlers)
    ↓
application (services)
    ↓
domain (business logic)
    ↓
infrastructure (database, external APIs)
```

The domain layer should NOT depend on infrastructure. Use traits to invert dependencies:

```rust
// domain layer
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepoError>;
}

// infrastructure layer
pub struct PostgresUserRepository { pool: PgPool }

impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepoError> {
        // SQL implementation
    }
}
```

> **Note:** a trait with a native `async fn` is **not dyn-compatible** — `Box<dyn UserRepository>`
> will not compile ("the trait `UserRepository` is not dyn compatible"). Generic parameters
> (`R: UserRepository`) are unaffected. If you need `dyn`, see the three options under
> "Plugin Architecture" below.

---

## Module Structure

### `mod.rs` vs named files

Both styles work. Choose one and be consistent:

```rust
// Style 1: mod.rs
src/
  users/
    mod.rs       // module definition + re-exports
    service.rs
    repository.rs

// Style 2: named module file
src/
  users.rs       // module definition + re-exports
  users/
    service.rs
    repository.rs
```

### Re-exports

Control what's public through re-exports:

```rust
// src/users/mod.rs
mod repository;
mod service;

pub use service::UserService;
pub use repository::{UserRepository, PostgresUserRepository};

// External code uses:
// use mycrate::users::UserService;
// NOT: use mycrate::users::service::UserService;
```

### Visibility modifiers

| Modifier | Meaning |
| ---------- | --------- |
| `pub` | Visible everywhere |
| `pub(crate)` | Visible within the crate |
| `pub(super)` | Visible to parent module |
| `pub(in path::to::module)` | Visible to specific module |
| (none) | Private to current module |

---

## When to Use Workspaces

A Cargo workspace manages multiple related crates in one repository.

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/my-app",
    "crates/my-domain",
    "crates/my-infrastructure",
]
resolver = "2"
```

### Benefits

- **Shared compilation** — dependencies are built once.
- **Atomic changes** — modify multiple crates in one commit.
- **Clear boundaries** — each crate has its own `Cargo.toml` and public API.
- **Independent versioning** — crates can be published separately.

### When to use workspaces

- **Multiple binaries** — a CLI tool and a library that shares code.
- **Microservices** — multiple services sharing common types.
- **Large projects** — when compile times become a problem.

### When NOT to use workspaces

- **Single binary** — a workspace adds complexity without benefit.
- **Small projects** — one crate is simpler.
- **Unrelated crates** — if crates don't share code, keep them separate.

---

## Crate Design

### Library vs binary crates

```toml
# Library crate
[lib]
name = "my_domain"
path = "src/lib.rs"

# Binary crate
[[bin]]
name = "my_app"
path = "src/main.rs"
```

### Crate boundaries

Each crate should have a single, clear responsibility:

- `my-domain` — business logic, no I/O.
- `my-infrastructure` — database, external APIs.
- `my-app` — composition root, wires everything together.

### Dependency rules

- Domain crates should have **zero dependencies** on infrastructure crates.
- Infrastructure crates depend on domain crates (not vice versa).
- Application crates depend on both.

### Publishing considerations

- Keep the public API small and stable.
- Use `#[doc(hidden)]` for internal items that must be `pub` for macro expansion.
- Follow semver strictly.

---

## Layered Architecture

### Three-layer pattern

```rust
// Presentation layer — HTTP, CLI, etc.
pub async fn create_user(
    State(state): State<AppState>,
    Json(input): Json<CreateUserRequest>,
) -> Result<Json<User>, AppError> {
    let user = state.users.create(input).await?;
    Ok(Json(user))
}

// Application layer — use cases
impl UserService {
    pub async fn create(&self, input: CreateUserRequest) -> Result<User, AppError> {
        let user = User::new(input.name, input.email)?;
        self.repo.save(&user).await?;
        Ok(user)
    }
}

// Domain layer — business rules
impl User {
    pub fn new(name: String, email: String) -> Result<Self, ValidationError> {
        if name.is_empty() { return Err(ValidationError::EmptyName); }
        Ok(Self { id: UserId::new(), name, email })
    }
}
```

### When to use layered architecture

- Medium to large applications.
- When you need to swap infrastructure (e.g., Postgres to MySQL).
- When multiple presentation layers exist (HTTP + CLI + tests).

### When NOT to use layered architecture

- Small projects — the overhead isn't worth it.
- When the domain logic is trivial.

---

## Plugin Architecture

Use traits or enums for extensibility. With async traits there is a hard rule to know:

**A trait with a native `async fn` is not dyn-compatible** (the compiler error is
"the trait `Notifier` is not dyn compatible"). Each implementation's `async fn`
returns a different anonymous future type, so the compiler cannot build a vtable
entry for it. `Box<dyn Notifier>` is therefore rejected for this trait:

```rust
trait Notifier: Send + Sync {
    async fn notify(&self, message: &str) -> Result<(), Error>;
}

struct NotifierRegistry {
    notifiers: Vec<Box<dyn Notifier>>, // ERROR: `Notifier` is not dyn compatible
}
```

There are three ways out:

### Option 1 (default): static dispatch via generics

```rust
trait Notifier: Send + Sync {
    async fn notify(&self, message: &str) -> Result<(), Error>;
}

struct NotifierRegistry<N: Notifier> {
    notifiers: Vec<N>,
}

impl<N: Notifier> NotifierRegistry<N> {
    async fn notify_all(&self, message: &str) {
        for notifier in &self.notifiers {
            if let Err(e) = notifier.notify(message).await {
                tracing::error!("notification failed: {e}");
            }
        }
    }
}
```

Zero-cost and the preferred default. Limitation: all elements must be one concrete
type — for a heterogeneous set with static dispatch, use an enum of the backends
instead of a trait.

### Option 2: `#[async_trait]` when `dyn` is required

```rust
use async_trait::async_trait;

#[async_trait]
trait Notifier: Send + Sync {
    async fn notify(&self, message: &str) -> Result<(), Error>;
}

struct EmailNotifier;
struct SlackNotifier;

struct NotifierRegistry {
    notifiers: Vec<Box<dyn Notifier>>, // now compiles
}
```

`#[async_trait]` rewrites each `async fn` to return
`Pin<Box<dyn Future<Output = _> + Send + '_>>`, which is dyn-compatible — at the
cost of one heap allocation per call.

### Option 3: hand-written boxed future (no extra crate)

```rust
use std::future::Future;
use std::pin::Pin;

trait Notifier: Send + Sync {
    fn notify<'a>(
        &'a self,
        message: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
}

struct EmailNotifier;

impl Notifier for EmailNotifier {
    fn notify<'a>(
        &'a self,
        message: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
        Box::pin(async move {
            // send the email...
            let _ = message;
            Ok(())
        })
    }
}
```

Same result as `#[async_trait]`, spelled out manually.

---

## Configuration Structure

Model configuration as a typed, deserializable struct that mirrors your config
sources. This section covers the *shape*; loading (layered files, environment
overrides, secrets) is covered in `production.md` — Configuration Management.

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingVar(&'static str),
}

impl Config {
    /// Minimal std-only loader. For layered file + environment sources,
    /// use the `config` crate as shown in production.md.
    pub fn from_env() -> Result<Self, ConfigError> {
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingVar("DATABASE_URL"))?;
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        Ok(Self {
            database: DatabaseConfig { url, max_connections: 10 },
            server: ServerConfig { host: "0.0.0.0".into(), port },
        })
    }
}
```
