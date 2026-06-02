/**
 * Backend API response types.
 * Field names use snake_case to match the Rust backend JSON responses exactly.
 * Use mapper functions to convert to camelCase frontend entities.
 */

// ── Active Vulnerability Scanner ──

export type ActiveVulnType =
  | "reflected_xss" | "stored_xss" | "dom_xss"
  | "sql_injection_error" | "sql_injection_union" | "sql_injection_blind_time" | "sql_injection_blind_boolean"
  | "command_injection" | "command_injection_blind"
  | "local_file_inclusion" | "remote_file_inclusion" | "path_traversal"
  | "server_side_request_forgery"
  | "xml_external_entity"
  | "open_redirect"
  | "jwt_none_algorithm" | "jwt_weak_secret"
  | "no_sql_injection" | "server_side_template_injection"
  | "graphql_introspection_enabled" | "graphql_abuse";

export interface ApiActiveScanRequest {
  target_url: string;
  vuln_types: ActiveVulnType[];
  max_duration_seconds?: number;
  rate_limit_rps?: number;
  follow_redirects?: boolean;
  profile_id?: string;
  custom_parameters?: string[];
}

export interface ApiActiveScanFinding {
  id: string;
  scan_id: string;
  timestamp: string;
  vuln_type: ActiveVulnType;
  target_url: string;
  affected_parameter: string;
  http_method: string;
  payload_used: string;
  evidence: {
    request_raw: string;
    response_raw: string;
    response_time_ms: number;
    matched_indicator: string;
    additional_notes: string[];
  };
  severity: string;
  confidence: string;
  exploitability: string;
  poc_generated: boolean;
  poc_id?: string;
  verified: boolean;
  false_positive: boolean;
}

export interface ApiActiveScanSummary {
  critical_count: number;
  high_count: number;
  medium_count: number;
  low_count: number;
  info_count: number;
  vuln_type_breakdown: Record<string, number>;
  waf_detected: boolean;
  waf_blocked_requests: number;
}

export interface ApiActiveScanResult {
  scan_id: string;
  target_url: string;
  status: "pending" | "running" | "completed" | "failed" | "cancelled";
  start_time: string;
  end_time?: string;
  total_requests: number;
  findings: ApiActiveScanFinding[];
  errors: string[];
  summary: ApiActiveScanSummary;
}

// ── Passive Scan (Website Analysis) ──

export interface ApiWebScanResult {
  original_target_url: string;
  final_url: string;
  scan_start_time: string;
  scan_end_time: string;
  total_duration_ms: number;
  final_status_code: number;
  latency_ms: number;
  redirect_count: number;
  redirect_chain: { url: string; status_code: number }[];
  headers: Record<string, string>;
  content_type?: string;
  content_length?: number;
  server?: string;
  cache_control?: string;
  detected_technologies: ApiDetectedTechnology[];
  security_headers: ApiSecurityHeaderResult[];
  risk_insights: ApiRiskInsight[];
  security_score: number;
  pages: ApiScannedPage[];
  timeline: ApiScanEvent[];
  summary?: ApiScanSummary;
  correlation?: ApiCorrelationReport;
  scan_certainty?: ApiCertaintyNote;
}

export interface ApiDetectedTechnology {
  name: string;
  category: string;
  confidence_score: number;
  evidences: { source: string; snippet: string }[];
}

export interface ApiSecurityHeaderResult {
  name: string;
  status: "present" | "missing" | "weak" | "misconfigured";
  value?: string;
  explanation: string;
}

export interface ApiRiskInsight {
  title: string;
  severity: string;
  explanation: string;
  evidence: string;
}

export interface ApiScannedPage {
  url: string;
  status_code: number;
  latency_ms: number;
  headers: Record<string, string>;
  content_type?: string;
  detected_technologies: ApiDetectedTechnology[];
  security_headers: ApiSecurityHeaderResult[];
  risk_insights: ApiRiskInsight[];
}

export interface ApiScanEvent {
  timestamp: string;
  event_type: string;
  level: string;
  message: string;
  payload?: unknown;
}

