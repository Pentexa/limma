/**
 * Finding Matcher — normalizes findings from both backend and frontend
 * into a common schema, then performs deterministic matching.
 */
import { createHash } from 'crypto';
import type {
  NormalizedFinding,
  MatchedFinding,
  FieldMismatch,
  ComparisonResult,
  DuplicateGroup,
} from './types';

// ── Deterministic key generator ──
export function generateDeterministicKey(
  category: string,
  severity: string,
  title: string,
): string {
  const raw = `${category}::${severity}::${title}`.toLowerCase().trim();
  return createHash('sha256').update(raw).digest('hex').substring(0, 16);
}

// ── Normalize backend SecurityAuditFinding ──
export function normalizeBackendFinding(finding: Record<string, unknown>): NormalizedFinding {
  const id = String(finding.id || '');
  const category = String(finding.category || '');
  const severity = String(finding.severity || '');
  const title = String(finding.summary || finding.title || '');
  const confidence = String(finding.confidence || '');
  const status = String(finding.status || '');
  const source_module = String(finding.source_module || '');
  const evidence = Array.isArray(finding.evidence) ? finding.evidence : [];

  const normalizedCategory = category.toLowerCase().replace(/_/g, ' ');
  return {
    finding_id: id,
    deterministic_key: generateDeterministicKey(normalizedCategory, severity, title),
    title,
    severity: severity.toLowerCase(),
    confidence: confidence.toLowerCase(),
    status: status.toLowerCase(),
    category: normalizedCategory,
    source_module: source_module.toLowerCase(),
    evidence_count: evidence.length,
    source: 'backend',
  };
}

// ── Normalize backend CanonicalFinding ──
export function normalizeBackendCanonicalFinding(finding: Record<string, unknown>): NormalizedFinding {
  const id = String(finding.id || '');
  const riskFamily = String(finding.risk_family || '');
  const severity = String(finding.severity || '');
  const title = String(finding.title || '');
  const confidence = String(finding.confidence || '');
  const verificationStatus = String(finding.verification_status || '');
  const contributingModules = Array.isArray(finding.contributing_modules) ? finding.contributing_modules : [];
  const normalizedCategory = riskFamily.toLowerCase().replace(/_/g, ' ');

  return {
    finding_id: id,
    deterministic_key: generateDeterministicKey(normalizedCategory, severity, title),
    title,
    severity: severity.toLowerCase(),
    confidence: confidence.toLowerCase(),
    status: verificationStatus.toLowerCase(),
    category: normalizedCategory,
    source_module: contributingModules.join(',').toLowerCase(),
    evidence_count: Number(finding.merged_evidence_count || 0),
    source: 'backend',
  };
}

// ── Normalize backend DynamicRuleFinding ──
export function normalizeBackendDynamicFinding(finding: Record<string, unknown>): NormalizedFinding {
  const ruleId = String(finding.rule_id || '');
  const category = String(finding.category || '');
  const severity = String(finding.severity || '');
  const title = String(finding.rule_name || '');
  const confidence = String(finding.confidence || '');
  const normalizedCategory = category.toLowerCase().replace(/_/g, ' ');

  return {
    finding_id: ruleId,
    deterministic_key: generateDeterministicKey(normalizedCategory, severity, title),
    title,
    severity: severity.toLowerCase(),
    confidence: confidence.toLowerCase(),
    status: 'active',
    category: normalizedCategory,
    source_module: 'dynamic_rule_engine',
    evidence_count: finding.evidence_summary ? 1 : 0,
    source: 'backend',
  };
}

// ── Normalize a frontend-extracted finding ──
export function normalizeFrontendFinding(finding: Record<string, string>): NormalizedFinding {
  const id = String(finding.id || finding.finding_id || '');
  const category = String(finding.category || finding.risk_family || '');
  const severity = String(finding.severity || '');
  const title = String(finding.title || finding.summary || '');
  const confidence = String(finding.confidence || '');
  const status = String(finding.status || finding.verification_status || '');
  const source_module = String(finding.source_module || finding.modules || '');
  const evidence_count = parseInt(String(finding.evidence_count || finding.evidence || '0'), 10) || 0;
  const normalizedCategory = category.toLowerCase().replace(/_/g, ' ');

  return {
    finding_id: id,
    deterministic_key: generateDeterministicKey(normalizedCategory, severity, title),
    title,
    severity: severity.toLowerCase(),
    confidence: confidence.toLowerCase(),
    status: status.toLowerCase(),
    category: normalizedCategory,
    source_module: source_module.toLowerCase(),
    evidence_count,
    source: 'frontend',
  };
}

