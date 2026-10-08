use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

use modular_monolith::modules::build_router;
use modular_monolith::modules::order::OrderModule;
use modular_monolith::modules::user::UserModule;

fn setup_app() -> axum::Router {
    let user_module = UserModule::new();
    // The order module validates user references through the user module's
    // public service handle.
    let order_module = OrderModule::new(user_module.service());
    build_router(user_module, order_module)
}

fn json_request(method: &str, uri: &str, body: Option<serde_json::Value>) -> Request<Body> {
    let builder = Request::builder().method(method).uri(uri);
    match body {
        Some(json) => builder
            .header("content-type", "application/json")
            .body(Body::from(json.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

async fn create_user(app: &axum::Router, email: &str, name: &str) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/users",
            Some(serde_json::json!({ "email": email, "name": name })),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    body_json(response).await
}

#[tokio::test]
async fn create_and_get_user() {
    let app = setup_app();

    let created = create_user(&app, "test@example.com", "Test User").await;
    let user_id = created["id"].as_str().unwrap();

    let get_response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/users/{user_id}"),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);

    let fetched = body_json(get_response).await;
    assert_eq!(fetched["email"], "test@example.com");
    assert_eq!(fetched["name"], "Test User");
}

#[tokio::test]
async fn create_duplicate_user_returns_conflict() {
    let app = setup_app();

    create_user(&app, "dup@example.com", "User").await;

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/users",
            Some(serde_json::json!({ "email": "dup@example.com", "name": "Other" })),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn concurrent_duplicate_user_creation_allows_exactly_one() {
    let app = setup_app();

    let body = || {
        json_request(
            "POST",
            "/api/v1/users",
            Some(serde_json::json!({ "email": "race@example.com", "name": "Racer" })),
        )
    };

    // The atomic check-and-insert in the repository must let exactly one
    // request win, even when both race on the same email.
    let (r1, r2) = tokio::join!(app.clone().oneshot(body()), app.oneshot(body()));
    let statuses = [r1.unwrap().status(), r2.unwrap().status()];

    assert!(
        statuses.contains(&StatusCode::CREATED),
        "one request should succeed, got {statuses:?}"
    );
    assert!(
        statuses.contains(&StatusCode::CONFLICT),
        "the other should conflict, got {statuses:?}"
    );
}

#[tokio::test]
async fn create_user_with_invalid_input_returns_400() {
    let app = setup_app();

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/users",
            Some(serde_json::json!({ "email": "not-an-email", "name": "User" })),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_nonexistent_user_returns_404() {
    let app = setup_app();

    let response = app
        .oneshot(json_request(
            "GET",
            "/api/v1/users/00000000-0000-0000-0000-000000000000",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_and_list_orders() {
    let app = setup_app();

    let user = create_user(&app, "order@example.com", "Order User").await;
    let user_id = user["id"].as_str().unwrap();

    // No `total` in the request: the server computes it from the items.
    let order_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/orders",
            Some(serde_json::json!({
                "user_id": user_id,
                "items": [
                    { "product_id": "00000000-0000-0000-0000-000000000001", "quantity": 2, "unit_price_cents": 1000 },
                    { "product_id": "00000000-0000-0000-0000-000000000002", "quantity": 1, "unit_price_cents": 499 }
                ]
            })),
        ))
        .await
        .unwrap();

    assert_eq!(order_response.status(), StatusCode::CREATED);
    let created = body_json(order_response).await;
    // 2 * 1000 + 1 * 499 = 2499 cents
    assert_eq!(created["total_cents"], 2499);
    assert_eq!(created["status"], "Pending");

    let list_response = app
        .oneshot(json_request("GET", "/api/v1/orders", None))
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);

    let body = list_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let orders: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0]["total_cents"], 2499);
}

#[tokio::test]
async fn create_order_for_unknown_user_returns_400() {
    let app = setup_app();

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/orders",
            Some(serde_json::json!({
                "user_id": "00000000-0000-0000-0000-000000000000",
                "items": [{ "product_id": "00000000-0000-0000-0000-000000000001", "quantity": 1, "unit_price_cents": 100 }]
            })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_order_without_items_returns_400() {
    let app = setup_app();

    let user = create_user(&app, "empty@example.com", "Empty User").await;
    let user_id = user["id"].as_str().unwrap();

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/orders",
            Some(serde_json::json!({ "user_id": user_id, "items": [] })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_order() {
    let app = setup_app();

    let user = create_user(&app, "del@example.com", "Del User").await;
    let user_id = user["id"].as_str().unwrap();

    let order_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/orders",
            Some(serde_json::json!({
                "user_id": user_id,
                "items": [{ "product_id": "00000000-0000-0000-0000-000000000001", "quantity": 1, "unit_price_cents": 100 }]
            })),
        ))
        .await
        .unwrap();
    let order = body_json(order_response).await;
    let order_id = order["id"].as_str().unwrap();

    let delete_response = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/orders/{order_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let get_response = app
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/orders/{order_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}
