use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;
use async_trait::async_trait;
use std::sync::Mutex;

pub struct InMemoryUserRepository {
    users: Mutex<Vec<User>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn save(&self, user: User) -> Result<(), String> {
        let mut users = self.users.lock().map_err(|e| e.to_string())?;
        users.push(user);
        Ok(())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
        let users = self.users.lock().map_err(|e| e.to_string())?;
        Ok(users.iter().find(|u| u.email == email).cloned())
    }
}
