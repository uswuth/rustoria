use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: &'static str,
}

pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// Listing items cannot fail, so the handler does not return `Result`.
pub async fn list_items(State(state): State<AppState>) -> impl IntoResponse {
    let items = state.items().await;
    Json(items)
}

#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(req): Json<CreateItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest(
            "Item name must not be empty".to_string(),
        ));
    }
    let item = state.create_item(name.to_string(), req.description).await;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    match state.get_item(id).await {
        Some(item) => Ok(Json(item)),
        None => Err(AppError::NotFound(format!("Item {id} not found"))),
    }
}
