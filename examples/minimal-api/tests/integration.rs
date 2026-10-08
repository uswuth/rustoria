use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

use minimal_api::router::build_router;
use minimal_api::state::AppState;

#[tokio::test]
async fn health_check_returns_200() {
    let state = AppState::new();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert!(json["version"].as_str().is_some());
}

#[tokio::test]
async fn create_and_get_item() {
    let state = AppState::new();
    let app = build_router(state);

    // Create an item
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "Test Item",
                        "description": "A test item"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body = create_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let item_id = created["id"].as_str().unwrap();

    // Get the item
    let get_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/items/{item_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);

    let body = get_response.into_body().collect().await.unwrap().to_bytes();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(fetched["name"], "Test Item");
    assert_eq!(fetched["description"], "A test item");
}

#[tokio::test]
async fn create_item_with_empty_name_returns_400() {
    let state = AppState::new();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/items")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "name": "   " }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("name"));
}

#[tokio::test]
async fn get_nonexistent_item_returns_404() {
    let state = AppState::new();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/items/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_items_returns_empty_initially() {
    let state = AppState::new();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let items: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert!(items.is_empty());
}
