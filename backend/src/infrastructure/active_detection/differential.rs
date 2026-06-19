use reqwest::Client;

use crate::infrastructure::active_detection::evidence::response_diff::ResponseDiffAnalyzer;
use crate::infrastructure::active_detection::timing::baseline_analyzer::BaselineAnalyzer;

#[derive(Debug, Clone)]
pub struct BaselineProfile {
    pub status_code: u16,
    pub content_length: usize,
    pub response_body: String,

    pub response_time_ms: u64,
    pub average_response_time_ms: u64,
    pub body_hash: String,
    pub header_fingerprint: Vec<String>,
    pub error_rate: f32,
    pub redirect_location: Option<String>,
}

impl BaselineProfile {
    /// Compares a new response to the baseline to detect structural changes.
    /// Returns true if the difference is significant (e.g. content length delta > 5% and different status).
    pub fn is_significantly_different(&self, new_status: u16, new_body: &str) -> bool {
        ResponseDiffAnalyzer::compare_to_baseline(self, new_status, new_body).is_significant()
    }

    /// Checks if the baseline naturally contains a certain indicator (e.g. "syntax error")
    pub fn contains_indicator(&self, indicator: &str) -> bool {
        self.response_body.contains(indicator)
    }
}

/// Builds a baseline profile for a given target URL and parameter.
pub async fn build_baseline(
    client: &Client,
    target_url: &str,
    param: &str,
    safe_value: &str,
) -> Result<BaselineProfile, String> {
    BaselineAnalyzer::build(client, target_url, param, safe_value).await
}
