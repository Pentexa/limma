use crate::domain::entities::*;

// ── Domain Error Types ──

/// Safety-related domain errors (pure business logic, no framework deps)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SafetyError {
    OutOfScope,
    InsufficientConsent,
    RateLimitExceeded,
    InvalidTarget(String),
}

impl std::fmt::Display for SafetyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyError::OutOfScope => write!(f, "Target is out of allowed scope"),
            SafetyError::InsufficientConsent => {
                write!(f, "Insufficient consent for this operation")
            }
            SafetyError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            SafetyError::InvalidTarget(t) => write!(f, "Invalid target: {}", t),
        }
    }
}

impl std::error::Error for SafetyError {}

// ── Domain Services (Pure Business Logic — No Framework Dependencies) ──

/// Domain service for exploit safety validation
pub struct ExploitSafetyService;

impl ExploitSafetyService {
    /// Pure function: Validate if PoC is safe to execute given scope constraints.
    /// Returns the appropriate safety level based on poc_type and scope config.
    pub fn validate_safety_level(
        poc_type: &PocType,
        target: &str,
        scope: &SafetyScope,
    ) -> Result<SafetyLevel, SafetyError> {
        // Check scope membership (skip if no domains configured — open scope)
        let is_in_scope = scope.target_domains.is_empty()
            || scope
                .target_domains
                .iter()
                .any(|domain| target.contains(domain));

        if !is_in_scope {
            return Err(SafetyError::OutOfScope);
        }

        if scope.read_only {
            // Read-only mode: assign safety levels based on PoC type
            match poc_type {
                PocType::SqlInjection => {
                    // SELECT-only proof = L1 safe
                    Ok(SafetyLevel::L1SafeReadOnly)
                }
                PocType::PathTraversal => {
                    // Read-only path traversal = L1 safe
                    Ok(SafetyLevel::L1SafeReadOnly)
                }
                PocType::ServerSideRequestForgery => {
                    // Internal probe = L2 sandbox
                    Ok(SafetyLevel::L2VerifiedSandbox)
                }
                PocType::CommandInjection => {
                    // Even read-only CMDi needs sandbox
                    Ok(SafetyLevel::L2VerifiedSandbox)
                }
                _ => Ok(SafetyLevel::L2VerifiedSandbox),
            }
        } else {
            // Write-enabled mode: requires explicit consent
            Ok(SafetyLevel::L3ActiveWithConsent)
        }
    }

    /// Calculate CVSS score with exploitability context.
    /// Business rule: Verified exploit gets +20% to base score (capped at 10.0).
    #[allow(dead_code)]
    pub fn calculate_cvss(
        base_score: f32,
        poc_verified: bool,
        _exploit_complexity: &ExploitComplexity,
    ) -> f32 {
        let multiplier = if poc_verified { 1.2 } else { 1.0 };
        let adjusted = base_score * multiplier;
        adjusted.min(10.0)
    }
}

/// Domain service for blind detection scoring
pub struct BlindDetectionScoringService;

impl BlindDetectionScoringService {
    /// Pure function: Calculate confidence from timing data.
    ///
    /// Business logic:
    /// - 4x+ delay ratio with 3+ iterations = high confidence (0.9+)
    /// - 2x-4x delay ratio = medium confidence (0.7+)
    /// - Below 2x = low confidence (0.5)
    pub fn calculate_timing_confidence(
        baseline_ms: u32,
        delayed_ms: u32,
        iterations: u32,
    ) -> f32 {
        let delay_ratio = delayed_ms as f32 / baseline_ms.max(1) as f32;
        let iteration_bonus = (iterations as f32 * 0.05).min(0.2);

        if delay_ratio >= 4.0 {
            (0.9 + iteration_bonus).min(1.0)
        } else if delay_ratio >= 2.0 {
            0.7 + iteration_bonus
        } else {
            0.5
        }
    }
}
