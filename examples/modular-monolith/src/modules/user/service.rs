use uuid::Uuid;

use super::error::UserError;
use super::models::{CreateUserRequest, User};
use super::repository::UserRepository;

/// Business logic for users. Also serves as the module's contract for other
/// modules: anything another module may ask about users goes through here.
#[derive(Clone)]
pub struct UserService {
    repository: UserRepository,
}

impl UserService {
    pub(crate) fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub async fn list_users(&self) -> Vec<User> {
        self.repository.find_all().await
    }

    pub async fn get_user(&self, id: Uuid) -> Option<User> {
        self.repository.find_by_id(id).await
    }

    /// Cross-module contract: does a user with this ID exist?
    pub async fn user_exists(&self, id: Uuid) -> bool {
        self.repository.find_by_id(id).await.is_some()
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> Result<User, UserError> {
        let email = req.email.trim();
        let name = req.name.trim();
        if name.is_empty() {
            return Err(UserError::InvalidInput("name must not be empty".into()));
        }
        if email.is_empty() || !email.contains('@') {
            return Err(UserError::InvalidInput(
                "email must be a valid address".into(),
            ));
        }

        let user = User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            name: name.to_string(),
            created_at: chrono::Utc::now(),
        };

        // The repository checks for duplicates and inserts atomically.
        self.repository.create(user).await
    }

    pub async fn delete_user(&self, id: Uuid) -> bool {
        self.repository.delete(id).await
    }
}
