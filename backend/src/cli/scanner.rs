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

    /// Attach authorization header if api_key is set.
    fn authorize(
        &self,
        request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        if let Some(ref key) = self.api_key {
            request.header("Authorization", format!("Bearer {}", key))
        } else {
            request
        }
    }

    // ── Full Scan ────────────────────────────────────────────────────────────

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

        let request = self
            .client
            .post(&url)
            .timeout(Duration::from_secs(timeout_minutes * 60))
            .json(&scan_request);

        let request = self.authorize(request);

        eprintln!("🚀 Starting scan against {}...", target);

        let response = request
            .send()
            .await
            .context("Failed to connect to Limma backend. Is the server running?")?;

        self.check_response(response).await
    }

    // ── Individual Module Endpoints ──────────────────────────────────────────

    /// Run website analysis only via POST /analyze
    pub async fn analyze(&self, target: &str) -> Result<serde_json::Value> {
        self.post_url_request("/analyze", target).await
    }

    /// Run server investigation only via POST /investigate
    pub async fn investigate(&self, target: &str) -> Result<serde_json::Value> {
        self.post_url_request("/investigate", target).await
    }

    /// Run API discovery only via POST /discover-apis
    pub async fn discover_apis(&self, target: &str) -> Result<serde_json::Value> {
        self.post_url_request("/discover-apis", target).await
    }

    /// Run service collector only via POST /collect-services
    pub async fn collect_services(&self, target: &str) -> Result<serde_json::Value> {
        self.post_url_request("/collect-services", target).await
    }

    /// Run security audit only via POST /audit-security
    pub async fn audit_security(&self, target: &str) -> Result<serde_json::Value> {
        self.post_url_request("/audit-security", target).await
    }

    /// Run form mapper only via POST /map-forms
    pub async fn map_forms(&self, target: &str) -> Result<serde_json::Value> {
        self.post_url_request("/map-forms", target).await
    }

    // ── History / Results ────────────────────────────────────────────────────

    /// Fetch a single scan result by ID via GET /api/history/scan/:scan_id
    pub async fn get_scan_by_id(&self, scan_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/history/scan/{}", self.base_url, scan_id);
        let request = self.authorize(self.client.get(&url));
        let response = request
            .send()
            .await
            .context("Failed to connect to Limma backend")?;
        self.check_response(response).await
    }

    /// List all scans via GET /api/history/scans
    pub async fn list_scans(
        &self,
        target_url: Option<&str>,
        limit: Option<i64>,
    ) -> Result<serde_json::Value> {
        let mut url = format!("{}/api/history/scans", self.base_url);
        let mut params = Vec::new();
        if let Some(t) = target_url {
            params.push(format!("target_url={}", urlencoding::encode(t)));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let request = self.authorize(self.client.get(&url));
        let response = request
            .send()
            .await
            .context("Failed to connect to Limma backend")?;
        self.check_response(response).await
    }

    /// Get trend data via GET /api/history/trends
    pub async fn get_trends(&self, target_url: &str) -> Result<serde_json::Value> {
        let url = format!(
            "{}/api/history/trends?target_url={}",
            self.base_url,
            urlencoding::encode(target_url)
        );
        let request = self.authorize(self.client.get(&url));
        let response = request
            .send()
            .await
            .context("Failed to connect to Limma backend")?;
        self.check_response(response).await
    }

    /// Get delta comparison via GET /api/history/delta
    pub async fn get_delta(
        &self,
        target_url: &str,
        current_scan_id: &str,
        previous_scan_id: &str,
    ) -> Result<serde_json::Value> {
        let url = format!(
            "{}/api/history/delta?target_url={}&current_scan_id={}&previous_scan_id={}",
            self.base_url,
            urlencoding::encode(target_url),
            current_scan_id,
            previous_scan_id
        );
        let request = self.authorize(self.client.get(&url));
        let response = request
            .send()
            .await
            .context("Failed to connect to Limma backend")?;
        self.check_response(response).await
    }

    // ── Rule Engine ──────────────────────────────────────────────────────────

    /// Get rule engine status via GET /api/rule-engine-status
    pub async fn get_rule_engine_status(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/rule-engine-status", self.base_url);
        let request = self.authorize(self.client.get(&url));
        let response = request
            .send()
            .await
            .context("Failed to connect to Limma backend")?;
        self.check_response(response).await
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Generic POST with `{ "url": target }` body.
    async fn post_url_request(
        &self,
        endpoint: &str,
        target: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, endpoint);
        let body = serde_json::json!({ "url": target });
        let request = self.authorize(self.client.post(&url).json(&body));
        let response = request
            .send()
            .await
            .with_context(|| format!("Failed to connect to Limma backend at {}", endpoint))?;
        self.check_response(response).await
    }

    /// Check HTTP response status and parse JSON body.
    async fn check_response(
        &self,
        response: reqwest::Response,
    ) -> Result<serde_json::Value> {
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
            .context("Failed to parse response as JSON")?;

        Ok(result)
    }
}
