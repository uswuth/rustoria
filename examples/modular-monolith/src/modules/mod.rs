pub mod order;
pub mod user;

use axum::Router;

use crate::modules::order::OrderModule;
use crate::modules::user::UserModule;

pub fn build_router(user_module: UserModule, order_module: OrderModule) -> Router {
    Router::new()
        .nest("/api/v1/users", user_module.router())
        .nest("/api/v1/orders", order_module.router())
}
