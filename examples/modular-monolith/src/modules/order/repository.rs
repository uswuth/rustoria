use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use super::models::Order;

/// In-memory order storage owned by this module.
///
/// Cloning is cheap (shared `Arc`) and all clones see the same data.
#[derive(Clone, Default)]
pub struct OrderRepository {
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
}

impl OrderRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn find_all(&self) -> Vec<Order> {
        self.orders.read().await.values().cloned().collect()
    }

    pub async fn find_by_id(&self, id: Uuid) -> Option<Order> {
        self.orders.read().await.get(&id).cloned()
    }

    pub async fn save(&self, order: &Order) {
        self.orders.write().await.insert(order.id, order.clone());
    }

    pub async fn delete(&self, id: Uuid) -> bool {
        self.orders.write().await.remove(&id).is_some()
    }
}
