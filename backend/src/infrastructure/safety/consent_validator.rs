use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// SQL-backed Consent Validator.
/// Verifies explicit user consent for active exploitation operations.
pub struct ConsentValidatorImpl {
    pool: PgPool,
}

impl ConsentValidatorImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Verifies if there is active consent for the given target domain and consent level.
    pub async fn verify_consent(
        &self,
        target_domain: &str,
        required_level: &str, // e.g., "L3" for active exploit
    ) -> Result<(), String> {
        let now = Utc::now();

        // 1. Check if there's an active, non-revoked consent record that hasn't expired.
        // We will just do a simple query. If multiple exist, we just need one valid one.
        let result: Option<(Uuid, String)> = sqlx::query(
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
        .map(|row: sqlx::postgres::PgRow| (row.get::<Uuid, _>("id"), row.get::<String, _>("consent_level")))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error while checking consent: {}", e))?;

        match result {
            Some((_id, consent_level)) => {
                // In a real system, you might have hierarchical levels (L3 > L2 > L1).
                // For now, we do a simple exact string match or hierarchical fallback if needed.
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

    /// Grants new consent for a target.
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

        tracing::info!(
            "[ConsentValidator] Consent granted for {} at level {} by {}",
            target_domain,
            consent_level,
            granted_by
        );

        Ok(id)
    }

    /// Revokes an existing consent by ID.
    pub async fn revoke_consent(&self, id: Uuid, target_domain: &str) -> Result<(), String> {
        let now = Utc::now();

        let rows = sqlx::query(
            r#"
            UPDATE consent_records
            SET revoked = TRUE, revoked_at = $1
            WHERE id = $2 AND target_domain = $3 AND revoked = FALSE
            "#,
        )
        .bind(now)
        .bind(id)
        .bind(target_domain)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to revoke consent: {}", e))?;

        if rows.rows_affected() > 0 {
            tracing::info!("[ConsentValidator] Consent record {} revoked", id);
            Ok(())
        } else {
            Err("Consent record not found or already revoked".to_string())
        }
    }
    pub async fn get_consents(&self) -> Result<Vec<crate::domain::entities::ConsentRecord>, String> {
        sqlx::query(
            r#"
            SELECT id, target_domain, consent_level, granted_by, granted_at, expires_at, revoked, revoked_at
            FROM consent_records
            ORDER BY granted_at DESC
            "#,
        )
        .map(|row: sqlx::postgres::PgRow| {
            crate::domain::entities::ConsentRecord {
                id: row.get("id"),
                target_domain: row.get("target_domain"),
                consent_level: row.get("consent_level"),
                granted_by: row.get("granted_by"),
                granted_at: row.get("granted_at"),
                expires_at: row.try_get("expires_at").unwrap_or(None),
                revoked: row.get("revoked"),
                revoked_at: row.try_get("revoked_at").unwrap_or(None),
            }
        })
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error while fetching consents: {}", e))
    }

    pub async fn log_audit(
        &self,
        action: &str,
        details: Option<&str>,
        target: Option<&str>,
        actor: Option<&str>,
    ) -> Result<(), String> {
        let id = Uuid::new_v4();
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
