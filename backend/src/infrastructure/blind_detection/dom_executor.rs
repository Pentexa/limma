use crate::domain::entities::*;

/// DOM XSS detection stub.
///
/// V1: Returns empty results. Real implementation will use a headless
/// browser (e.g., chromiumoxide) for taint analysis in a future phase.
pub struct DomExecutorStub;

impl DomExecutorStub {
    pub fn new() -> Self {
        Self
    }

    /// Stub: DOM XSS detection not yet implemented
    pub async fn detect_dom_xss(
        &self,
        _target_url: &str,
    ) -> Result<Vec<RawBlindFinding>, String> {
        tracing::debug!("[DomExecutorStub] DOM XSS detection deferred to future phase (headless browser required)");
        Ok(vec![])
    }
}
