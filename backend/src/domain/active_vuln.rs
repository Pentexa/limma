use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ── Active Vulnerability Types ──

/// 28 aktif tespit edilebilen açık türü
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveVulnType {
    // XSS Family
    ReflectedXss,
    StoredXss,
    DomXss,

    // SQL Injection Family
    SqlInjectionError,
    SqlInjectionUnion,
    SqlInjectionBlindTime,
    SqlInjectionBlindBoolean,

    // Command Injection
    CommandInjection,
    CommandInjectionBlind,

    // Path Traversal / File Inclusion
    LocalFileInclusion,
    RemoteFileInclusion,
    PathTraversal,

    // Server-Side Request Forgery
    ServerSideRequestForgery,

    // XML External Entity
    XmlExternalEntity,

    // Insecure Deserialization
    InsecureDeserializationJava,
    InsecureDeserializationPhp,
    InsecureDeserializationPython,

    // Open Redirect
    OpenRedirect,

    // Insecure Direct Object Reference
    InsecureDirectObjectReference,

    // JWT Attacks
    JwtNoneAlgorithm,
    JwtWeakSecret,

    // GraphQL
    // GraphQL
    GraphqlIntrospectionEnabled,
    GraphqlAbuse,

    // Misconfiguration
    HostHeaderInjection,
    CorsMisconfiguration,
    HttpRequestSmuggling,
    CacheDeception,

    // Phase 4 New Vulns
    NoSqlInjection,
    ServerSideTemplateInjection,
}

impl ActiveVulnType {
    /// Returns the default severity for this vulnerability type.
    #[allow(dead_code)]
    pub fn default_severity(&self) -> crate::domain::entities::SeverityLevel {
        use ActiveVulnType::*;
        match self {
            SqlInjectionError | SqlInjectionUnion | SqlInjectionBlindTime | SqlInjectionBlindBoolean
            | CommandInjection | CommandInjectionBlind
            | RemoteFileInclusion
            | ServerSideRequestForgery
            | XmlExternalEntity
            | InsecureDeserializationJava | InsecureDeserializationPhp | InsecureDeserializationPython
            | HttpRequestSmuggling | ServerSideTemplateInjection => crate::domain::entities::SeverityLevel::Critical,

            ReflectedXss | StoredXss | DomXss
            | LocalFileInclusion | PathTraversal
            | InsecureDirectObjectReference
            | JwtNoneAlgorithm | JwtWeakSecret
            | HostHeaderInjection | NoSqlInjection | GraphqlAbuse => crate::domain::entities::SeverityLevel::High,

            OpenRedirect | CorsMisconfiguration | CacheDeception => crate::domain::entities::SeverityLevel::Medium,

            GraphqlIntrospectionEnabled => crate::domain::entities::SeverityLevel::Low,
        }
    }
}

// ── Exploitability ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExploitabilityLevel {
    /// Directly exploitable with a simple request
    Actionable,
    /// Exploitable with some conditions (e.g. user interaction)
    Conditional,
    /// Theoretical exploitation, hard to reproduce
    Theoretical,
}

// ── Payload System ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadDefinition {
    pub id: String,
    pub payload: String,
    pub description: String,
    pub expected_indicator: ExpectedIndicator,
    pub severity: crate::domain::entities::SeverityLevel,
    pub safe_for_production: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedIndicator {
    /// SQL/runtime error message pattern in response body
    ErrorPattern(String),
    /// Payload reflected back in response without encoding
    ReflectedContent(String),
    /// Response time delay (ms) indicating time-based blind injection
    TimeDelay(u64),
    /// Known file content appears in response (e.g. /etc/passwd root line)
    FileContent(String),
    /// Specific HTTP status code expected
    StatusCode(u16),
    /// Response body differs significantly between true/false condition
    ResponseDiff { baseline_hash: String, indicator: String },
    /// Location header redirects to attacker-controlled URL
    RedirectLocation(String),
    /// JWT accepted with manipulated algorithm/payload
    JwtAccepted,
}

// ── Scan Configuration ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveScanConfig {
    pub target_url: String,
    pub vuln_types: Vec<ActiveVulnType>,
    pub max_duration_seconds: u32,
    pub rate_limit_rps: u32,
    pub follow_redirects: bool,
    pub profile_id: Option<String>,
    pub enable_waf_bypass: bool,
    pub safe_mode: bool,
    pub custom_parameters: Option<Vec<String>>,
}

// ── Scan Status ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// ── Active Vulnerability Finding ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveVulnFinding {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub vuln_type: ActiveVulnType,
    pub target_url: String,
    pub affected_parameter: String,
    pub http_method: String,
    pub payload_used: String,
    pub evidence: ActiveVulnEvidence,
    pub severity: crate::domain::entities::SeverityLevel,
    pub confidence: crate::domain::entities::ConfidenceLevel,
    pub exploitability: ExploitabilityLevel,
    pub poc_generated: bool,
    pub poc_id: Option<Uuid>,
    pub verified: bool,
    pub false_positive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveVulnEvidence {
    pub request_raw: String,
    pub response_raw: String,
    pub response_time_ms: u64,
    pub matched_indicator: String,
    pub additional_notes: Vec<String>,
}

// ── Scan Result ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveScanResult {
    pub scan_id: Uuid,
    pub target_url: String,
    pub status: ActiveScanStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_requests: u32,
    pub findings: Vec<ActiveVulnFinding>,
    pub errors: Vec<String>,
    pub summary: ScanSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanSummary {
    pub critical_count: u32,
    pub high_count: u32,
    pub medium_count: u32,
    pub low_count: u32,
    pub info_count: u32,
    pub vuln_type_breakdown: HashMap<String, u32>,
    pub waf_detected: bool,
    pub waf_blocked_requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveFindingQueryParams {
    pub scan_id: Option<Uuid>,
    pub vuln_type: Option<ActiveVulnType>,
    pub severity: Option<crate::domain::entities::SeverityLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanQueryParams {
    pub target_url: Option<String>,
    pub status: Option<ActiveScanStatus>,
}
