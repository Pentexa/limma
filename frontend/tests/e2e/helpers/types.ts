/**
 * Types used throughout the LIMMA Frontend Verification Test Suite.
 */

// ── Normalized Finding for comparison ──
export interface NormalizedFinding {
  /** Primary key: finding ID from backend or generated deterministic key */
  finding_id: string;
  /** Deterministic key: category + severity + summary hash for fallback matching */
  deterministic_key: string;
  /** Finding title or summary */
  title: string;
  severity: string;
  confidence: string;
  status: string;
  category: string;
  source_module: string;
  evidence_count: number;
  /** Which data source produced this normalized finding */
  source: 'backend' | 'frontend';
}

// ── Comparison results ──
export interface MatchedFinding {
  finding_id: string;
  backend: NormalizedFinding;
  frontend: NormalizedFinding;
  mismatches: FieldMismatch[];
}

export interface FieldMismatch {
  field: string;
  backend_value: string;
  frontend_value: string;
}

export interface ComparisonResult {
  total_backend_findings: number;
  total_frontend_findings: number;
  matched_findings: MatchedFinding[];
  missing_in_frontend: NormalizedFinding[];   // false negatives
  extra_in_frontend: NormalizedFinding[];     // false positives
  duplicated_in_frontend: DuplicateGroup[];
  field_mismatches: MatchedFinding[];         // matched but with field diffs
  accuracy: number;
  false_positive_rate: number;
  false_negative_rate: number;
  timestamp: string;
}

export interface DuplicateGroup {
  deterministic_key: string;
  findings: NormalizedFinding[];
}

// ── SSE Verification ──
export interface SSEEvent {
  timestamp: string;
  type: string;
  data: unknown;
  sequence: number;
}

export interface SSEVerificationResult {
  total_events_received: number;
  unique_event_types: string[];
  dropped_events: number;
  duplicate_renders: SSEDuplicateRender[];
  final_state_matches_backend: boolean;
  event_sequence_valid: boolean;
  errors: string[];
}

export interface SSEDuplicateRender {
  event_type: string;
  data_hash: string;
  occurrences: number;
}

// ── Full Verification Report ──
export interface VerificationReport {
  suite_name: string;
  target_url: string;
  scan_duration_ms: number;
  comparison: ComparisonResult;
  sse_verification: SSEVerificationResult | null;
  timestamp: string;
  pass: boolean;
  summary: string;
}
