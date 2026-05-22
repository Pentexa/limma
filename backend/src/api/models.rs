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

    pub profile_id: Option<String>,
}

// ── Active Vulnerability Detection Models ──

#[derive(Deserialize)]

pub struct ActiveScanApiRequest {
    pub target_url: String,
    pub vuln_types: Vec<crate::domain::active_vuln::ActiveVulnType>,
    pub max_duration_seconds: Option<u32>,
    pub rate_limit_rps: Option<u32>,
    pub follow_redirects: Option<bool>,
    pub profile_id: Option<String>,
    pub custom_parameters: Option<Vec<String>>,
}
