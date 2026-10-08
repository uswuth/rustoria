use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use super::error::UserError;
use super::models::User;

/// In-memory user storage owned by this module. A real implementation would
/// talk to a database; callers only depend on these methods, not on how the
/// data is stored.
///
/// Cloning is cheap (shared `Arc`) and all clones see the same data.
#[derive(Clone, Default)]
pub struct UserRepository {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
}

impl UserRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn find_all(&self) -> Vec<User> {
        self.users.read().await.values().cloned().collect()
    }

    pub async fn find_by_id(&self, id: Uuid) -> Option<User> {
        self.users.read().await.get(&id).cloned()
    }

    /// Inserts a user, failing if the email is already taken.
    ///
    /// The existence check and the insert run under a single write lock, so
    /// two concurrent requests cannot both pass the check (no TOCTOU race).
    pub async fn create(&self, user: User) -> Result<User, UserError> {
        let mut users = self.users.write().await;
        if users.values().any(|u| u.email == user.email) {
            return Err(UserError::EmailTaken(user.email.clone()));
        }
        users.insert(user.id, user.clone());
        Ok(user)
    }

    pub async fn delete(&self, id: Uuid) -> bool {
        self.users.write().await.remove(&id).is_some()
    }
}