export interface ApiScanSummary {
  total_pages: number;
  average_latency_ms: number;
  common_technologies: string[];
}

export interface ApiCorrelationReport {
  overall_risk_score: number;
  correlated_risks: { title: string; severity: string; explanation: string; evidences: string[] }[];
}

export interface ApiCertaintyNote {
  level: string;
  reason: string;
}

// ── Server Investigation ──

export interface ApiServerInfo {
  original_target: string;
  resolved_url: string;
  status_code: number;
  latency_ms: number;
  raw_headers: Record<string, string[]>;
  categorized_headers: Record<string, Record<string, string[]>>;
  infrastructure_signals: { signal_type: string; value: string; evidence: string }[];
  fingerprints: { name: string; category: string; confidence_score: number; evidences: string[]; explanation: string; certainty?: string }[];
  delivery_insights: { name: string; category: string; confidence_score: number; evidence: string; explanation: string; certainty?: string }[];
  security_insights: { name: string; category: string; status: string; confidence_score: number; evidence: string; explanation: string; certainty?: string }[];
  routes_checked: string[];
  consistency_insights: { name: string; severity: string; category: string; evidences: string[]; explanation: string }[];
  activity_log: string[];
  investigation_certainty?: ApiCertaintyNote;
}

// ── API Discovery ──

export interface ApiDiscoveryResult {
  base_url: string;
  detected_endpoints: {
    path: string;
    method_prediction: string;
    parameters: { name: string; param_type: string; data_type: string }[];
    auth_probability: number;
    auth_likelihood: string;
    confidence_score: number;
    evidences: { source_type: string; snippet: string; reason: string; line_number?: number }[];
    runtime_verification?: {
      is_valid: boolean;
      best_method: string;
      status_code: number;
      response_time_ms: number;
      has_body: boolean;
      content_type?: string;
      server?: string;
    };
    certainty?: string;
  }[];
  suspected_api_technologies: string[];
  metrics?: {
    total_endpoints: number;
    valid_endpoints: number;
    false_positives: number;
    precision: number;
    source_distribution: Record<string, number>;
    confidence_accuracy_correlation: number;
  };
  discovery_certainty?: ApiCertaintyNote;
}

// ── Service Collector ──

export interface ApiCollectorSnapshot {
  target_input: { original_input: string; normalized_url: string; host: string; scheme?: string; default_port: number };
  resolved_target: { ip_addresses: string[]; primary_ip?: string; hostname?: string };
  timestamp: string;
  port_results: {
    port: number;
    state: string;
    latency_ms?: number;
    service_candidates: {
      service_name: string;
      confidence_breakdown: {
        port_evidence: number;
        protocol_validation: number;
        fingerprint_strength: number;
        header_reliability: number;
        redirect_penalty: number;
        cdn_penalty: number;
        response_quality: number;
        final_score: number;
      };
      decision: string;
      probe_method: string;
      reasoning: string;
    }[];
    all_evidence: { kind: string; strength: string; source: string; raw_signal: string; interpretation: string; is_negative: boolean }[];
    fallback_used: boolean;
    retry_count: number;
    probe_duration_ms: number;
  }[];
  activity_timeline: { timestamp: string; event_type: string; message: string; severity?: string }[];
  errors: string[];
  overall_status: string;
}

// ── Security Audit ──

export interface ApiSecurityReport {
  url: string;
  security_score: number;
  missing_headers: string[];
  robot_rules_disallowed: string[];
  recommendations: string[];
}

// ── Form Mapping ──

export interface ApiFormMapping {
  url: string;
  detected_forms: { action: string; method: string; fields: string[] }[];
  login_pages_found: string[];
}

// ── Master Report ──

export type ApiExploitabilityLevel = "actionable" | "theoretical" | "inert";

export interface ApiVerificationData {
  status: string;
  reasoning: string;
}

export interface ApiPriorityAssessment {
  priority_score: number;
  priority_level: string;
  reasoning: string[];
}

