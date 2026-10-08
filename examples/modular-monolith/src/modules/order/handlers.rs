use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use super::error::OrderError;
use super::models::CreateOrderRequest;
use super::service::OrderService;
use crate::common::error::AppError;

/// Domain errors are mapped to HTTP errors here, at the module's edge.
impl From<OrderError> for AppError {
    fn from(e: OrderError) -> Self {
        // All current order-domain errors are client errors.
        AppError::BadRequest(e.to_string())
    }
}

/// Listing orders cannot fail, so the handler does not return `Result`.
pub async fn list_orders(State(service): State<OrderService>) -> impl IntoResponse {
    Json(service.list_orders().await)
}

pub async fn create_order(
    State(service): State<OrderService>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<impl IntoResponse, AppError> {
    let order = service.create_order(req).await?;
    Ok((StatusCode::CREATED, Json(order)))
}

pub async fn get_order(
    State(service): State<OrderService>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    match service.get_order(id).await {
        Some(order) => Ok(Json(order)),
        None => Err(AppError::NotFound(format!("Order {id} not found"))),
    }
}

pub async fn delete_order(
    State(service): State<OrderService>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    match service.delete_order(id).await {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err(AppError::NotFound(format!("Order {id} not found"))),
    }
}
