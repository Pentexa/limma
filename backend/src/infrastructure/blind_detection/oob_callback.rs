use crate::domain::entities::*;

/// Out-of-band callback detection stub.
///
/// V1: Returns empty results. Real implementation will spawn a
/// DNS/HTTP callback server for SSRF/XXE interaction detection.
pub struct OobCallbackStub;

impl OobCallbackStub {
    pub fn new() -> Self {
        Self
    }

    /// Stub: OOB SSRF detection not yet implemented
    pub async fn detect_ssrf(
        &self,
        _target_url: &str,
    ) -> Result<Vec<RawBlindFinding>, String> {
        tracing::debug!("[OobCallbackStub] OOB SSRF detection deferred to future phase (callback server required)");
        Ok(vec![])
    }
}