export interface ApiAttackPath {
  id: string;
  attack_path_score: number;
  narrative: string;
  involved_canonical_slugs: string[];
  shared_context: string[];
  overall_risk_level: ApiExploitabilityLevel;
  required_conditions: string[];
  active_verification?: ApiVerificationData;
  priority_assessment?: ApiPriorityAssessment;
}

export interface ApiCanonicalFinding {
  id: string;
  canonical_slug: string;
  title: string;
  risk_family: string;
  severity: string;
  confidence: string;
  affected_routes: string[];
}

export interface ApiNormalizedAuditReport {
  target: string;
  timestamp: string;
  total_findings: number;
  attack_paths: ApiAttackPath[];
  canonical_findings: ApiCanonicalFinding[];
}

export interface ApiMasterReport {
  url: string;
  analysis?: ApiWebScanResult;
  server_info?: ApiServerInfo;
  api_discovery?: ApiDiscoveryResult;
  service_collector?: ApiCollectorSnapshot;
  security_audit?: ApiSecurityReport;
  normalized_audit?: ApiNormalizedAuditReport;
  form_mapping?: ApiFormMapping;
  overall_health_score: number;
  module_errors?: string[];
}

// ── Blind Detection & Exploitation ──

export interface ApiBlindScanRequest {
  target_url: string;
  detection_types: string[];
  scan_id?: string;
  target_id?: string;
  max_duration_seconds?: number;
}

export interface ApiBlindFinding {
  id: string;
  scan_id: string;
  target_id: string;
  vulnerability_type: string;
  detection_method: string;
  confidence: number;
  evidence: {
    evidence_type: string;
    details: Record<string, unknown>;
  };
  payload_used: string;
  created_at: string;
  verified: boolean;
}

export interface ApiPoc {
  id: string;
  finding_id: string;
  poc_type: string;
  code: string;
  language: string;
  safety_level: string;
  verification_status: string;
  created_at: string;
}

export interface ApiExploitResult {
  id: string;
  poc_id: string;
  executed_at: string;
  success: boolean;
  output?: string;
  error?: string;
  execution_time_ms: number;
  sandbox_logs?: string;
}

// ── History & Trends ──

export interface ApiTrendPoint {
  scan_id: string;
  timestamp_sec: number;
  score: number;
  total_endpoints: number;
  total_findings: number;
}

export interface ApiDeltaResult {
  base_scan_id: string;
  compare_scan_id: string;
  new_endpoints: { url: string; method: string }[];
  resolved_findings: { name: string; severity: string; url: string }[];
  new_findings: { name: string; severity: string; url: string }[];
}

// ── Settings ──

export type WordlistSize = "small" | "medium" | "large" | "massive" | string;
export type DnsResolution = "system" | "custom" | "cloudflare" | string;
export type ExploitMode = "safe_verification" | "authorized_active" | string;
export type FuzzingIntensity = "low" | "medium" | "high" | "aggressive" | string;

export interface ApiSettingsProfile {
  id: string;
  name: string;
  description: string;
  is_custom: boolean;
  global: { 
    timeout_ms: number; 
    rate_limit_req_per_sec: number; 
    use_proxy: boolean; 
    proxy_url: string | null; 
    target_scope: string; 
    auth_profile_id: string | null; 
  };
  scanner: { 
    user_agent: string; 
    wordlist_size: WordlistSize; 
    follow_redirects: boolean; 
    max_depth: number; 
  };
  investigator: { 
    dns_resolution: DnsResolution; 
    fingerprint_level: number; 
    concurrent_hosts: number; 
  };
  api_discovery: { 
    wordlist_size: WordlistSize; 
    custom_headers: boolean; 
    schema_parsing: boolean; 
  };
  services: { 
    port_scan_range: string; 
    banner_grabbing: boolean; 
    timeout_per_port_ms: number; 
  };
  forms: { 
    fuzzing_intensity: FuzzingIntensity; 
    extract_hidden_inputs: boolean; 
    avoid_waf: boolean; 
  };
  audit: { 
    risk_coefficient: number; 
    ignore_informational: boolean; 
    auto_map_cwe: boolean; 
  };
  rules: { 
    strict_mode: boolean; 
    auto_sync_rules: boolean; 
    custom_rule_path: string; 
  };
  exploit: { 
    mode: ExploitMode; 
    sandbox_validation: boolean; 
    manual_approval_required: boolean; 
  };
  proxy: { 
    intercept_requests: boolean; 
    history_limit: number; 
    auto_drop_malicious: boolean; 
  };
  sessions: { 
    auto_delete_days: number; 
    archive_artifacts: boolean; 
  };
  subdomain: {
    wordlist_size: WordlistSize;
    enable_crtsh: boolean;
    enable_dns_bruteforce: boolean;
    http_probe_enabled: boolean;
    max_concurrent_dns: number;
  };
}

