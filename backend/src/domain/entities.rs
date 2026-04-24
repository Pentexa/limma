use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(default, skip_serializing)]
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Epistemic Honesty: Global Certainty Primitives ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CertaintyLevel {
    Certain,   // Doğrudan kanıtla doğrulanmış
    Likely,    // Güçlü göstergeler var ama tam doğrulama yok
    Uncertain, // Zayıf kanıt, tahmin içeriyor
    Unknown,   // Bilgi yok — bilmiyorum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertaintyNote {
    pub level: CertaintyLevel,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectChainEntry {
    pub url: String,
    pub status_code: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechEvidence {
    pub source: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTechnology {
    pub name: String,
    pub category: String,
    pub confidence_score: f32,
    pub evidences: Vec<TechEvidence>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityHeaderStatus {
    Present,
    Missing,
    Weak,
    Misconfigured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeaderResult {
    pub name: String,
    pub status: SecurityHeaderStatus,
    pub value: Option<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskInsight {
    pub title: String,
    pub severity: RiskSeverity,
    pub explanation: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedPage {
    pub url: String,
    pub status_code: u16,
    pub latency_ms: u64,
    pub headers: std::collections::HashMap<String, String>,
    pub content_type: Option<String>,
    pub detected_technologies: Vec<DetectedTechnology>,
    pub security_headers: Vec<SecurityHeaderResult>,
    pub risk_insights: Vec<RiskInsight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String, // e.g. "SCAN_STARTED", "PAGE_CRAWLED", "RISK_GENERATED"
    pub level: String,      // "INFO", "WARN", "ERROR"
    pub message: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_pages: u32,
    pub average_latency_ms: u64,
    pub common_technologies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedRisk {
    pub title: String,
    pub severity: RiskSeverity,
    pub explanation: String,
    pub evidences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationReport {
    pub overall_risk_score: u8,
    pub correlated_risks: Vec<CorrelatedRisk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebScanResult {
    pub original_target_url: String,
    pub final_url: String,
    pub scan_start_time: chrono::DateTime<chrono::Utc>,
    pub scan_end_time: chrono::DateTime<chrono::Utc>,
    pub total_duration_ms: u64,
    pub final_status_code: u16,
    pub latency_ms: u64,
    pub redirect_count: u32,
    pub redirect_chain: Vec<RedirectChainEntry>,
    pub headers: std::collections::HashMap<String, String>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub server: Option<String>,
    pub cache_control: Option<String>,
    pub detected_technologies: Vec<DetectedTechnology>,
    pub security_headers: Vec<SecurityHeaderResult>,
    pub risk_insights: Vec<RiskInsight>,
    pub security_score: u8,
    pub pages: Vec<ScannedPage>,
    pub timeline: Vec<ScanEvent>,
    pub summary: Option<ScanSummary>,
    pub correlation: Option<CorrelationReport>,
    pub scan_certainty: Option<CertaintyNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigatorFingerprint {
    pub name: String,
    pub category: String, // "CMS", "Deployment Provider", "Framework"
    pub confidence_score: f32,
    pub evidences: Vec<String>,
    pub explanation: String,
    pub certainty: Option<CertaintyLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureSignal {
    pub signal_type: String,
    pub value: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryInsight {
    pub name: String,
    pub category: String, // "Cache Behavior", "Edge/CDN Signal", "Proxy/Gateway Indicator"
    pub confidence_score: f32,
    pub evidence: String,
    pub explanation: String,
    pub certainty: Option<CertaintyLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPostureInsight {
    pub name: String,
    pub category: String, // "TLS & Transport", "Security Header Interpretation", "Infrastructure Exposure"
    pub status: String,   // "Secure", "Warning", "Critical", "Informational"
    pub confidence_score: f32,
    pub evidence: String,
    pub explanation: String,
    pub certainty: Option<CertaintyLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyInsight {
    pub name: String,
    pub severity: String, // "High", "Medium", "Low", "Informational"
    pub category: String, // "Security Header Consistency", "Cache Consistency", "Platform Uniformity"
    pub evidences: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub original_target: String,
    pub resolved_url: String,
    pub status_code: u16,
    pub latency_ms: u64,
    pub raw_headers: std::collections::HashMap<String, Vec<String>>,
    pub categorized_headers:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
    pub infrastructure_signals: Vec<InfrastructureSignal>,
    pub fingerprints: Vec<InvestigatorFingerprint>,
    pub delivery_insights: Vec<DeliveryInsight>,
    pub security_insights: Vec<SecurityPostureInsight>,
    pub routes_checked: Vec<String>,
    pub consistency_insights: Vec<ConsistencyInsight>,
    pub activity_log: Vec<String>,
    pub investigation_certainty: Option<CertaintyNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationEvent {
    pub timestamp: String,
    pub event_type: String, // e.g. "INFRA_SIGNAL_DETECTED", "CMS_FINGERPRINT_MATCHED"
    pub message: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    pub source_type: String,
    pub snippet: String,
    pub reason: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDetail {
    pub name: String,
    pub param_type: String, // "query", "body", "path"
    pub data_type: String,  // "string", "email", "token", "id", "number"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetrics {
    pub total_endpoints: usize,
    pub valid_endpoints: usize,
    pub false_positives: usize,
    pub precision: f32, // Valid / (Valid + False Positives)
    pub source_distribution: std::collections::HashMap<String, f32>, // percentages of different source types
    pub confidence_accuracy_correlation: f32, // Score indicating how well confidence reflects runtime reality
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVerification {
    pub is_valid: bool,
    pub best_method: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub has_body: bool,
    pub content_type: Option<String>,
    pub server: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDetail {
    pub path: String,
    pub method_prediction: String,
    pub parameters: Vec<ParamDetail>,
    pub auth_probability: f32,   // 0.0 to 1.0
    pub auth_likelihood: String, // "None", "Low", "Likely"
    pub confidence_score: f32,   // 0.3 to 0.95
    pub evidences: Vec<Evidence>,
    pub runtime_verification: Option<RuntimeVerification>,
    pub certainty: Option<CertaintyLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDiscoveryResult {
    pub base_url: String,
    pub detected_endpoints: Vec<EndpointDetail>,
    pub suspected_api_technologies: Vec<String>,
    pub metrics: Option<DiscoveryMetrics>,
    pub discovery_certainty: Option<CertaintyNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PortState {
    Open,
    Closed,
    Filtered,
    Timeout,
    Unreachable,
    Ambiguous,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivitySeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeMethod {
    Banner,
    Http,
    Tls,
    Greeting,
    PortDefault,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CollectorStatus {
    Completed,
    PartialFailure,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInput {
    pub original_input: String,
    pub normalized_url: String,
    pub host: String,
    pub scheme: Option<String>,
    pub default_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTarget {
    pub ip_addresses: Vec<String>,
    pub primary_ip: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSummary {
    pub has_tls: bool,
    pub protocol_version: Option<String>,
    pub cipher_suite: Option<String>,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    pub alpn: Option<String>,
    pub sni_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSummary {
    pub status_code: Option<u16>,
    pub server_header: Option<String>,
    pub content_type: Option<String>,
    pub redirect_target: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
    pub response_length: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    ProtocolGreeting, // SSH banner, MySQL handshake, etc.
    TlsHandshake,     // TLS metadata (ALPN, cert, cipher)
    HttpResponse,     // HTTP status, headers
    BannerText,       // Raw text banner
    PortAssumption,   // Default port mapping
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStrength {
    Strong, // Protocol greeting, TLS ALPN match
    Medium, // HTTP headers, banner patterns
    Weak,   // Port-based assumption only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub kind: EvidenceKind,
    pub strength: EvidenceStrength,
    pub source: ProbeMethod,
    pub raw_signal: String,
    pub interpretation: String,
    pub suggests_service: Option<String>,
    pub is_negative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionOutcome {
    Verified,        // Strongly confirmed active service (Truth Layer)
    Suspected,       // Inferred/guessed but unconfirmed service
    CdnEdge,         // Hosted behind CDN proxy/waf
    RoutingBehavior, // 301/302 redirects
    Filtered,        // Detected as filtered
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityReason {
    pub description: String,
    pub conflicting_evidence: Vec<String>,
}

// ── Phase 4+5: Fingerprint System Types ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleCategory {
    BannerContains,
    BannerStartsWith,
    TlsPresent,
    TlsAlpnContains,
    TlsCertSubjectContains,
    HttpStatusRange,
    HttpServerContains,
    HttpContentTypeContains,
    GreetingSignature,
    PortBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleWeight {
    Critical,   // Protocol-level proof (greeting, ALPN) — dominates scoring
    Strong,     // High-quality signal (TLS cert, specific banner)
    Medium,     // Moderate signal (HTTP headers, generic banner)
    Weak,       // Low-quality hint (port number alone)
    Contextual, // Only applies when a prerequisite rule already matched
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FingerprintTier {
    Specific, // Concrete service (e.g., OpenSSH, MySQL 8.x)
    Generic,  // Category (e.g., "HTTP server", "Database")
    Fallback, // Catch-all (e.g., "Unknown TLS", "Unknown service")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CoverageLevel {
    Full,    // All rules evaluated and matched
    High,    // Required rules + most optional matched
    Partial, // Some rules matched, gaps remain
    Minimal, // Very few rules matched
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FingerprintConfidence {
    Confirmed, // Strong protocol-level proof
    High,      // Multiple aligned signals
    Medium,    // Moderate evidence
    Low,       // Weak or partial evidence
    Tentative, // Partial match, may be wrong
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationItem {
    pub category: String, // "boost", "decay", "penalty", "info", "contextual"
    pub description: String,
    pub impact: f32, // positive = boost, negative = penalty
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPenalty {
    pub reason: String,
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintRule {
    pub category: RuleCategory,
    pub expected_value: String,
    pub weight: f32,
    pub rule_weight: RuleWeight,
    pub required: bool,
    pub description: String,
    pub contextual_requires: Option<RuleCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintDefinition {
    pub id: String,
    pub service_name: String,
    pub description: String,
    pub tier: FingerprintTier,
    pub rules: Vec<FingerprintRule>,
    pub min_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchStrength {
    Full,
    Strong,
    Partial,
    Weak,
    NoMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluation {
    pub category: RuleCategory,
    pub expected: String,
    pub actual: Option<String>,
    pub matched: bool,
    pub weight: f32,
    pub rule_weight: RuleWeight,
    pub contribution: f32,
    pub skipped_contextual: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintMatch {
    pub fingerprint_id: String,
    pub service_name: String,
    pub tier: FingerprintTier,
    pub strength: MatchStrength,
    pub confidence: f32,
    pub confidence_level: FingerprintConfidence,
    pub coverage: CoverageLevel,
    pub matched_rules: Vec<RuleEvaluation>,
    pub missing_rules: Vec<RuleEvaluation>,
    pub conflicting_rules: Vec<RuleEvaluation>,
    pub explanation_items: Vec<ExplanationItem>,
    pub penalties: Vec<MatchPenalty>,
    pub reasoning: String,
}

// Keep ProbeEvidence as a simpler alias used inside probes, then converted to EvidenceItem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEvidence {
    pub method: ProbeMethod,
    pub raw_signal: String,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceBreakdown {
    pub port_evidence: f32,
    pub protocol_validation: f32,
    pub fingerprint_strength: f32,
    pub header_reliability: f32,
    pub redirect_penalty: f32,
    pub cdn_penalty: f32,
    pub response_quality: f32,
    pub final_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTreeStep {
    pub step: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCandidate {
    pub service_name: String,
    pub confidence_breakdown: ConfidenceBreakdown,
    pub decision: DecisionOutcome,
    pub probe_method: ProbeMethod,
    pub supporting_evidence: Vec<EvidenceItem>,
    pub conflicting_evidence: Vec<EvidenceItem>,
    pub reasoning: String,
    pub tls_summary: Option<TlsSummary>,
    pub http_summary: Option<HttpSummary>,
    pub ambiguity: Option<AmbiguityReason>,
    pub fingerprint_match: Option<FingerprintMatch>,
    pub verification_trail: Vec<DecisionTreeStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortProbeResult {
    pub port: u16,
    pub state: PortState,
    pub latency_ms: Option<u64>,
    pub service_candidates: Vec<ServiceCandidate>,
    pub all_evidence: Vec<EvidenceItem>,
    pub fingerprint_evaluations: Vec<FingerprintMatch>,
    pub fallback_used: bool,
    pub retry_count: u8,
    pub probe_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: ActivitySeverity,
    pub event_type: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

// ── Phase 6: Snapshot History & Diffing Types ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Removed,
    Changed,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub change_type: ChangeType,
    pub resource: String, // e.g., "Port 80", "Service on Port 443"
    pub before: Option<String>,
    pub after: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub previous_timestamp: chrono::DateTime<chrono::Utc>,
    pub current_timestamp: chrono::DateTime<chrono::Utc>,
    pub changes: Vec<ChangeEvent>,
    pub summaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorSnapshot {
    pub target_input: TargetInput,
    pub resolved_target: ResolvedTarget,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub port_results: Vec<PortProbeResult>,
    pub activity_timeline: Vec<ActivityEvent>,
    pub errors: Vec<String>,
    pub overall_status: CollectorStatus,
    pub diff: Option<SnapshotDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub url: String,
    pub security_score: u8,
    pub missing_headers: Vec<String>,
    pub robot_rules_disallowed: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormMapping {
    pub url: String,
    pub detected_forms: Vec<DetectedForm>,
    pub login_pages_found: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedForm {
    pub action: String,
    pub method: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterReport {
    pub url: String,
    pub analysis: Option<WebScanResult>,
    pub server_info: Option<ServerInfo>,
    pub api_discovery: Option<ApiDiscoveryResult>,
    pub service_collector: Option<CollectorSnapshot>,
    pub security_audit: Option<SecurityReport>,
    pub normalized_audit: Option<NormalizedAuditReport>,
    pub form_mapping: Option<FormMapping>,
    pub scan_strategy: Option<Vec<ScanStrategyDecision>>,
    pub overall_health_score: u8,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub module_errors: Vec<String>,
}

// ── Phase 1: Normalized Security Auditor Data Models ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SourceModule {
    WebScanner,
    ServerInvestigator,
    ApiDiscoverer,
    SecurityAuditor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    SecurityMisconfiguration,
    InformationDisclosure,
    InsecureTransport,
    AuthenticationBypass,
    SuspiciousEndpoint,
    InfrastructureLeak,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SeverityLevel {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Certain,
    Firm,
    #[default]
    Tentative,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Open,
    Accepted,
    Mitigated,
    FalsePositive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceWeight {
    Strong,
    Moderate,
    Weak,
    Zero,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Exploitability {
    Proven,
    Likely,
    Possible,
    Unlikely,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvidenceItem {
    pub description: String,
    pub validation_context: Option<String>,
    pub raw_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedFindingReference {
    pub id: String,
    pub category: FindingCategory,
    pub severity: SeverityLevel,
    pub short_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditFinding {
    pub id: String, // Uuid as string for simplicity
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub target_identifier: String,
    pub affected_path_or_endpoint: Option<String>,
    pub protocol: Option<String>,
    pub method: Option<String>,
    pub category: FindingCategory,
    pub severity: SeverityLevel,
    pub confidence: ConfidenceLevel,
    pub status: FindingStatus,
    pub summary: String,
    pub technical_details: String,
    pub source_module: SourceModule,
    pub evidence: Vec<AuditEvidenceItem>,
    pub raw_reference: Option<serde_json::Value>,

    // Correlation Enhancement Info
    pub correlation_group_id: Option<String>,
    pub correlation_count: usize,
    pub correlation_type: Option<CorrelationType>,
    pub correlation_confidence: Option<ConfidenceLevel>,
    pub correlation_summary: Option<String>,
    pub correlation_is_hygiene_gap: bool,
    pub related_findings: Vec<CorrelatedFindingReference>,

    // Risk Scoring (Phase 4)
    pub risk_score: Option<RiskScore>,

    // Context-Aware Assessment (Phase 5)
    pub exploitability: Option<Exploitability>,
    pub context_summary: Option<String>,
    pub evidence_weight: Option<EvidenceWeight>,
    pub context_assessment: Option<ContextAwareAssessment>,
    pub certainty: Option<CertaintyLevel>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Unverified,
    PartiallyVerified,
    VerifiedActionable,
    VerifiedInert,
    VerificationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationTrace {
    pub endpoint: String,
    pub method: String,
    pub request_snapshot: String,
    pub response_snapshot: String,
    pub is_successful: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveVerificationData {
    pub status: VerificationStatus,
    pub reasoning: String,
    pub reproducibility_score: u8,
    pub traces: Vec<VerificationTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExploitabilityLevel {
    Actionable,
    Theoretical,
    Inert,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackAction {
    VerifiedTruePositive,
    FalsePositive,
    Ignored,
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEvent {
    pub action: FeedbackAction,
    pub timestamp_sec: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternReliabilityHistory {
    pub total_feedback_events: u32,
    pub fp_weight: f32,
    pub tp_weight: f32,
    pub ignore_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternCalibrationMetrics {
    pub total_observations: u32,
    pub successful_verifications: u32,
    pub failed_verifications: u32,
    pub partial_verifications: u32,
    pub average_reproducibility: f32, // 0.0 to 100.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfidenceCalibrationResult {
    pub original_confidence: ConfidenceLevel,
    pub adjusted_confidence: ConfidenceLevel,
    pub reliability_coefficient: f32,
    pub calibration_impact: String,
    pub reasoning: String,
    pub learning_impact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PriorityLevel {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityAssessment {
    pub priority_score: u8,
    pub priority_level: PriorityLevel,
    pub reasoning: Vec<String>,
    pub learning_impact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TargetPriorityLevel {
    DeepAnalysis,
    Standard,
    Deprioritized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStrategyDecision {
    pub target: String,
    pub priority: TargetPriorityLevel,
    pub adaptive_scan_depth: u8,
    pub reasoning: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFinding {
    pub id: String,
    pub canonical_slug: String,
    pub title: String,
    pub risk_family: FindingCategory,
    pub severity: SeverityLevel,
    pub confidence: ConfidenceLevel,
    pub merged_evidence_count: usize,
    pub contributing_modules: Vec<SourceModule>,
    pub affected_routes: Vec<String>,
    pub underlying_findings: Vec<SecurityAuditFinding>,
    pub verification_status: VerificationStatus,
    pub exploitability_score: Option<u32>,
    pub exploitability_level: Option<ExploitabilityLevel>,
    pub exploitability_reasoning: Option<String>,
    pub attack_surface_tags: Vec<String>,
    pub active_verification: Option<ActiveVerificationData>,
    pub confidence_calibration: Option<ConfidenceCalibrationResult>,
    pub priority_assessment: Option<PriorityAssessment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPath {
    pub id: String,
    pub attack_path_score: u32,
    pub narrative: String,
    pub involved_canonical_slugs: Vec<String>,
    pub shared_context: Vec<String>,
    pub overall_risk_level: ExploitabilityLevel,
    pub required_conditions: Vec<String>,
    pub active_verification: Option<ActiveVerificationData>,
    pub priority_assessment: Option<PriorityAssessment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedAuditReport {
    pub target: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub total_findings: usize,
    pub findings: Vec<SecurityAuditFinding>,
    pub accepted_findings: usize,
    pub rejected_findings: usize,
    pub normalization_log: Vec<String>,
    pub rule_results: Vec<RuleMatchResult>,
    pub correlations: Vec<CorrelationResult>,
    pub canonical_findings: Vec<CanonicalFinding>,
    pub attack_paths: Vec<AttackPath>,
    pub scoring_stats: Option<ScoringStats>,
    pub context_stats: Option<ContextStats>,
    pub audit_certainty: Option<CertaintyNote>,
    pub dynamic_rule_findings: Vec<crate::infrastructure::rule_engine::DynamicRuleFinding>,
}

// ── Phase 4: Risk Scoring & Prioritization ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskFactor {
    BaseSeverity,
    ConfidenceMultiplier,
    EvidenceQuality,
    CorrelationBoost,
    MultiModuleConfirmation,
    SensitiveEndpoint,
    AuthenticationExposure,
    DangerousMethod,
    WeakEvidencePenalty,
    LowConfidencePenalty,
    GenericMatchPenalty,
    NoisyCorrelationPenalty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskContribution {
    pub factor: RiskFactor,
    pub delta: i32,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub finding_score: u32,
    pub correlation_score: u32,
    pub total_score: u32,
    pub level: RiskLevel,
    pub contributions: Vec<RiskContribution>,
    pub priority_statement: String,
    pub escalation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringStats {
    pub total_scored: usize,
    pub boosted: usize,
    pub downgraded: usize,
    pub overall_risk_score: f64,
    pub top_risk_summary: Option<String>,
}

// ── Phase 2: Rule Evaluation Engine Data Models ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleOutcome {
    Matched,
    PartiallyMatched,
    NotMatched,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleConditionType {
    CategoryIs,
    SeverityMin,
    ConfidenceMin,
    SummaryContains,
    HasEvidence,
    ProtocolIs,
    MethodIs,
    SourceModuleIs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub condition_type: RuleConditionType,
    pub expected_value: String,
    pub is_mandatory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRule {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category_mapping: FindingCategory,
    pub conditions: Vec<RuleCondition>,
    pub default_severity: SeverityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConditionEvaluation {
    pub condition: RuleCondition,
    pub is_met: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMatchResult {
    pub rule_id: String,
    pub rule_title: String,
    pub outcome: RuleOutcome,
    pub finding_id: String,
    pub summary: String, // UI-friendly clear summary like "Missing CSP detected"
    pub evaluations: Vec<AuditConditionEvaluation>,
}

// ── Phase 3: Correlation Engine Data Models ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CorrelationType {
    DuplicateSignal,
    SupportingSignal,
    CompoundRisk,
    RepeatedSurface,
    ContextualLink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationReason {
    pub code: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationLink {
    pub finding_id: String,
    pub relationship_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationGroup {
    pub id: String,
    pub core_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    pub group_id: String,
    pub core_target: String,
    pub correlation_type: CorrelationType,
    pub confidence: ConfidenceLevel,
    pub summary: String,
    pub reason: CorrelationReason,
    pub linked_findings: Vec<CorrelationLink>,
    pub is_hygiene_gap: bool,
}

// ── Phase 5: Context-Aware Noise Reduction ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextSignal {
    AuthenticationSurface,
    SessionOrCookieRelevance,
    SensitiveEndpointExposed,
    DangerousMethodOnSensitivePath,
    MultiModuleConfirmation,
    ConcreteExploitEvidence,
    AdministrativePathExposed,
    HighConfidenceChain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NoiseIndicator {
    GenericInfraObservation,
    WeakCorrelationOverlap,
    DuplicateHeaderInCluster,
    SpeculativeImpactNoEvidence,
    LowConfidenceIsolated,
    OverlyBroadCategory,
    StaticAssetHeaderMissing,
    RedundantFinding,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionReason {
    DuplicateClusterNoise,
    ZeroExploitRelevance,
    InsufficientEvidence,
    GenericNonActionable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PriorityAdjustment {
    Elevated,
    Unchanged,
    Downgraded,
    Suppressed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareAssessment {
    pub adjustment: PriorityAdjustment,
    pub score_delta: i32,
    pub adjusted_score: u32,
    pub adjusted_level: RiskLevel,
    pub signals: Vec<ContextSignal>,
    pub noise_indicators: Vec<NoiseIndicator>,
    pub suppression_reason: Option<SuppressionReason>,
    pub context_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub elevated: usize,
    pub downgraded: usize,
    pub suppressed: usize,
    pub unchanged: usize,
}

// ── Burp Suite Bridge Data Models ──

/// A single HTTP request/response pair captured from Burp Suite proxy, scanner, or repeater.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpTrafficItem {
    pub url: String,
    pub method: String,
    pub request_headers: std::collections::HashMap<String, String>,
    pub request_body: Option<String>,
    pub response_status: u16,
    pub response_headers: std::collections::HashMap<String, String>,
    pub response_body: Option<String>,
    pub timestamp: i64,
    /// Which Burp tool captured this: "proxy", "scanner", "repeater", "intruder"
    pub tool_source: String,
}

/// Tracks the state of a Burp Bridge session — one per connected plugin instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpBridgeSession {
    pub session_id: String,
    pub target_url: String,
    pub burp_version: Option<String>,
    pub plugin_version: Option<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub imported_traffic_count: usize,
    pub exported_findings_count: usize,
    pub status: BurpSessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BurpSessionStatus {
    Connected,
    Syncing,
    Idle,
    Disconnected,
}

/// Handshake request from the Burp plugin to establish a bridge session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpHandshakeRequest {
    pub burp_version: Option<String>,
    pub plugin_version: Option<String>,
    pub target_url: String,
    /// Optional: link to an existing LIMMA scan session
    pub limma_session_id: Option<String>,
}

/// Handshake response confirming bridge session creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpHandshakeResponse {
    pub session_id: String,
    pub status: BurpSessionStatus,
    pub server_version: String,
    pub capabilities: Vec<String>,
}

/// Bulk traffic import request from Burp plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpImportTrafficRequest {
    pub session_id: String,
    pub items: Vec<BurpTrafficItem>,
}

/// Response after importing traffic items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpImportTrafficResponse {
    pub imported_count: usize,
    pub session_id: String,
    pub new_findings_triggered: usize,
    pub enrichment_notes: Vec<String>,
}

/// A finding formatted for Burp Suite native consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpNativeFinding {
    pub name: String,
    pub detail: String,
    pub severity: String,   // "High", "Medium", "Low", "Information"
    pub confidence: String, // "Certain", "Firm", "Tentative"
    pub url: String,
    pub path: String,
    pub host: String,
    pub port: i32,
    pub protocol: String,
    pub remediation: String,
    pub issue_type: u32,
    pub cwe_id: Option<u32>,
}

/// Response containing findings in Burp-native format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpFindingsResponse {
    pub session_id: String,
    pub target: String,
    pub findings: Vec<BurpNativeFinding>,
    pub total_count: usize,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// SSE events sent from backend to Burp plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BurpSseEvent {
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: i64 },
    #[serde(rename = "finding_detected")]
    FindingDetected(BurpNativeFinding),
    #[serde(rename = "sync_status")]
    SyncStatus {
        status: BurpSessionStatus,
        message: String,
    },
}
