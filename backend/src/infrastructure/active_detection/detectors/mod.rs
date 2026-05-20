pub mod xss_detector;
pub mod sqli_detector;
pub mod cmdi_detector;
pub mod lfi_detector;
pub mod ssrf_detector;
pub mod xxe_detector;
pub mod redirect_detector;
pub mod jwt_detector;
pub mod deser_detector;
pub mod idor_detector;
pub mod nosql_detector;
pub mod ssti_detector;

use async_trait::async_trait;
use crate::domain::active_vuln::{ActiveVulnFinding, ActiveVulnType};
use uuid::Uuid;
use std::sync::Arc;
use crate::infrastructure::safety::waf_monitor::WafMonitor;
use crate::infrastructure::active_detection::differential::BaselineProfile;

/// Common trait for all vulnerability detectors.
#[async_trait]
pub trait VulnDetector: Send + Sync {
    /// Returns which vulnerability types this detector can identify.
    fn supported_types(&self) -> Vec<ActiveVulnType>;

    /// Runs detection against a specific target URL and parameter.
    async fn detect(
        &self,
        target_url: &str,
        parameter: &str,
        scan_id: Uuid,
        payload_selector: &crate::infrastructure::active_detection::payload_selector::PayloadSelector,
        rate_limit_ms: u64,
        waf_monitor: Arc<WafMonitor>,
        baseline: Option<&BaselineProfile>,
    ) -> Result<Vec<ActiveVulnFinding>, String>;
}
