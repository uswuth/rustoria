//! Order module. Same rules as the user module: the facade below is the
//! only public surface; handlers, repository, and storage stay private.

mod error;
mod handlers;
mod models;
mod repository;
mod service;

use axum::routing::{get, Router};

use crate::modules::user::UserService;

pub use error::OrderError;
pub use models::{CreateOrderRequest, Order, OrderItem, OrderStatus};
pub use service::OrderService;

/// The order module's facade: owns its wiring and exposes its router.
pub struct OrderModule {
    service: OrderService,
}

impl OrderModule {
    /// Creates the order module. `users` is the user module's service
    /// handle — the defined contract the order module uses to validate
    /// user references before creating orders.
    pub fn new(users: UserService) -> Self {
        let repository = repository::OrderRepository::new();
        let service = service::OrderService::new(repository, users);
        Self { service }
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(handlers::list_orders).post(handlers::create_order))
            .route(
                "/{id}",
                get(handlers::get_order).delete(handlers::delete_order),
            )
            .with_state(self.service.clone())
    }
}
