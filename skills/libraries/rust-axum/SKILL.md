---
name: rust-axum
description: Use when building HTTP APIs with axum.
---

# Axum — HTTP Framework for Rust

## Overview

Axum is an HTTP routing and request-handling library focused on ergonomics and modularity. It uses `tower::Service` for
middleware, giving access to the entire tower ecosystem.

## Installation

```toml
[dependencies]
axum = "0.8"  # 0.8.9 as of late 2026
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower-http = "0.7"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Core Concepts

### 1. Basic Server

```rust
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
    email: String,
}

async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1337,
        username: payload.username,
        email: payload.email,
    };
    (StatusCode::CREATED, Json(user))
}
```

### 2. Routing

> **axum 0.8 route syntax**: path parameters use `{id}` (and `{*rest}` for wildcards). The old `:id` / `*rest` syntax
> from 0.7 was removed — registering a route containing `:` **panics at startup**. Migrate: `/users/:id` →
> `/users/{id}`.

```rust
// Fragment: handler functions (get_index, get_users, ...) are defined elsewhere.
let app = Router::new()
    .route("/", get(get_index))
    .route("/users", get(get_users).post(create_user))
    .route("/users/{id}", get(get_user).put(update_user).delete(delete_user))
    .route("/users/{id}/posts", get(get_user_posts))
    .nest("/api", api_routes())
    .fallback(not_found);

// Nested routes
fn api_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/version", get(version))
}
```

### 3. Path Parameters

```rust
use axum::extract::Path;
use axum::Json;

// Matches "/users/{id}"; `User` is defined in §1.
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    Json(User { id, username: "...".into(), email: "...".into() })
}

// Multiple params — matches "/users/{user_id}/posts/{post_id}"
async fn get_user_post(
    Path((user_id, post_id)): Path<(u64, u64)>
) -> String {
    format!("User {}, Post {}", user_id, post_id)
}
```

### 4. Query Parameters

```rust
use axum::extract::Query;
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u64>,
    per_page: Option<u64>,
}

async fn list_users(Query(pagination): Query<Pagination>) -> Json<Vec<User>> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20);
    println!("page {page}, per_page {per_page}");
    Json(vec![]) // replace with a real query
}
```

### 5. Extractors

Extractors are types that implement `FromRequest` or `FromRequestParts`. They parse the request and provide typed data
to handlers.

```rust
// Signature overview — illustrative fragment; `Params`/`AppState` are your own types.
use axum::{
    extract::{Path, Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

async fn handler(
    Json(body): Json<CreateUser>,          // JSON body
    Path(id): Path<u64>,                   // path param
    Query(params): Query<Params>,          // query string
    headers: HeaderMap,                    // all headers
    State(state): State<AppState>,         // application state
    request: Request,                      // full request (consuming extractor goes last)
) -> impl IntoResponse {
    let _ = (body, id, params, headers, state, request);
    StatusCode::OK
}
```

Custom extractor:

```rust
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};

struct AuthUser(User); // `User` from §1

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Demo-grade: replace with real verification (JWT, session lookup, ...)
        match auth_header.strip_prefix("Bearer ") {
            Some(token) if !token.is_empty() => {
                let user = User {
                    id: 1,
                    username: "demo".into(),
                    email: "demo@example.com".into(),
                };
                Ok(AuthUser(user))
            }
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
```

### 6. State

```rust
use axum::{extract::State, routing::get, Router};
use std::sync::Arc;

struct AppState {
    // e.g. a database pool, HTTP client, config
    environment: String,
}

async fn get_users(State(state): State<Arc<AppState>>) -> String {
    format!("running in {}", state.environment)
}

// Arc<AppState> is Clone, so it satisfies with_state's bound without
// deriving Clone on AppState itself.
fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/users", get(get_users))
        .with_state(state)
}
```

### 7. Error Handling

```rust
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug)]
enum AppError {
    NotFound,
    Database(String), // message from your DB layer (diesel, sqlx, ...)
    Validation(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Database(e) => {
                tracing::error!("database error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            // Return the owned String — a &str borrowed from `msg` would not
            // live long enough (E0597).
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
        }
    }
}

// Handlers return Result<_, AppError> and use ?
async fn get_user(Path(id): Path<u64>) -> Result<Json<User>, AppError> {
    if id == 0 {
        return Err(AppError::NotFound);
    }
    // A DB failure maps like: .map_err(|e| AppError::Database(e.to_string()))?
    Ok(Json(User {
        id,
        username: "demo".into(),
        email: "demo@example.com".into(),
    }))
}
```

### 8. Middleware

```rust
use axum::{
    extract::Request,
    http::{header, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

// Security: CorsLayer::permissive() allows ANY origin — fine for local dev,
// unsafe in production. Restrict origins, methods, and headers instead:
let cors = CorsLayer::new()
    .allow_origin(
        "https://app.example.com"
            .parse::<HeaderValue>()
            .expect("valid origin"),
    )
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([header::CONTENT_TYPE]);

let app = Router::new()
    .route("/", get(root))
    .layer(TraceLayer::new_for_http())
    .layer(cors)
    .layer(CompressionLayer::new());

// Custom middleware
async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    println!("token present: {}", !token.is_empty());
    Ok(next.run(request).await)
}

let protected = Router::new()
    .route("/protected", get(root))
    .route_layer(middleware::from_fn(auth_middleware));
```

### 9. JSON API

```rust
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    data: T,
    message: Option<String>,
}

async fn get_users() -> Json<ApiResponse<Vec<User>>> {
    Json(ApiResponse {
        data: vec![], // replace with a real query
        message: None,
    })
}
```

### 10. WebSocket

Requires the `ws` feature: `axum = { version = "0.8", features = ["ws"] }`.

```rust
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg { msg } else { return; };
        if socket.send(msg).await.is_err() { return; }
    }
}
```

### 11. Testing

`ServiceExt::oneshot` comes from tower — for axum 0.8 add it as a dev-dependency:

```toml
[dev-dependencies]
tower = { version = "0.5", features = ["util"] }
```

```rust
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tower::ServiceExt; // for oneshot

#[tokio::test]
async fn test_get_user() {
    // `get_user` from §7
    let app = Router::new().route("/users/{id}", get(get_user));

    let response = app
        .oneshot(Request::builder().uri("/users/1").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

Alternative: bind a real `TcpListener` on port 0 and test over HTTP with `reqwest` — slower, but exercises the full stack.

### 12. Best Practices

1. **Use `State` for shared dependencies** — don't use global statics
2. **Implement `IntoResponse` for errors** — clean error handling
3. **Use `tower-http` middleware** — CORS, tracing, compression
4. **Validate input with extractors** — don't trust raw input
5. **Use `tracing` for logging** — structured, async-aware
6. **Keep handlers thin** — business logic in services
7. **Use `Router::nest`** for API versioning
8. **Return proper status codes** — 201 for create, 204 for delete

## When to Use Axum

- REST APIs
- Microservices
- WebSocket servers
- Proxy servers
- Any HTTP-based service
