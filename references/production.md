# Production Readiness in Rust

## Overview

Production code requires observability, graceful shutdown, and robust configuration management. This reference covers
the essential patterns for running Rust services in production.

---

## Observability with `tracing`

`tracing` is the standard observability framework. It provides structured, async-aware logging.

### Setup

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

### Spans and events

```rust
use tracing::{info, instrument};

#[instrument]
async fn process_order(order_id: u64) -> Result<Order, Error> {
    info!(order_id, "processing order");

    validate_order(order_id).await?;

    info!(order_id, "order processed successfully");
    Ok(Order::new(order_id))
}
```

`#[instrument]` creates a span for the call and manages entering it correctly
across `.await` points — do not also `span.enter()` manually.

**Rule: never hold an `Entered` guard (`let _enter = span.enter();`) across
`.await`.** The guard is `Send`, but the span stays "current" on that thread
while the task is suspended, leaking your context into unrelated tasks (the
owned `EnteredSpan` guard is `!Send`, which prevents spawning entirely). For
sync code the guard is fine; for futures use
`.instrument(span)` (see below).

### Structured fields

```rust
use tracing::info;

let user_id = 42;
let action = "login";

info!(user_id, action, "user activity");
// Output: user_id=42 action=login "user activity"
```

### Propagating context across threads

```rust
use tracing::Instrument;

let span = tracing::info_span!("background_task");
tokio::spawn(
    async move {
        // work
    }
    .instrument(span),
);
```

### Propagating context across services

`tracing` does **not** inject headers for you — propagation is manual, or done via
OpenTelemetry. Manual version:

```rust
// Extract trace context from an incoming request
let trace_id = req
    .headers()
    .get("x-trace-id")
    .and_then(|v| v.to_str().ok())
    .map(String::from);

// Record it on your span as a plain field
let span = tracing::info_span!("handler", trace_id = trace_id.as_deref().unwrap_or("none"));

// Inject it into outgoing requests explicitly
let mut request = http::Request::new(());
if let Some(id) = trace_id {
    request
        .headers_mut()
        .insert("x-trace-id", id.parse().expect("header value"));
}
```

For standardized propagation (W3C `traceparent` headers, OTLP export to
Jaeger/Tempo/etc.), use `tracing-opentelemetry` with the OpenTelemetry SDK —
it bridges tracing spans into OpenTelemetry and provides header extractors
and injectors.

### JSON logging

For log aggregation (Loki, Elasticsearch, Cloud Logging), emit JSON:

```rust
use tracing_subscriber::fmt;

fmt()
    .json()
    .with_current_span(true)
    .with_span_list(true)
    .init();
```

### Log levels

```rust
use tracing::{debug, error, info, trace, warn};

trace!("very detailed");  // development only
debug!("debug info");     // development and staging
info!("informational");   // production default
warn!("warning");         // production
error!("error");          // production
```

### Correlation IDs

```rust
use tracing::info_span;

let correlation_id = uuid::Uuid::new_v4();
let span = info_span!("request", correlation_id = %correlation_id);
let _enter = span.enter();

// All events within this scope include the correlation_id field.
// (Sync code only — across .await, use .instrument(span) as shown above.)
info!("processing request");
```

---

## Metrics

### `metrics` crate

Examples use the **metrics 0.21+ handle API**: the macros return a handle
(`Counter`/`Gauge`/`Histogram`) that you call methods on. (In 0.20 and earlier
the macros recorded immediately with a different signature.)

```rust
use metrics::{counter, gauge, histogram};

// Counter — monotonically increasing
counter!("requests_total", "method" => "GET").increment(1);

// Gauge — can go up or down
gauge!("connections_active").set(42.0);

// Histogram — distribution of values
histogram!("request_duration_seconds").record(0.123);
```

### `metrics-exporter-prometheus`

```rust
use metrics_exporter_prometheus::PrometheusBuilder;

fn init_metrics() {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");
}
```

### Custom metrics

```rust
use metrics::{describe_counter, describe_histogram, counter, histogram};

describe_counter!("http_requests_total", "Total HTTP requests");
describe_histogram!("http_request_duration_seconds", "HTTP request duration");

counter!("http_requests_total", "status" => "200").increment(1);
histogram!("http_request_duration_seconds").record(0.045);
```

---

## Health Checks

### Liveness vs readiness

- **Liveness** — is the process running? (Kubernetes restarts if failing)
- **Readiness** — is the service ready to accept traffic? (Kubernetes removes from endpoints if failing)

```rust
use axum::{routing::get, Json, Router};

async fn liveness() -> &'static str {
    "ok"
}

async fn readiness(State(state): State<AppState>) -> Result<Json<HealthStatus>, StatusCode> {
    if state.database.ping().await {
        Ok(Json(HealthStatus { status: "ready" }))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[derive(serde::Serialize)]
struct HealthStatus {
    status: String,
}

let app = Router::new()
    .route("/healthz", get(liveness))
    .route("/readyz", get(readiness));
```

### Deep health checks

```rust
async fn deep_health_check(state: &AppState) -> HealthReport {
    let db_healthy = state.database.ping().await;
    let cache_healthy = state.cache.ping().await;
    let queue_healthy = state.queue.ping().await;

    HealthReport {
        status: if db_healthy && cache_healthy && queue_healthy {
            "healthy"
        } else {
            "unhealthy"
        },
        checks: vec![
            ("database", db_healthy),
            ("cache", cache_healthy),
            ("queue", queue_healthy),
        ],
    }
}
```

