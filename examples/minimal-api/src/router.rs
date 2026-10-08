use axum::{routing::get, Router};

use crate::handlers;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route(
            "/api/v1/items",
            get(handlers::list_items).post(handlers::create_item),
        )
        // axum 0.8 path parameter syntax (`{id}`, not `:id`)
        .route("/api/v1/items/{id}", get(handlers::get_item))
        .with_state(state)
}
