use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use super::error::UserError;
use super::models::CreateUserRequest;
use super::service::UserService;
use crate::common::error::AppError;

/// Domain errors are mapped to HTTP errors here, at the module's edge.
impl From<UserError> for AppError {
    fn from(e: UserError) -> Self {
        match e {
            UserError::EmailTaken(email) => {
                AppError::Conflict(format!("User with email {email} already exists"))
            }
            UserError::InvalidInput(msg) => AppError::BadRequest(msg),
        }
    }
}

/// Listing users cannot fail, so the handler does not return `Result`.
pub async fn list_users(State(service): State<UserService>) -> impl IntoResponse {
    Json(service.list_users().await)
}

pub async fn create_user(
    State(service): State<UserService>,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = service.create_user(req).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn get_user(
    State(service): State<UserService>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    match service.get_user(id).await {
        Some(user) => Ok(Json(user)),
        None => Err(AppError::NotFound(format!("User {id} not found"))),
    }
}

pub async fn delete_user(
    State(service): State<UserService>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    match service.delete_user(id).await {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err(AppError::NotFound(format!("User {id} not found"))),
    }
}
