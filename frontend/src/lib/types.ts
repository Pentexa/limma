export interface WebScanResult {
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
  detected_technologies: DetectedTechnology[];
  security_headers: SecurityHeaderResult[];
  risk_insights: RiskInsight[];
  security_score: number;
  pages: ScannedPage[];
  timeline: ScanEvent[];
  summary?: ScanSummary;
  correlation?: CorrelationReport;
  scan_certainty?: CertaintyNote;
}

export interface DetectedTechnology {
  name: string;
  category: string;
  confidence_score: number;
  evidences: { source: string; snippet: string }[];
}

export interface SecurityHeaderResult {
  name: string;
  status: 'present' | 'missing' | 'weak' | 'misconfigured';
  value?: string;
  explanation: string;
}

export interface RiskInsight {
  title: string;
  severity: string;
  explanation: string;
  evidence: string;
}

export interface ScannedPage {
  url: string;
  status_code: number;
  latency_ms: number;
  headers: Record<string, string>;
  content_type?: string;
  detected_technologies: DetectedTechnology[];
  security_headers: SecurityHeaderResult[];
  risk_insights: RiskInsight[];
}

export interface ScanEvent {
  timestamp: string;
  event_type: string;
  level: string;
  message: string;
  payload?: unknown;
}

export interface ScanSummary {
  total_pages: number;
  average_latency_ms: number;
  common_technologies: string[];
}

export interface CorrelationReport {
  overall_risk_score: number;
  correlated_risks: CorrelatedRisk[];
}

export interface CorrelatedRisk {
  title: string;
  severity: string;
  explanation: string;
  evidences: string[];
}

export interface CertaintyNote {
  level: string;
  reason: string;
}

export interface ServerInfo {
  original_target: string;
  resolved_url: string;
  status_code: number;
  latency_ms: number;
  raw_headers: Record<string, string[]>;
  categorized_headers: Record<string, Record<string, string[]>>;
  infrastructure_signals: InfrastructureSignal[];
  fingerprints: InvestigatorFingerprint[];
  delivery_insights: DeliveryInsight[];
  security_insights: SecurityPostureInsight[];
  routes_checked: string[];
  consistency_insights: ConsistencyInsight[];
  activity_log: string[];
  investigation_certainty?: CertaintyNote;
}

export interface InfrastructureSignal {
  signal_type: string;
  value: string;
  evidence: string;
}

export interface InvestigatorFingerprint {
  name: string;
  category: string;
  confidence_score: number;
  evidences: string[];
  explanation: string;
  certainty?: string;
}

export interface DeliveryInsight {
  name: string;
  category: string;
  confidence_score: number;
  evidence: string;
  explanation: string;
  certainty?: string;
}

export interface SecurityPostureInsight {
  name: string;
  category: string;
  status: string;
  confidence_score: number;
  evidence: string;
  explanation: string;
  certainty?: string;
}

export interface ConsistencyInsight {
  name: string;
  severity: string;
  category: string;
  evidences: string[];
  explanation: string;
}

export interface ApiDiscoveryResult {
  base_url: string;
  detected_endpoints: EndpointDetail[];
  suspected_api_technologies: string[];
  metrics?: DiscoveryMetrics;
  discovery_certainty?: CertaintyNote;
}

export interface EndpointDetail {
  path: string;
  method_prediction: string;
  parameters: { name: string; param_type: string; data_type: string }[];
  auth_probability: number;
  auth_likelihood: string;
  confidence_score: number;
  evidences: { source_type: string; snippet: string; reason: string; line_number?: number }[];
  runtime_verification?: RuntimeVerification;
  certainty?: string;
}

export interface RuntimeVerification {
  is_valid: boolean;
  best_method: string;
  status_code: number;
  response_time_ms: number;
  has_body: boolean;
  content_type?: string;
  server?: string;
}

export interface DiscoveryMetrics {
  total_endpoints: number;
  valid_endpoints: number;
  false_positives: number;
  precision: number;
  source_distribution: Record<string, number>;
  confidence_accuracy_correlation: number;
}

export interface ActivityEvent {
  timestamp: string;
  event_type: string;
  message: string;
  severity?: string;
}

