use crate::domain::active_vuln::{ActiveVulnType, PayloadDefinition};
use crate::domain::engine_config::{EngineConfig, FuzzingIntensity};
use crate::infrastructure::active_detection::payloads::PayloadDatabase;
use std::sync::Arc;

/// `PayloadSelector` is a centralized, intensity-aware wrapper around `PayloadDatabase`.
///
/// Instead of each detector directly calling `PayloadDatabase::get_payloads()`,
/// they go through `PayloadSelector` which:
///   1. Filters payloads by `FuzzingIntensity` (Low → minimal, Aggressive → full + WAF bypass)
///   2. Enforces sandbox/approval constraints for aggressive modes
///   3. Applies safe_mode filtering (production-safe only)
///   4. Optionally enables WAF bypass transformations
pub struct PayloadSelector {
    payload_db: Arc<PayloadDatabase>,
    intensity: FuzzingIntensity,
    safe_mode: bool,
    enable_waf_bypass: bool,
    active_exploit_enabled: bool,
}


impl PayloadSelector {
    /// Creates a new `PayloadSelector` from an `EngineConfig` and shared `PayloadDatabase`.
    pub fn from_config(config: &EngineConfig, payload_db: Arc<PayloadDatabase>) -> Self {
        Self {
            payload_db,
            intensity: config.fuzzing_intensity.clone(),
            safe_mode: !config.active_exploit_enabled, // safe_mode is inverse of exploit permission
            enable_waf_bypass: config.avoid_waf,
            active_exploit_enabled: config.active_exploit_enabled,
        }
    }

    /// Creates a new `PayloadSelector` with explicit parameters (for backward compat).
    pub fn new(
        payload_db: Arc<PayloadDatabase>,
        intensity: FuzzingIntensity,
        safe_mode: bool,
        enable_waf_bypass: bool,
        active_exploit_enabled: bool,
    ) -> Self {
        Self {
            payload_db,
            intensity,
            safe_mode,
            enable_waf_bypass,
            active_exploit_enabled,
        }
    }

    /// Returns the maximum number of payloads to use for a given vulnerability type,
    /// based on the current fuzzing intensity.
    fn max_payloads_for_intensity(&self) -> usize {
        match self.intensity {
            FuzzingIntensity::Low => 2,
            FuzzingIntensity::Medium => 5,
            FuzzingIntensity::High => 15,
            FuzzingIntensity::Aggressive => usize::MAX, // No limit
        }
    }

    /// Returns whether WAF bypass transformations should be applied.
    fn should_apply_waf_bypass(&self) -> bool {
        self.enable_waf_bypass
            && matches!(
                self.intensity,
                FuzzingIntensity::High | FuzzingIntensity::Aggressive
            )
    }

    /// Returns whether non-production-safe payloads are allowed.
    fn allows_unsafe_payloads(&self) -> bool {
        // Aggressive mode with exploit enabled AND sandbox constraints satisfied
        !self.safe_mode
            && self.active_exploit_enabled
            && self.intensity == FuzzingIntensity::Aggressive
    }

    /// Returns filtered payloads for a specific vulnerability type.
    ///
    /// This is the primary entry point for all detectors.
    pub fn select(&self, vuln_type: ActiveVulnType) -> Vec<PayloadDefinition> {
        let effective_safe_mode = if self.allows_unsafe_payloads() {
            false
        } else {
            true // Default to safe mode unless explicitly unlocked
        };

        let mut payloads = if self.should_apply_waf_bypass() {
            self.payload_db
                .get_payloads_with_bypass(vuln_type, effective_safe_mode)
        } else {
            self.payload_db.get_payloads(vuln_type, effective_safe_mode)
        };

        // Truncate to intensity-based limit
        let max = self.max_payloads_for_intensity();
        if payloads.len() > max {
            payloads.truncate(max);
        }

        payloads
    }

    /// Returns filtered payloads for multiple vulnerability types at once.
    pub fn select_multi(
        &self,
        vuln_types: &[ActiveVulnType],
    ) -> Vec<(ActiveVulnType, PayloadDefinition)> {
        let mut result = Vec::new();
        for vt in vuln_types {
            for payload in self.select(*vt) {
                result.push((*vt, payload));
            }
        }
        result
    }

    /// Returns the current fuzzing intensity level.
    pub fn intensity(&self) -> &FuzzingIntensity {
        &self.intensity
    }

    /// Returns whether we're operating in safe mode.
    pub fn is_safe_mode(&self) -> bool {
        self.safe_mode
    }

    /// Returns whether WAF bypass is active.
    pub fn is_waf_bypass_enabled(&self) -> bool {
        self.should_apply_waf_bypass()
    }
}
