//! Shared DTOs between limma-backend and limma-cli.
//!
//! These types are used for API request/response serialization.
//! They are intentionally kept lightweight — no database or infrastructure dependencies.

use serde::{Deserialize, Serialize};

// ── Scan Metadata (CI/CD) ────────────────────────────────────────────────────

/// Metadata injected by the CI/CD environment (GitHub Action, GitLab CI, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanMetadata {
    pub repo: Option<String>,
    pub sha: Option<String>,
    pub git_ref: Option<String>,
    pub run_id: Option<String>,
}

// ── Master Report Request ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterReportRequest {
    pub url: String,
    #[serde(default)]
    pub ci_mode: bool,
    #[serde(default)]
    pub enable_correlation: bool,
    #[serde(default)]
    pub metadata: Option<ScanMetadata>,
}

// ── URL-only Request ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlRequest {
    pub url: String,
}

// ── Active Scan Request ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveScanRequest {
    pub target_url: String,
    #[serde(default)]
    pub vuln_types: Vec<String>,
    #[serde(default = "default_max_duration")]
    pub max_duration_seconds: u32,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_rps: u32,
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    pub profile_id: Option<String>,
    #[serde(default)]
    pub enable_waf_bypass: bool,
    #[serde(default = "default_true")]
    pub safe_mode: bool,
    #[serde(default)]
    pub custom_parameters: Option<Vec<String>>,
}

fn default_max_duration() -> u32 {
    300
}
fn default_rate_limit() -> u32 {
    10
}
fn default_true() -> bool {
    true
}

// ── Blind Scan Request ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindScanRequest {
    pub target_url: String,
    pub target_id: Option<String>,
    pub scan_types: Option<Vec<String>>,
}

// ── PoC Generation Request ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PocGenerateRequest {
    pub finding_id: String,
    pub preferred_language: Option<String>,
}

// ── Exploit Verify Request ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitVerifyRequest {
    pub poc_id: String,
    pub execution_mode: Option<String>, // "dry_run", "sandbox", "active"
}

// ── Export Request ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub target: String,
    pub scan_id: Option<String>,
}

// ── Settings Update Request ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsUpdateRequest {
    pub key: String,
    pub value: serde_json::Value,
}

// ── Output Format ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Markdown,
    Sarif,
}