export interface CollectorSnapshot {
  target_input: { original_input: string; normalized_url: string; host: string; scheme?: string; default_port: number };
  resolved_target: { ip_addresses: string[]; primary_ip?: string; hostname?: string };
  timestamp: string;
  port_results: PortProbeResult[];
  activity_timeline: ActivityEvent[];
  errors: string[];
  overall_status: string;
  diff?: SnapshotDiff;
}

export interface PortProbeResult {
  port: number;
  state: string;
  latency_ms?: number;
  service_candidates: ServiceCandidate[];
  all_evidence: EvidenceItem[];
  fingerprint_evaluations: FingerprintMatch[];
  fallback_used: boolean;
  retry_count: number;
  probe_duration_ms: number;
}

export interface ServiceCandidate {
  service_name: string;
  confidence_breakdown: ConfidenceBreakdown;
  decision: string;
  probe_method: string;
  supporting_evidence: EvidenceItem[];
  conflicting_evidence: EvidenceItem[];
  reasoning: string;
  tls_summary?: TlsSummary;
  http_summary?: HttpSummary;
  ambiguity?: { description: string; conflicting_evidence: string[] };
  fingerprint_match?: FingerprintMatch;
  verification_trail: { step: string; detail: string }[];
}

export interface ConfidenceBreakdown {
  port_evidence: number;
  protocol_validation: number;
  fingerprint_strength: number;
  header_reliability: number;
  redirect_penalty: number;
  cdn_penalty: number;
  response_quality: number;
  final_score: number;
}

export interface EvidenceItem {
  kind: string;
  strength: string;
  source: string;
  raw_signal: string;
  interpretation: string;
  suggests_service?: string;
  is_negative: boolean;
}

export interface FingerprintMatch {
  fingerprint_id: string;
  service_name: string;
  tier: string;
  strength: string;
  confidence: number;
  confidence_level: string;
  coverage: string;
  matched_rules: RuleEvaluation[];
  missing_rules: RuleEvaluation[];
  conflicting_rules: RuleEvaluation[];
  explanation_items: { category: string; description: string; impact: number }[];
  penalties: { reason: string; amount: number }[];
  reasoning: string;
}

export interface RuleEvaluation {
  category: string;
  expected: string;
  actual?: string;
  matched: boolean;
  weight: number;
  rule_weight: string;
  contribution: number;
  skipped_contextual: boolean;
}

export interface TlsSummary {
  has_tls: boolean;
  protocol_version?: string;
  cipher_suite?: string;
  subject?: string;
  issuer?: string;
  alpn?: string;
  sni_used: boolean;
}

export interface HttpSummary {
  status_code?: number;
  server_header?: string;
  content_type?: string;
  redirect_target?: string;
  headers: Record<string, string>;
  response_length?: number;
}

export interface SnapshotDiff {
  previous_timestamp: string;
  current_timestamp: string;
  changes: { change_type: string; resource: string; before?: string; after?: string; description: string }[];
  summaries: string[];
}

export interface SecurityReport {
  url: string;
  security_score: number;
  missing_headers: string[];
  robot_rules_disallowed: string[];
  recommendations: string[];
}

export interface FormMapping {
  url: string;
  detected_forms: { action: string; method: string; fields: string[] }[];
  login_pages_found: string[];
}

export interface MasterReport {
  url: string;
  analysis?: WebScanResult;
  server_info?: ServerInfo;
  api_discovery?: ApiDiscoveryResult;
  service_collector?: CollectorSnapshot;
  security_audit?: SecurityReport;
  normalized_audit?: NormalizedAuditReport;
  form_mapping?: FormMapping;
  scan_strategy?: ScanStrategyDecision[];
  overall_health_score: number;
  module_errors?: string[];
}

export interface NormalizedAuditReport {
  target: string;
  timestamp: string;
  total_findings: number;
  findings: SecurityAuditFinding[];
  accepted_findings: number;
  rejected_findings: number;
  normalization_log: string[];
  rule_results: RuleMatchResult[];
  correlations: CorrelationResult[];
  canonical_findings: CanonicalFinding[];
  attack_paths: AttackPath[];
  scoring_stats?: ScoringStats;
  context_stats?: ContextStats;
  audit_certainty?: CertaintyNote;
  dynamic_rule_findings: DynamicRuleFinding[];
}

