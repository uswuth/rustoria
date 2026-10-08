# Modular Monolith

A modular monolith example demonstrating feature-based organization with clear domain boundaries.

## Features

- **Feature-based modules** - Each module (user, order) has its own handlers, service, repository, models, and domain error
- **Enforced module boundaries** - Submodules are private; each module exposes only a small facade (`mod.rs`
  re-exports). Handlers, repositories, and storage details cannot leak into other modules
- **Explicit cross-module contract** - The order module validates user references by calling the user module's public
  `UserService` handle (passed in at wiring time). Modules never share storage
- **Shared common code** - `common/` holds only module-agnostic infrastructure (HTTP error type, configuration) and
  never depends on a feature module
- **Domain errors per module** - Services/repositories return domain errors (`UserError`, `OrderError`); handlers map
  them to HTTP responses
- **Safe money handling** - Prices are integer cents (`u64`), never `f64`; order totals are computed server-side from
  the items, never taken from the client
- **Race-free uniqueness** - Duplicate-email check and insert run under a single lock (no TOCTOU race)
- **Testing** - Integration tests cover each module and the cross-module interactions

## Project Structure

```text
src/
├── main.rs              # Entry point (thin shell over the library crate)
├── lib.rs               # Library exports
├── common/              # Shared, module-agnostic infrastructure
│   ├── mod.rs
│   ├── error.rs         # HTTP error type + JSON error body
│   └── config.rs        # Configuration
└── modules/             # Feature modules
    ├── mod.rs           # Module router aggregation
    ├── user/            # User module
    │   ├── mod.rs       # Facade: UserModule + public re-exports
    │   ├── handlers.rs  # HTTP handlers (private)
    │   ├── service.rs   # Business logic / cross-module contract
    │   ├── repository.rs# Data access (private)
    │   ├── models.rs    # Domain models
    │   └── error.rs     # UserError domain error
    └── order/           # Order module
        ├── mod.rs       # Facade: OrderModule + public re-exports
        ├── handlers.rs
        ├── service.rs
        ├── repository.rs
        ├── models.rs
        └── error.rs     # OrderError domain error
```

Adding a new feature module means adding a new directory under `modules/` and
wiring it in `modules/mod.rs` and `main.rs` — `common/` never changes.

## Running

```bash
# Set environment variables (optional)
export HOST=127.0.0.1
export PORT=3000   # must be a valid port number; invalid values abort startup

# Run the server
cargo run

# Run tests
cargo test
```

State is kept in memory (per-module repositories) and is lost on restart;
there is no external database to configure.

## API Endpoints

### Users

| Method | Path | Description |
| -------- | ------ | ------------- |
| GET | `/api/v1/users` | List all users |
| POST | `/api/v1/users` | Create a new user |
| GET | `/api/v1/users/{id}` | Get a specific user |
| DELETE | `/api/v1/users/{id}` | Delete a user |

### Orders

| Method | Path | Description |
| -------- | ------ | ------------- |
| GET | `/api/v1/orders` | List all orders |
| POST | `/api/v1/orders` | Create a new order (user must exist; total computed server-side) |
| GET | `/api/v1/orders/{id}` | Get a specific order |
| DELETE | `/api/v1/orders/{id}` | Delete an order |

Example order payload (amounts in cents):

```json
{
  "user_id": "<existing user id>",
  "items": [
    { "product_id": "<uuid>", "quantity": 2, "unit_price_cents": 1000 }
  ]
}
```

## What It Demonstrates

- **Modular architecture** - Feature-based organization with boundaries enforced by visibility, not convention
- **Layered design** - Handlers → Service → Repository pattern within each module
- **Dependency direction** - `common` depends on nothing; modules depend on `common` and (through service handles) on
  each other's facades
- **Domain modeling** - Per-module models and domain errors; money as integer cents
- **Cross-module integrity** - Orders cannot reference nonexistent users, checked through the user module's contract
- **Cross-module testing** - Integration tests that span multiple modules, including a concurrent
  duplicate-registration race test
- **Error propagation** - Domain errors in services/repositories, mapped to HTTP status codes at the handler layer
- **axum 0.8** - `{id}` path-parameter syntax; nested routers match the bare nest path (`/api/v1/users`, no trailing slash)