// ── Rule Engine ──

export interface ApiRuleEngineStatus {
  total_rules: number;
  disabled_packs: string[];
  disabled_rules: string[];
  active_rules: {
    id: string;
    name: string;
    category: string;
    pack: string;
    source: string;
    version: string;
    default_severity: string;
    default_confidence: string;
    is_active: boolean;
  }[];
  feedback_stats: Record<string, {
    total_feedback: number;
    confirmed: number;
    false_positives: number;
    ignored: number;
    reputation_score: number;
  }>;
}

// ── Feedback ──

export interface ApiFeedbackStatsResponse {
  total_feedback_entries: number;
  rule_stats: Record<string, {
    rule_name: string;
    total_feedback: number;
    confirmed: number;
    false_positives: number;
    ignored: number;
    reputation_score: number;
  }>;
  recent_feedback: {
    rule_id: string;
    action: string;
    target_url: string;
    timestamp: string;
  }[];
}

// ── Burp Bridge ──

export interface ApiBurpSession {
  session_id: string;
  plugin_version: string;
  created_at: string;
}

export interface ApiBurpFinding {
  id: string;
  session_id: string;
  name: string;
  severity: string;
  url: string;
  detail: string;
}

// ── Subdomain Discovery ──

export type ApiSubdomainSource = "crt_sh" | "dns_wordlist" | "crawler_links";

export type ApiSubdomainStatus =
  | "validated"
  | "unresolved"
  | "wildcard_filtered"
  | "http_alive"
  | "http_dead";

export interface ApiDnsRecord {
  record_type: string;
  values: string[];
}

export interface ApiRedirectChainEntry {
  url: string;
  status_code: number;
}

export interface ApiHttpProbeResult {
  url: string;
  status_code: number;
  title?: string;
  server?: string;
  content_type?: string;
  technologies: string[];
  tls_issuer?: string;
  tls_subject?: string;
  redirect_chain: ApiRedirectChainEntry[];
  response_time_ms: number;
}

export interface ApiSubdomainAsset {
  asset: string;
  asset_type: string;
  sources: ApiSubdomainSource[];
  resolved_ips: string[];
  dns_records: ApiDnsRecord[];
  http_probe?: ApiHttpProbeResult;
  http_status?: number;
  technologies: string[];
  confidence: number;
  status: ApiSubdomainStatus;
  risk_tags: string[];
  last_seen: string;
  certainty?: string;
}

export interface ApiWildcardDnsInfo {
  is_wildcard: boolean;
  test_subdomain: string;
  resolved_ips: string[];
}

export interface ApiSubdomainDiscoveryMetrics {
  total_candidates: number;
  validated_count: number;
  wildcard_filtered_count: number;
  duplicate_removed_count: number;
  http_alive_count: number;
  passive_source_count: number;
  active_source_count: number;
  precision: number;
  scan_duration_ms: number;
}

export interface ApiSubdomainDiscoveryResult {
  domain: string;
  wildcard_dns: ApiWildcardDnsInfo;
  assets: ApiSubdomainAsset[];
  metrics: ApiSubdomainDiscoveryMetrics;
  discovery_certainty?: ApiCertaintyNote;
  scan_timestamp: string;
}

export interface ApiSubdomainDiscoveryRequest {
  domain: string;
  profile_id?: string;
}

// ── Port Verification ──

export interface ApiVerifyPortResponse {
  is_active: boolean;
  latency_ms?: number;
  banner?: string;
}