export interface SecurityAuditFinding {
  id: string;
  timestamp: string;
  target_identifier: string;
  affected_path_or_endpoint?: string;
  protocol?: string;
  method?: string;
  category: string;
  severity: string;
  confidence: string;
  status: string;
  summary: string;
  technical_details: string;
  source_module: string;
  evidence: { description: string; validation_context?: string; raw_data: string }[];
  correlation_group_id?: string;
  correlation_count: number;
  correlation_type?: string;
  correlation_confidence?: string;
  correlation_summary?: string;
  correlation_is_hygiene_gap: boolean;
  related_findings: { id: string; category: string; severity: string; short_summary: string }[];
  risk_score?: RiskScore;
  exploitability?: string;
  context_summary?: string;
  evidence_weight?: string;
  context_assessment?: ContextAwareAssessment;
  certainty?: string;
}

export interface RiskScore {
  finding_score: number;
  correlation_score: number;
  total_score: number;
  level: string;
  contributions: { factor: string; delta: number; explanation: string }[];
  priority_statement: string;
  escalation_reason?: string;
}

export interface ContextAwareAssessment {
  adjustment: string;
  score_delta: number;
  adjusted_score: number;
  adjusted_level: string;
  signals: string[];
  noise_indicators: string[];
  suppression_reason?: string;
  context_summary: string;
}

export interface RuleMatchResult {
  rule_id: string;
  rule_title: string;
  outcome: string;
  finding_id: string;
  summary: string;
  evaluations: { condition: unknown; is_met: boolean; detail: string }[];
}

export interface CorrelationResult {
  group_id: string;
  core_target: string;
  correlation_type: string;
  confidence: string;
  summary: string;
  reason: { code: string; explanation: string };
  linked_findings: { finding_id: string; relationship_note: string }[];
  is_hygiene_gap: boolean;
}

export interface CanonicalFinding {
  id: string;
  canonical_slug: string;
  title: string;
  risk_family: string;
  severity: string;
  confidence: string;
  merged_evidence_count: number;
  contributing_modules: string[];
  affected_routes: string[];
  underlying_findings: SecurityAuditFinding[];
  verification_status: string;
  exploitability_score?: number;
  exploitability_level?: string;
  exploitability_reasoning?: string;
  attack_surface_tags: string[];
  active_verification?: { status: string; reasoning: string; reproducibility_score: number; traces: unknown[] };
  confidence_calibration?: { original_confidence: string; adjusted_confidence: string; reliability_coefficient: number; calibration_impact: string; reasoning: string; learning_impact?: string };
  priority_assessment?: { priority_score: number; priority_level: string; reasoning: string[]; learning_impact?: string };
}

export interface AttackPath {
  id: string;
  attack_path_score: number;
  narrative: string;
  involved_canonical_slugs: string[];
  shared_context: string[];
  overall_risk_level: string;
  required_conditions: string[];
  active_verification?: unknown;
  priority_assessment?: { priority_score: number; priority_level: string; reasoning: string[] };
}

export interface ScoringStats {
  total_scored: number;
  boosted: number;
  downgraded: number;
  overall_risk_score: number;
  top_risk_summary?: string;
}

export interface ContextStats {
  elevated: number;
  downgraded: number;
  suppressed: number;
  unchanged: number;
}

export interface CertaintyNote {
  level: string;
  reason: string;
}

export interface ScanStrategyDecision {
  target: string;
  priority: string;
  adaptive_scan_depth: number;
  reasoning: string[];
}

export interface DynamicRuleFinding {
  rule_id: string;
  rule_name: string;
  category: string;
  severity: string;
  confidence: string;
  matched_target: string;
  evidence_summary: string;
  reputation_score?: number;
}

export interface RuleEngineStatus {
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

export interface FeedbackStatsResponse {
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

export interface VerifyPortResponse {
  is_active: boolean;
  latency_ms?: number;
  banner?: string;
}

// ── Delta Engine Types ──

export interface TrendPoint {
  scan_id: string;
  timestamp_sec: number;
  score: number;
  total_endpoints: number;
  total_findings: number;
}

export interface DeltaEndpoint {
  url: string;
  method: string;
}

export interface DeltaFinding {
  name: string;
  severity: string;
  url: string;
}

export interface DeltaResult {
  base_scan_id: string;
  compare_scan_id: string;
  new_endpoints: DeltaEndpoint[];
  resolved_findings: DeltaFinding[];
  new_findings: DeltaFinding[];
}

