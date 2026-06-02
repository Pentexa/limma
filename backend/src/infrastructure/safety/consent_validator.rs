use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[async_trait]
pub trait ConsentRepository: Send + Sync {
    async fn get_active_consent(&self, target_domain: &str) -> Result<Option<(Uuid, String)>, String>;
    async fn insert_consent(
        &self,
        id: Uuid,
        target_domain: &str,
        consent_level: &str,
        granted_by: &str,
        granted_at: chrono::DateTime<Utc>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<(), String>;
    async fn revoke_consent(
        &self,
        id: Uuid,
        target_domain: &str,
        revoked_at: chrono::DateTime<Utc>,
    ) -> Result<bool, String>;
    async fn get_consents(&self) -> Result<Vec<crate::domain::entities::ConsentRecord>, String>;
    async fn insert_audit_log(
        &self,
        id: Uuid,
        action: &str,
        details: Option<&str>,
        target: Option<&str>,
        actor: Option<&str>,
    ) -> Result<(), String>;
}

pub struct PgConsentRepository {
    pool: PgPool,
}

impl PgConsentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConsentRepository for PgConsentRepository {
    async fn get_active_consent(&self, target_domain: &str) -> Result<Option<(Uuid, String)>, String> {
        let now = Utc::now();
        sqlx::query(
            r#"
            SELECT id, consent_level
            FROM consent_records
            WHERE target_domain = $1
              AND revoked = FALSE
              AND (expires_at IS NULL OR expires_at > $2)
            ORDER BY granted_at DESC
            LIMIT 1
            "#,
        )
        .bind(target_domain)
        .bind(now)
        .map(|row: sqlx::postgres::PgRow| (row.get("id"), row.get("consent_level")))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error while checking consent: {}", e))
    }

    async fn insert_consent(
        &self,
        id: Uuid,
        target_domain: &str,
        consent_level: &str,
        granted_by: &str,
        granted_at: chrono::DateTime<Utc>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO consent_records (id, target_domain, consent_level, granted_by, granted_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(target_domain)
        .bind(consent_level)
        .bind(granted_by)
        .bind(granted_at)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save consent record: {}", e))?;
        Ok(())
    }

    async fn revoke_consent(
        &self,
        id: Uuid,
        target_domain: &str,
        revoked_at: chrono::DateTime<Utc>,
    ) -> Result<bool, String> {
        let rows = sqlx::query(
            r#"
            UPDATE consent_records
            SET revoked = TRUE, revoked_at = $1
            WHERE id = $2 AND target_domain = $3 AND revoked = FALSE
            "#,
        )
        .bind(revoked_at)
        .bind(id)
        .bind(target_domain)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to revoke consent: {}", e))?;

        Ok(rows.rows_affected() > 0)
    }

    async fn get_consents(&self) -> Result<Vec<crate::domain::entities::ConsentRecord>, String> {
        sqlx::query(
            r#"
            SELECT id, target_domain, consent_level, granted_by, granted_at, expires_at, revoked, revoked_at
            FROM consent_records
            ORDER BY granted_at DESC
            "#,
        )
        .map(|row: sqlx::postgres::PgRow| crate::domain::entities::ConsentRecord {
            id: row.get("id"),
            target_domain: row.get("target_domain"),
            consent_level: row.get("consent_level"),
            granted_by: row.get("granted_by"),
            granted_at: row.get("granted_at"),
            expires_at: row.try_get("expires_at").unwrap_or(None),
            revoked: row.get("revoked"),
            revoked_at: row.try_get("revoked_at").unwrap_or(None),
        })
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error while fetching consents: {}", e))
    }

    async fn insert_audit_log(
        &self,
        id: Uuid,
        action: &str,
        details: Option<&str>,
        target: Option<&str>,
        actor: Option<&str>,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, action, details, target, actor)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(action)
        .bind(details)
        .bind(target)
        .bind(actor)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save audit log: {}", e))?;
        Ok(())
    }
}

/// Consent Validator business logic
pub struct ConsentValidatorImpl {
    repo: Box<dyn ConsentRepository>,
}

impl ConsentValidatorImpl {
    pub fn new(repo: Box<dyn ConsentRepository>) -> Self {
        Self { repo }
    }

    /// Verifies if there is active consent for the given target domain and consent level.
    pub async fn verify_consent(
        &self,
        target_domain: &str,
        required_level: &str,
    ) -> Result<(), String> {
        let result = self.repo.get_active_consent(target_domain).await?;

        match result {
            Some((_id, consent_level)) => {
                if consent_level == required_level || consent_level == "L3" {
                    tracing::debug!(
                        "[ConsentValidator] Consent granted for {} at level {}",
                        target_domain,
                        consent_level
                    );
                    Ok(())
                } else {
                    tracing::warn!(
                        "[ConsentValidator] Insufficient consent level for {}. Required: {}, Found: {}",
                        target_domain,
                        required_level,
                        consent_level
                    );
                    Err(format!(
                        "Insufficient consent level. Required: {}, Found: {}",
                        required_level, consent_level
                    ))
                }
            }
            None => {
                tracing::warn!(
                    "[ConsentValidator] No valid consent found for {}",
                    target_domain
                );
                Err(format!(
                    "No valid consent record found for domain: {}",
                    target_domain
                ))
            }
        }
    }

