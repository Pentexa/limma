pub mod consent_validator;
pub mod rate_limiter;
pub mod scope_enforcer;
pub mod waf_monitor;

use crate::domain::entities::*;
use async_trait::async_trait;

/// Infrastructure trait for the safety framework.
/// Validates scope, consent, and rate limits before exploit operations.
#[async_trait]
#[allow(dead_code)]
pub trait SafetyFramework: Send + Sync {
    /// Validate that a target URL is within the allowed scope
    async fn validate_target(&self, target_url: &str) -> Result<(), String>;

    /// Verify explicit user consent for L3 active exploitation
    async fn verify_explicit_consent(&self, poc: &Poc) -> Result<(), String>;

    /// Check rate limiting for exploit operations
    async fn check_rate_limit(&self, target_url: &str) -> Result<(), String>;
}

/// Concrete safety framework combining scope, consent, and rate limiting
pub struct SafetyFrameworkImpl {
    scope_enforcer: scope_enforcer::ScopeEnforcer,
    consent_validator: consent_validator::ConsentValidatorStub,
    rate_limiter: rate_limiter::ExploitRateLimiter,
}

impl SafetyFrameworkImpl {
    pub fn new(allowed_domains: Vec<String>, max_requests_per_minute: u32) -> Self {
        Self {
            scope_enforcer: scope_enforcer::ScopeEnforcer::new(allowed_domains),
            consent_validator: consent_validator::ConsentValidatorStub::new(),
            rate_limiter: rate_limiter::ExploitRateLimiter::new(max_requests_per_minute),
        }
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
        self.consent_validator
            .verify_consent(&format!("{:?}", poc.poc_type))
            .await
    }

    async fn check_rate_limit(&self, target_url: &str) -> Result<(), String> {
        self.rate_limiter.check(target_url)
    }
}
