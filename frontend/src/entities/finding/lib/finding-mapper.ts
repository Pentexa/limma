/**
 * Map backend ActiveScanFinding (snake_case) → frontend Finding (camelCase).
 */
import type { Finding, DetectorType, Confidence, VerificationStatus } from "../model/types";
import type { Severity } from "@/shared/types/common";
import { asFindingId, asScanId } from "@/shared/types/common";
import type { ApiActiveScanFinding } from "@/shared/types/api";

/** Map backend vuln_type to frontend DetectorType */
function mapDetector(vulnType: string): DetectorType {
  const mapping: Record<string, DetectorType> = {
    reflected_xss: "xss",
    stored_xss: "xss",
    dom_xss: "xss",
    sql_injection_error: "sqli",
    sql_injection_union: "sqli",
    sql_injection_blind_time: "sqli",
    sql_injection_blind_boolean: "sqli",
    command_injection: "cmdi",
    command_injection_blind: "cmdi",
    local_file_inclusion: "lfi",
    remote_file_inclusion: "lfi",
    path_traversal: "lfi",
    server_side_request_forgery: "ssrf",
    xml_external_entity: "xxe",
    open_redirect: "redirect",
    jwt_none_algorithm: "jwt",
    jwt_weak_secret: "jwt",
    no_sql_injection: "nosql",
    server_side_template_injection: "ssti",
    graphql_introspection_enabled: "idor",
    graphql_abuse: "idor",
  };
  return mapping[vulnType] ?? "xss";
}

/** Map backend severity string to frontend Severity */
function mapSeverity(severity: string): Severity {
  const s = severity?.toLowerCase();
  if (s === "critical") return "critical";
  if (s === "high") return "high";
  if (s === "medium") return "medium";
  if (s === "low") return "low";
  return "info";
}

/** Map backend confidence to frontend Confidence */
function mapConfidence(confidence: string): Confidence {
  const c = confidence?.toLowerCase();
  if (c === "confirmed" || c === "definite" || c === "certain") return "confirmed";
  if (c === "high" || c === "firm") return "high";
  if (c === "medium") return "medium";
  if (c === "low") return "low";
  return "tentative";
}

/** Derive verification status from finding flags */
function mapVerification(finding: ApiActiveScanFinding): VerificationStatus {
  if (finding.false_positive) return "false_positive";
  if (finding.verified) return "verified";
  return "unverified";
}

/** Map vuln_type to a human-readable title */
function buildTitle(vulnType: string): string {
  return vulnType
    .replace(/_/g, " ")
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

/** Build CWE from vuln_type (best-effort mapping) */
function mapCwe(vulnType: string): string {
  const cweMap: Record<string, string> = {
    reflected_xss: "CWE-79",
    stored_xss: "CWE-79",
    dom_xss: "CWE-79",
    sql_injection_error: "CWE-89",
    sql_injection_union: "CWE-89",
    sql_injection_blind_time: "CWE-89",
    sql_injection_blind_boolean: "CWE-89",
    command_injection: "CWE-78",
    command_injection_blind: "CWE-78",
    local_file_inclusion: "CWE-98",
    remote_file_inclusion: "CWE-98",
    path_traversal: "CWE-22",
    server_side_request_forgery: "CWE-918",
    xml_external_entity: "CWE-611",
    open_redirect: "CWE-601",
    jwt_none_algorithm: "CWE-347",
    jwt_weak_secret: "CWE-347",
    no_sql_injection: "CWE-943",
    server_side_template_injection: "CWE-1336",
  };
  return cweMap[vulnType] ?? "CWE-0";
}

/** Estimate CVSS from severity */
function estimateCvss(severity: string): number | null {
  const s = severity?.toLowerCase();
  if (s === "critical") return 9.5;
  if (s === "high") return 7.5;
  if (s === "medium") return 5.5;
  if (s === "low") return 3.0;
  return null;
}

/** Map a single backend finding to a frontend Finding entity */
export function mapActiveFindingToFinding(api: ApiActiveScanFinding): Finding {
  return {
    id: asFindingId(api.id ?? String(Date.now())),
    scanId: asScanId(api.scan_id ?? ""),
    detector: mapDetector(api.vuln_type),
    title: buildTitle(api.vuln_type),
    description: `${buildTitle(api.vuln_type)} detected at ${api.affected_parameter} parameter`,
    severity: mapSeverity(api.severity),
    confidence: mapConfidence(api.confidence),
    verification: mapVerification(api),
    url: api.target_url,
    parameter: api.affected_parameter,
    method: api.http_method,
    payload: api.payload_used,
    response: api.evidence?.response_raw?.slice(0, 500) ?? "",
    evidence: [
      api.evidence?.matched_indicator,
      ...(api.evidence?.additional_notes ?? []),
    ].filter(Boolean) as string[],
    cwe: mapCwe(api.vuln_type),
    cvss: estimateCvss(api.severity),
    references: [],
    createdAt: api.timestamp,
    updatedAt: api.timestamp,
  };
}

/** Map an array of backend findings */
export function mapActiveFindingList(apiFindings: ApiActiveScanFinding[]): Finding[] {
  return apiFindings.map(mapActiveFindingToFinding);
}
export function mapMasterFindingToFinding(api: Record<string, unknown>, scanId: string): Finding {
  const evidence = api.evidence as Array<Record<string, unknown>> | undefined;
  const riskScore = api.risk_score as Record<string, unknown> | undefined;
  return {
    id: asFindingId((api.id as string) ?? Math.random().toString(36).substring(7)),
    scanId: asScanId(scanId || "master-scan"),
    detector: mapDetector((api.category as string) || (api.source_module as string) || 'unknown'),
    title: (api.summary as string) || 'Security Insight',
    description: (api.technical_details as string) || (api.summary as string) || '',
    severity: mapSeverity((api.severity as string) || ''),
    confidence: mapConfidence((api.confidence as string) || ''),
    verification: api.status === 'verified' ? 'verified' : 'unverified',
    url: (api.target_identifier as string) || '',
    parameter: (api.affected_path_or_endpoint as string) || '',
    method: (api.method as string) || 'GET',
    payload: '',
    response: '',
    evidence: Array.isArray(evidence) ? evidence.map((e) => (e.raw_data as string) || (e.description as string) || '') : [],
    cwe: '',
    cvss: riskScore?.total_score ? (riskScore.total_score as number) / 10 : estimateCvss((api.severity as string) || ''),
    references: [],
    createdAt: (api.timestamp as string) || new Date().toISOString(),
    updatedAt: (api.timestamp as string) || new Date().toISOString(),
  };
}

export function mapMasterFindingList(apiFindings: Record<string, unknown>[], scanId: string): Finding[] {
  if (!Array.isArray(apiFindings)) return [];
  return apiFindings.map(f => mapMasterFindingToFinding(f, scanId));
}