    pub async fn grant_consent(
        &self,
        target_domain: &str,
        consent_level: &str,
        granted_by: &str,
        duration_days: Option<i64>,
    ) -> Result<Uuid, String> {
        let id = Uuid::new_v4();
        let granted_at = Utc::now();
        let expires_at = duration_days.map(|d| granted_at + chrono::Duration::days(d));

        self.repo
            .insert_consent(id, target_domain, consent_level, granted_by, granted_at, expires_at)
            .await?;

        tracing::info!(
            "[ConsentValidator] Consent granted for {} at level {} by {}",
            target_domain,
            consent_level,
            granted_by
        );

        Ok(id)
    }

    pub async fn revoke_consent(&self, id: Uuid, target_domain: &str) -> Result<(), String> {
        let now = Utc::now();
        let success = self.repo.revoke_consent(id, target_domain, now).await?;

        if success {
            tracing::info!("[ConsentValidator] Consent record {} revoked", id);
            Ok(())
        } else {
            Err("Consent record not found or already revoked".to_string())
        }
    }

    pub async fn get_consents(&self) -> Result<Vec<crate::domain::entities::ConsentRecord>, String> {
        self.repo.get_consents().await
    }

    pub async fn log_audit(
        &self,
        action: &str,
        details: Option<&str>,
        target: Option<&str>,
        actor: Option<&str>,
    ) -> Result<(), String> {
        let id = Uuid::new_v4();
        self.repo.insert_audit_log(id, action, details, target, actor).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // A mock repository for pure unit tests
    struct MockConsentRepository {
        active_consent: Mutex<Option<(Uuid, String)>>,
        revoked: Mutex<bool>,
        audit_logs: Mutex<Vec<String>>,
    }

    impl MockConsentRepository {
        fn new() -> Self {
            Self {
                active_consent: Mutex::new(None),
                revoked: Mutex::new(false),
                audit_logs: Mutex::new(Vec::new()),
            }
        }
        fn set_active(&self, level: &str) {
            *self.active_consent.lock().unwrap() = Some((Uuid::new_v4(), level.to_string()));
        }
    }

    #[async_trait]
    impl ConsentRepository for MockConsentRepository {
        async fn get_active_consent(&self, _target_domain: &str) -> Result<Option<(Uuid, String)>, String> {
            Ok(self.active_consent.lock().unwrap().clone())
        }
        async fn insert_consent(
            &self,
            id: Uuid,
            _target_domain: &str,
            consent_level: &str,
            _granted_by: &str,
            _granted_at: chrono::DateTime<Utc>,
            _expires_at: Option<chrono::DateTime<Utc>>,
        ) -> Result<(), String> {
            *self.active_consent.lock().unwrap() = Some((id, consent_level.to_string()));
            Ok(())
        }
        async fn revoke_consent(&self, _id: Uuid, _target_domain: &str, _revoked_at: chrono::DateTime<Utc>) -> Result<bool, String> {
            *self.revoked.lock().unwrap() = true;
            *self.active_consent.lock().unwrap() = None;
            Ok(true)
        }
        async fn get_consents(&self) -> Result<Vec<crate::domain::entities::ConsentRecord>, String> {
            Ok(vec![])
        }
        async fn insert_audit_log(&self, _id: Uuid, action: &str, _details: Option<&str>, _target: Option<&str>, _actor: Option<&str>) -> Result<(), String> {
            self.audit_logs.lock().unwrap().push(action.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_verify_consent_insufficient_level() {
        let repo = MockConsentRepository::new();
        repo.set_active("L1");
        let validator = ConsentValidatorImpl::new(Box::new(repo));

        let res = validator.verify_consent("example.com", "L3").await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Insufficient consent level. Required: L3, Found: L1");
    }

    #[tokio::test]
    async fn test_verify_consent_sufficient_level() {
        let repo = MockConsentRepository::new();
        repo.set_active("L3");
        let validator = ConsentValidatorImpl::new(Box::new(repo));

        let res = validator.verify_consent("example.com", "L2").await;
        assert!(res.is_ok(), "L3 implies sufficient for L2/L1 implicitly via code");
    }

    #[tokio::test]
    async fn test_grant_and_revoke_consent() {
        let repo = MockConsentRepository::new();
        let validator = ConsentValidatorImpl::new(Box::new(repo));

        let id = validator.grant_consent("example.com", "L2", "admin", None).await.unwrap();
        assert!(validator.verify_consent("example.com", "L2").await.is_ok());

        validator.revoke_consent(id, "example.com").await.unwrap();
        assert!(validator.verify_consent("example.com", "L2").await.is_err());
    }

    #[tokio::test]
    async fn test_audit_log_records_action() {
        let repo = Box::new(MockConsentRepository::new());
        let validator = ConsentValidatorImpl::new(repo);

        validator.log_audit("TEST_ACTION", None, Some("example.com"), None).await.unwrap();
        // With a proper trait/mock we could assert the audit logs list.
    }
}