// ── Match findings from backend and frontend ──
export function matchFindings(
  backendFindings: NormalizedFinding[],
  frontendFindings: NormalizedFinding[],
): ComparisonResult {
  const matched: MatchedFinding[] = [];
  const matchedBackendIds = new Set<string>();
  const matchedFrontendIds = new Set<string>();

  // Phase 1: Match by finding_id (exact match)
  for (const bf of backendFindings) {
    if (!bf.finding_id) continue;
    const ff = frontendFindings.find(
      (f) => f.finding_id === bf.finding_id && !matchedFrontendIds.has(f.finding_id),
    );
    if (ff) {
      const mismatches = compareFields(bf, ff);
      matched.push({ finding_id: bf.finding_id, backend: bf, frontend: ff, mismatches });
      matchedBackendIds.add(bf.finding_id);
      matchedFrontendIds.add(ff.finding_id);
    }
  }

  // Phase 2: Match remaining by deterministic key
  for (const bf of backendFindings) {
    if (matchedBackendIds.has(bf.finding_id)) continue;
    const ff = frontendFindings.find(
      (f) =>
        f.deterministic_key === bf.deterministic_key &&
        !matchedFrontendIds.has(f.finding_id) &&
        !matchedFrontendIds.has(f.deterministic_key),
    );
    if (ff) {
      const mismatches = compareFields(bf, ff);
      const matchId = bf.finding_id || bf.deterministic_key;
      matched.push({ finding_id: matchId, backend: bf, frontend: ff, mismatches });
      matchedBackendIds.add(bf.finding_id);
      matchedFrontendIds.add(ff.finding_id || ff.deterministic_key);
    }
  }

  // Missing in frontend (false negatives)
  const missingInFrontend = backendFindings.filter(
    (bf) => !matchedBackendIds.has(bf.finding_id),
  );

  // Extra in frontend (false positives)
  const extraInFrontend = frontendFindings.filter(
    (ff) => !matchedFrontendIds.has(ff.finding_id) && !matchedFrontendIds.has(ff.deterministic_key),
  );

  // Detect duplicates in frontend
  const duplicated = detectDuplicates(frontendFindings);

  // Field mismatches
  const fieldMismatches = matched.filter((m) => m.mismatches.length > 0);

  // Metrics
  const totalBackend = backendFindings.length;
  const totalFrontend = frontendFindings.length;
  const matchedCount = matched.length;
  const accuracy = totalBackend > 0 ? (matchedCount / totalBackend) * 100 : 100;
  const fpRate = totalFrontend > 0 ? (extraInFrontend.length / totalFrontend) * 100 : 0;
  const fnRate = totalBackend > 0 ? (missingInFrontend.length / totalBackend) * 100 : 0;

  return {
    total_backend_findings: totalBackend,
    total_frontend_findings: totalFrontend,
    matched_findings: matched,
    missing_in_frontend: missingInFrontend,
    extra_in_frontend: extraInFrontend,
    duplicated_in_frontend: duplicated,
    field_mismatches: fieldMismatches,
    accuracy: Math.round(accuracy * 100) / 100,
    false_positive_rate: Math.round(fpRate * 100) / 100,
    false_negative_rate: Math.round(fnRate * 100) / 100,
    timestamp: new Date().toISOString(),
  };
}

// ── Compare individual fields ──
function compareFields(backend: NormalizedFinding, frontend: NormalizedFinding): FieldMismatch[] {
  const mismatches: FieldMismatch[] = [];
  const fieldsToCompare: (keyof NormalizedFinding)[] = [
    'severity', 'confidence', 'status',
  ];

  for (const field of fieldsToCompare) {
    const bVal = String(backend[field] || '').toLowerCase().trim();
    const fVal = String(frontend[field] || '').toLowerCase().trim();
    if (bVal && fVal && bVal !== fVal) {
      mismatches.push({
        field,
        backend_value: bVal,
        frontend_value: fVal,
      });
    }
  }

  // Compare evidence count as a numeric mismatch
  if (backend.evidence_count > 0 && frontend.evidence_count > 0) {
    if (backend.evidence_count !== frontend.evidence_count) {
      mismatches.push({
        field: 'evidence_count',
        backend_value: String(backend.evidence_count),
        frontend_value: String(frontend.evidence_count),
      });
    }
  }

  return mismatches;
}

// ── Detect duplicates by deterministic key ──
function detectDuplicates(findings: NormalizedFinding[]): DuplicateGroup[] {
  const groups = new Map<string, NormalizedFinding[]>();

  for (const f of findings) {
    const key = f.deterministic_key;
    if (!groups.has(key)) {
      groups.set(key, []);
    }
    groups.get(key)!.push(f);
  }

  const duplicates: DuplicateGroup[] = [];
  for (const [key, items] of groups) {
    if (items.length > 1) {
      duplicates.push({ deterministic_key: key, findings: items });
    }
  }

  return duplicates;
}
