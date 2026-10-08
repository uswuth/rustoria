# Minimal API

A minimal but production-grade REST API built with Axum.

## Features

- **Health check endpoint** - Simple liveness probe
- **Item operations** - Create, read, and list items (no update/delete)
- **Error handling** - Structured error responses with proper HTTP status codes, including input validation
- **Configuration** - Environment-based configuration with dotenv support
- **Observability** - Structured logging with tracing
- **Testing** - Integration tests with in-memory state

## Project Structure

```text
src/
├── main.rs      # Entry point, server setup
├── lib.rs       # Library exports
├── router.rs    # Route definitions
├── handlers.rs  # HTTP handlers
├── error.rs     # Error types
├── state.rs     # Application state
└── config.rs    # Configuration
```

## Running

```bash
# Set environment variables (optional)
export HOST=127.0.0.1
export PORT=3000

# Run the server
cargo run

# Run tests
cargo test
```

## API Endpoints

| Method | Path | Description |
| -------- | ------ | ------------- |
| GET | `/health` | Health check |
| GET | `/api/v1/items` | List all items |
| POST | `/api/v1/items` | Create a new item |
| GET | `/api/v1/items/{id}` | Get a specific item |

## What It Demonstrates

- Proper module boundaries with separate concerns (binary is a thin shell over the library crate)
- Error handling with `thiserror` and custom `IntoResponse`
- Application state management with `Arc<RwLock<T>>`
- Configuration loading from environment variables with validation (an invalid `PORT` is a startup error)
- Structured logging with `tracing`
- Integration testing with `tower::ServiceExt`
- axum 0.8, including its `{id}` path-parameter syntax
