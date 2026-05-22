use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct BaselineProfile {
    pub status_code: u16,
    pub content_length: usize,
    pub response_body: String,
    
    pub response_time_ms: u64,
}

impl BaselineProfile {
    /// Compares a new response to the baseline to detect structural changes.
    /// Returns true if the difference is significant (e.g. content length delta > 5% and different status).
    pub fn is_significantly_different(&self, new_status: u16, new_body: &str) -> bool {
        if self.status_code != new_status {
            return true;
        }

        let new_len = new_body.len();
        let diff_ratio = if self.content_length > 0 {
            (self.content_length as f64 - new_len as f64).abs() / (self.content_length as f64)
        } else {
            if new_len > 0 {
                1.0
            } else {
                0.0
            }
        };

        diff_ratio > 0.05 // 5% delta threshold
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
    let start_time = std::time::Instant::now();
    let resp = client
        .get(target_url)
        .query(&[(param, safe_value)])
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let elapsed = start_time.elapsed().as_millis() as u64;
    let status_code = resp.status().as_u16();
    let response_body = resp.text().await.unwrap_or_default();
    let content_length = response_body.len();

    Ok(BaselineProfile {
        status_code,
        content_length,
        response_body,
        response_time_ms: elapsed,
    })
}