---

## Graceful Shutdown

### Signal handling

```rust
use tokio::signal;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install ctrl-c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received, starting graceful shutdown");
}
```

### Graceful shutdown with `JoinSet`

```rust
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let mut set = JoinSet::new();

    for listener in listeners {
        set.spawn(async move {
            serve(listener).await
        });
    }

    tokio::select! {
        _ = shutdown_signal() => {
            tracing::info!("shutting down...");
            set.abort_all();
            while set.join_next().await.is_some() {}
        }
    }
}
```

### Graceful shutdown with `CancellationToken`

```rust
use tokio_util::sync::CancellationToken;

async fn worker(token: CancellationToken) {
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                tracing::info!("worker shutting down");
                return;
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                // do work
            }
        }
    }
}
```

### Database connection cleanup

```rust
async fn graceful_shutdown(pool: PgPool) {
    // Stop accepting new connections
    // Wait for in-flight queries to complete
    pool.close().await;
}
```

---

## Configuration Management

### Layered configuration

```rust
use serde::Deserialize;
use config::{Config, Environment, File};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}
```

### Configuration files

```toml
# config/default.toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://localhost/mydb"
max_connections = 10

[logging]
level = "info"
```

### Environment variable overrides

```bash
# Override with environment variables
APP_SERVER__PORT=9090
APP_DATABASE__URL=postgres://prod-db/mydb
APP_LOGGING__LEVEL=debug
```

### Secrets in configuration

Use `secrecy::SecretString` (from the `secrecy` crate) — redacted `Debug`, memory
zeroized on drop:

```rust
use secrecy::{ExposeSecret, SecretString};

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub password: SecretString, // Debug prints redacted; zeroized on drop
}

impl DatabaseConfig {
    pub fn connect_string(&self) -> String {
        format!("{}?password={}", self.url, self.password.expose_secret())
    }
}
```

See `security.md` — Secrets Management — for sourcing secrets safely.

---

## Error Handling in Production

### Don't leak internal details

```rust
// Bad: exposes internal error details to clients
async fn bad() -> Response {
    match internal_operation().await {
        Ok(val) => Json(val).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// Good: log the error, return a generic message
async fn good() -> Response {
    match internal_operation().await {
        Ok(val) => Json(val).into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "internal error");
            (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
        }
    }
}
```

### Error responses

```rust
#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}
```

---

## Dependencies: std-First Rule

Before adding a dependency, ask:

1. **Does std provide it?** std grows: `LazyLock` replaced `lazy_static`,
   `OnceLock` covers most `once_cell` uses, `std::sync::mpsc` covers basic
   channels, `std::hint::black_box` covers benchmarking needs.
2. **Do std's semantics and performance suffice?** `std::sync::Mutex` is fine for
   short critical sections; you need `parking_lot` only for its specific API or
   measured contention wins.
3. **What does the crate add beyond std?** Name it concretely — async integration,
   a proven algorithm, platform coverage. If you can't, don't add it.
4. **Weigh the costs:** maintenance status and bus factor, security surface and
   auditability, MSRV compatibility, compile time, binary size, license,
   operational maturity.
5. **Add it when justified.** This is not "never use dependencies" — `tokio`,
   `serde`, and `tracing` earn their place many times over; a crate that saves
   three lines does not.

For auditing what you do add: `cargo audit` / `cargo deny` (see `security.md`
and `toolchain.md`).

## Engineering Decision Checklist

Before finalizing a design, walk these questions:

- **Ownership** — who creates, owns, mutates, and drops this value? Is `&T` /
  `&mut T` enough, or does it genuinely need `Box`/`Rc`/`Arc`?
- **Error paths** — which failures are recoverable? Does the caller need to match
  on variants (typed errors, `thiserror`) or just a message (`anyhow`)? Where is
  context added?
- **Concurrency** — does this cross threads? What must be `Send`/`Sync`? Shared
  state (`Arc<Mutex<T>>`) or message passing (channel)?
- **Async** — is the work I/O-bound (async fits) or CPU-bound (threads /
  `spawn_blocking`)? If the future is dropped at any `.await` (cancellation), is
  the state left consistent?
- **Performance** — where are the allocations and clones on the hot path? Did you
  measure before optimizing?
- **Dependencies** — maintained? License-compatible? Compile-time cost? Does std
  already cover it (see above)?
- **API** — is the surface minimal? Are invalid states unrepresentable? Is misuse
  a compile error where possible? Is it documented?

---

## Deployment Considerations

### Static linking with `musl`

```bash
# Build a fully static binary
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

### Docker multi-stage build

```dockerfile
# Build stage
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage - install everything the image execs (ca-certificates for TLS,
# curl for the HEALTHCHECK below)
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/my-app /usr/local/bin/my-app
ENTRYPOINT ["my-app"]
```

### Health check endpoints

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/healthz || exit 1
```

---

## Summary

| Practice | Tool/Crate | When |
| ---------- | ------------ | ------ |
| Structured logging | `tracing` | Always |
| Metrics | `metrics`, `metrics-exporter-prometheus` | Always |
| Health checks | custom endpoints | Always |
| Graceful shutdown | `tokio::signal`, `CancellationToken` | Always |
| Configuration | `config` crate + env vars | Always |
| Secrets in config | `secrecy` | Always |
| JSON logging | `tracing-subscriber` | Production |
| Correlation IDs | `tracing` spans | Distributed systems |
| Static linking | `musl` target | Container deployment |
