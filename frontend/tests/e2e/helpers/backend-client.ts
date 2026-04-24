/**
 * Backend Client — fetches truth data directly from the backend API.
 */
import type { NormalizedFinding } from './types';
import {
  normalizeBackendFinding,
  normalizeBackendCanonicalFinding,
  normalizeBackendDynamicFinding,
} from './finding-matcher';

const BACKEND_BASE = process.env.BACKEND_URL || 'http://localhost:8900';

/**
 * Trigger a master report scan from the backend and return the raw JSON.
 */
export async function fetchBackendMasterReport(
  targetUrl: string,
): Promise<Record<string, unknown>> {
  const res = await fetch(`${BACKEND_BASE}/master-report`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ url: targetUrl }),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Backend /master-report failed (${res.status}): ${text}`);
  }

  return res.json() as Promise<Record<string, unknown>>;
}

/**
 * Extract ALL findings from a backend master report, normalized for comparison.
 * Combines: canonical_findings + raw findings + dynamic_rule_findings.
 */
export function extractBackendFindings(
  report: Record<string, unknown>,
  findingType: 'all' | 'canonical' | 'raw' | 'dynamic' = 'all',
): NormalizedFinding[] {
  const audit = report.normalized_audit as Record<string, unknown> | undefined;
  if (!audit) return [];

  const findings: NormalizedFinding[] = [];

  // Raw SecurityAuditFindings
  if (findingType === 'all' || findingType === 'raw') {
    const rawFindings = (audit.findings || []) as Record<string, unknown>[];
    for (const f of rawFindings) {
      findings.push(normalizeBackendFinding(f));
    }
  }

  // Canonical Findings
  if (findingType === 'all' || findingType === 'canonical') {
    const canonicalFindings = (audit.canonical_findings || []) as Record<string, unknown>[];
    for (const f of canonicalFindings) {
      findings.push(normalizeBackendCanonicalFinding(f));
    }
  }

  // Dynamic Rule Findings
  if (findingType === 'all' || findingType === 'dynamic') {
    const dynamicFindings = (audit.dynamic_rule_findings || []) as Record<string, unknown>[];
    for (const f of dynamicFindings) {
      findings.push(normalizeBackendDynamicFinding(f));
    }
  }

  return findings;
}
