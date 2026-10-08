use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub items: Vec<OrderItem>,
    /// Order total in cents, computed server-side from `items`.
    /// (Money is never `f64`: floats round, integers don't.)
    pub total_cents: u64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: Uuid,
    pub quantity: u32,
    /// Price per unit, in cents.
    pub unit_price_cents: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: Uuid,
    pub items: Vec<OrderItem>,
    // Deliberately no `total` field: the server computes it from `items`
    // and never trusts a client-supplied amount.
}
