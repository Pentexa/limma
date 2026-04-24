use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;
use async_trait::async_trait;
use sqlx::PgPool;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn save(&self, user: User) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO users (id, name, email, password_hash, created_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(user.id)
        .bind(user.name)
        .bind(user.email)
        .bind(user.password_hash)
        .bind(user.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
        let row = sqlx::query(
            "SELECT id, name, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.map(|r| {
            use sqlx::Row;
            User {
                id: r.try_get("id").unwrap_or_default(),
                name: r.try_get("name").unwrap_or_default(),
                email: r.try_get("email").unwrap_or_default(),
                password_hash: r.try_get("password_hash").unwrap_or_default(),
                created_at: r
                    .try_get("created_at")
                    .unwrap_or_else(|_| chrono::Utc::now()),
            }
        }))
    }
}
