use anyhow::{Context, Result};
use std::time::Duration;

/// Metadata injected by the CI/CD environment (GitHub Action, GitLab CI, etc.)
pub struct ScanMetadata {
    pub repo: Option<String>,
    pub sha: Option<String>,
    pub git_ref: Option<String>,
    pub run_id: Option<String>,
}

/// HTTP client that communicates with the Limma backend API.
pub struct LimmaClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl LimmaClient {
    /// Creates a new client pointing at the given backend base URL.
    pub fn new(base_url: &str, api_key: Option<&str>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(600)) // 10-minute default hard timeout
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.map(|s| s.to_string()),
        })
    }

    /// Runs a full security scan via the backend's `/master-report` endpoint.
    pub async fn scan(
        &self,
        target: &str,
        timeout_minutes: u64,
        ci_mode: bool,
        enable_correlation: bool,
        metadata: ScanMetadata,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/master-report", self.base_url);

        let scan_request = serde_json::json!({
            "url": target,
            "ci_mode": ci_mode,
            "enable_correlation": enable_correlation,
            "metadata": {
                "repo": metadata.repo,
                "sha": metadata.sha,
                "ref": metadata.git_ref,
                "run_id": metadata.run_id,
            }
        });

        let mut request = self
            .client
            .post(&url)
            .timeout(Duration::from_secs(timeout_minutes * 60))
            .json(&scan_request);

        // API key is a placeholder for future authenticated scans
        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        eprintln!("🚀 Starting scan against {}...", target);

        let response = request
            .send()
            .await
            .context("Failed to connect to Limma backend. Is the server running?")?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            anyhow::bail!("Backend returned HTTP {}: {}", status.as_u16(), body);
        }

        let result: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse scan response as JSON")?;

        Ok(result)
    }
}
