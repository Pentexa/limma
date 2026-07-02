use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AnalysisRequest {
    pub url: String,
    pub profile_id: Option<String>,
    #[serde(default)]
    pub save_to_history: Option<bool>,
}

#[derive(Deserialize)]
pub struct ProxyRequest {
    pub url: String,
    pub method: String,
    pub body: Option<String>,
}

#[derive(Deserialize)]
pub struct VerifyPortRequest {
    pub host: String,
    pub port: u16,
}

#[derive(Serialize)]
pub struct VerifyPortResponse {
    pub is_active: bool,
    pub latency_ms: Option<u64>,
    pub banner: Option<String>,
}

#[derive(Deserialize)]
pub struct FeedbackRequest {
    pub signature: String,
    pub action: crate::domain::entities::FeedbackAction,
}

// ── Faz F: Blind Detection & Exploitation Models ──

#[derive(Deserialize)]
pub struct BlindScanApiRequest {
    pub target_url: String,
    pub detection_types: Vec<crate::domain::entities::BlindVulnType>,
    pub scan_id: Option<uuid::Uuid>,
    pub target_id: Option<uuid::Uuid>,
    pub max_duration_seconds: Option<u32>,
    /// Session cookie supplied by the authenticated scan context. It is only
    /// used for authenticated-vs-anonymous cache deception comparisons.
    pub cookie: Option<String>,

    pub profile_id: Option<String>,
}

// ── Active Vulnerability Detection Models ──

#[derive(Deserialize)]

pub struct ActiveScanApiRequest {
    pub target_url: String,
    pub vuln_types: Vec<crate::domain::active_vuln::ActiveVulnType>,
    pub scan_mode: String,
    pub enable_headless_browser: bool,
    pub max_browser_tabs: u32,
    pub bearer_token: Option<String>,
    pub cookie: Option<String>,
    pub custom_headers: Option<String>,
    pub basic_auth_user: Option<String>,
    pub basic_auth_pass: Option<String>,
    pub enable_json_fuzzing: bool,
    pub enable_xss_verification: bool,
    pub allow_destructive_methods: bool,
    pub l3_consent_accepted: bool,
    pub max_scan_duration_sec: Option<u32>,
    pub max_requests_per_endpoint: Option<u32>,
    pub follow_redirects: Option<bool>,
    pub profile_id: Option<String>,
    pub custom_parameters: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct SubdomainDiscoveryRequest {
    pub domain: String,
    pub profile_id: Option<String>,
}

fn default_validate_assets() -> bool {
    true
}

#[derive(Deserialize)]
pub struct DiscoverCertificatesRequest {
    pub domain: String,
    pub profile_id: Option<String>,
    #[serde(default = "default_validate_assets")]
    pub validate_assets: bool,
}

#[derive(Serialize)]
pub struct DiscoverCertificatesResponse {
    pub total_cert_names: usize,
    pub unique_candidates: usize,
    pub wildcard_removed: usize,
    pub out_of_scope_removed: usize,
    pub validated_assets: usize,
    pub assets: Vec<crate::domain::entities::SubdomainAsset>,
    pub source: String,
    pub warnings: Vec<String>,
    pub scan_duration_ms: u64,
}
