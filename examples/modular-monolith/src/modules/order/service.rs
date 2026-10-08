use uuid::Uuid;

use super::error::OrderError;
use super::models::{CreateOrderRequest, Order, OrderStatus};
use super::repository::OrderRepository;
use crate::modules::user::UserService;

/// Business logic for orders. Depends on the user module only through the
/// `UserService` handle it is given — the modules never share storage.
#[derive(Clone)]
pub struct OrderService {
    repository: OrderRepository,
    users: UserService,
}

impl OrderService {
    pub(crate) fn new(repository: OrderRepository, users: UserService) -> Self {
        Self { repository, users }
    }

    pub async fn list_orders(&self) -> Vec<Order> {
        self.repository.find_all().await
    }

    pub async fn get_order(&self, id: Uuid) -> Option<Order> {
        self.repository.find_by_id(id).await
    }

    pub async fn create_order(&self, req: CreateOrderRequest) -> Result<Order, OrderError> {
        if req.items.is_empty() {
            return Err(OrderError::NoItems);
        }
        if let Some(item) = req.items.iter().find(|i| i.quantity == 0) {
            return Err(OrderError::ZeroQuantity(item.product_id));
        }

        // Cross-module integrity check via the user module's contract.
        if !self.users.user_exists(req.user_id).await {
            return Err(OrderError::UnknownUser(req.user_id));
        }

        // Never trust a client-supplied total: compute it from the items.
        let total_cents = req
            .items
            .iter()
            .map(|i| u64::from(i.quantity) * i.unit_price_cents)
            .sum();

        let order = Order {
            id: Uuid::new_v4(),
            user_id: req.user_id,
            items: req.items,
            total_cents,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now(),
        };

        self.repository.save(&order).await;
        Ok(order)
    }

    pub async fn delete_order(&self, id: Uuid) -> bool {
        self.repository.delete(id).await
    }
}
