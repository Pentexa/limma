pub mod consent_validator;
pub mod rate_limiter;
pub mod scope_enforcer;
pub mod waf_monitor;

use crate::domain::entities::*;
use async_trait::async_trait;

/// Infrastructure trait for the safety framework.
/// Validates scope, consent, and rate limits before exploit operations.
#[async_trait]

pub trait SafetyFramework: Send + Sync {
    /// Validate that a target URL is within the allowed scope
    async fn validate_target(&self, target_url: &str) -> Result<(), String>;

    /// Verify explicit user consent for L3 active exploitation
    async fn verify_explicit_consent(&self, poc: &Poc) -> Result<(), String>;

    /// Check rate limiting for exploit operations
    async fn check_rate_limit(&self, target_url: &str) -> Result<(), String>;
}

use sqlx::PgPool;

/// Concrete safety framework combining scope, consent, and rate limiting
pub struct SafetyFrameworkImpl {
    pool: PgPool,
    scope_enforcer: scope_enforcer::ScopeEnforcer,
    consent_validator: consent_validator::ConsentValidatorImpl,
    rate_limiter: rate_limiter::ExploitRateLimiter,
}

impl SafetyFrameworkImpl {
    pub fn new(pool: PgPool, allowed_domains: Vec<String>, max_requests_per_minute: u32) -> Self {
        let consent_repo = Box::new(consent_validator::PgConsentRepository::new(pool.clone()));
        Self {
            pool: pool.clone(),
            scope_enforcer: scope_enforcer::ScopeEnforcer::new(allowed_domains),
            consent_validator: consent_validator::ConsentValidatorImpl::new(consent_repo),
            rate_limiter: rate_limiter::ExploitRateLimiter::new(max_requests_per_minute),
        }
    }

    pub async fn grant_consent(
        &self,
        target_domain: &str,
        requested_by: &str,
        scope_level: &str,
        expires_in_hours: i64,
    ) -> Result<uuid::Uuid, String> {
        self.consent_validator
            .grant_consent(
                target_domain,
                requested_by,
                scope_level,
                Some(expires_in_hours),
            )
            .await
    }

    pub async fn revoke_consent(&self, id: uuid::Uuid, target_domain: &str) -> Result<(), String> {
        self.consent_validator
            .revoke_consent(id, target_domain)
            .await
    }

    pub async fn get_consents(
        &self,
    ) -> Result<Vec<crate::domain::entities::ConsentRecord>, String> {
        self.consent_validator.get_consents().await
    }

    pub async fn log_audit(
        &self,
        action: &str,
        details: Option<&str>,
        target: Option<&str>,
        actor: Option<&str>,
    ) -> Result<(), String> {
        self.consent_validator
            .log_audit(action, details, target, actor)
            .await
    }
}

#[async_trait]
impl SafetyFramework for SafetyFrameworkImpl {
    async fn validate_target(&self, target_url: &str) -> Result<(), String> {
        self.scope_enforcer.validate(target_url)?;
        self.rate_limiter.check(target_url)?;
        Ok(())
    }

    async fn verify_explicit_consent(&self, poc: &Poc) -> Result<(), String> {
        let result = sqlx::query("SELECT url FROM active_findings WHERE id = $1")
            .bind(poc.finding_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        let url_str = result
            .map(|r| {
                use sqlx::Row;
                r.try_get::<String, _>("url")
                    .unwrap_or_else(|_| "unknown".to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        // Extract domain
        let target_domain = if let Ok(parsed_url) = url::Url::parse(&url_str) {
            parsed_url.host_str().unwrap_or("unknown").to_string()
        } else {
            url_str
        };

        self.consent_validator
            .verify_consent(&target_domain, "L3")
            .await
    }

    async fn check_rate_limit(&self, target_url: &str) -> Result<(), String> {
        self.rate_limiter.check(target_url)
    }
}
