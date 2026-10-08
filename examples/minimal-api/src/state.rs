use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::handlers::Item;

#[derive(Clone)]
pub struct AppState {
    items: Arc<RwLock<HashMap<Uuid, Item>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn items(&self) -> Vec<Item> {
        let items = self.items.read().await;
        items.values().cloned().collect()
    }

    pub async fn create_item(&self, name: String, description: Option<String>) -> Item {
        let item = Item {
            id: Uuid::new_v4(),
            name,
            description,
        };
        let mut items = self.items.write().await;
        items.insert(item.id, item.clone());
        item
    }

    pub async fn get_item(&self, id: Uuid) -> Option<Item> {
        let items = self.items.read().await;
        items.get(&id).cloned()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
