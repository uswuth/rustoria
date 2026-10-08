//! User module. Everything outside this file only sees the facade below:
//! `UserModule` (wiring + router), `UserService` (the cross-module contract),
//! the request/model types, and the domain error. Handlers, the repository,
//! and storage details stay private to the module.

mod error;
mod handlers;
mod models;
mod repository;
mod service;

use axum::routing::{get, Router};

pub use error::UserError;
pub use models::{CreateUserRequest, User};
pub use service::UserService;

/// The user module's facade: owns its wiring and exposes its router.
pub struct UserModule {
    service: UserService,
}

impl UserModule {
    pub fn new() -> Self {
        let repository = repository::UserRepository::new();
        let service = service::UserService::new(repository);
        Self { service }
    }

    /// A handle to the user service. Other modules (e.g. orders) use this —
    /// and only this — to ask questions about users.
    pub fn service(&self) -> UserService {
        self.service.clone()
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(handlers::list_users).post(handlers::create_user))
            .route(
                "/{id}",
                get(handlers::get_user).delete(handlers::delete_user),
            )
            .with_state(self.service.clone())
    }
}

impl Default for UserModule {
    fn default() -> Self {
        Self::new()
    }
}
