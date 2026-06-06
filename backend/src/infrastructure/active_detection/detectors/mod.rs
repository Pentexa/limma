pub mod cmdi_detector;
pub mod deser_detector;
pub mod idor_detector;
pub mod jwt_detector;
pub mod lfi_detector;
pub mod nosql_detector;
pub mod redirect_detector;
pub mod sqli_detector;
pub mod ssrf_detector;
pub mod ssti_detector;
pub mod xss_detector;
pub mod xxe_detector;

use crate::domain::active_vuln::{ActiveVulnFinding, ActiveVulnType};
use crate::infrastructure::active_detection::differential::BaselineProfile;
use crate::infrastructure::safety::waf_monitor::WafMonitor;
use crate::domain::fuzzing::{EndpointContext, InsertionPoint};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Common trait for all vulnerability detectors.
#[async_trait]
#[allow(clippy::too_many_arguments)]
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
        endpoint_ctx: Option<&EndpointContext>,
        insertion_point: Option<&InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String>;
}

/// Factory function to build all detectors with a specific HTTP client.
/// This allows each scan to have its own authenticated client instance.
pub fn build_detectors(client: reqwest::Client) -> Vec<Box<dyn VulnDetector>> {
    vec![
        Box::new(xss_detector::XssDetector::new(client.clone())),
        Box::new(sqli_detector::SqliDetector::new(client.clone())),
        Box::new(cmdi_detector::CmdiDetector::new(client.clone())),
        Box::new(lfi_detector::LfiDetector::new(client.clone())),
        Box::new(ssrf_detector::SsrfDetector::new(client.clone())),
        Box::new(xxe_detector::XxeDetector::new(client.clone())),
        Box::new(redirect_detector::RedirectDetector::new(client.clone())),
        Box::new(jwt_detector::JwtDetector::new(client.clone())),
        Box::new(deser_detector::DeserDetector::new(client.clone())),
        Box::new(idor_detector::IdorDetector::new(client.clone())),
        Box::new(nosql_detector::NosqlDetector::new(client.clone())),
        Box::new(ssti_detector::SstiDetector::new(client.clone())),
    ]
}
