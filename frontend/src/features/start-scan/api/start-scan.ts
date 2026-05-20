import { httpClient } from "@/shared/api/http-client";
import type { ActiveVulnType } from "@/shared/types/api";

export interface StartScanRequest {
  target_url: string;
  profile_id?: string;
  enabled_detectors?: string[];
  max_depth?: number;
  max_pages?: number;
}

export interface StartActiveScanRequest {
  target_url: string;
  vuln_types: ActiveVulnType[];
  max_duration_seconds?: number;
  rate_limit_rps?: number;
  follow_redirects?: boolean;
  profile_id?: string;
  custom_parameters?: string[];
}

export interface StartScanResponse {
  scan_id: string;
  status: string;
  message?: string;
}

/** Start a passive website scan (analysis + stream) */
export async function startPassiveScan(data: StartScanRequest): Promise<StartScanResponse> {
  const result = await httpClient.post<Record<string, unknown>>("/analyze", {
    url: data.target_url,
  });
  return {
    scan_id: String(result.scan_id ?? ""),
    status: "running",
    message: "Passive scan started",
  };
}

/** Start an active vulnerability scan */
export async function startActiveScan(data: StartActiveScanRequest): Promise<StartScanResponse> {
  return httpClient.post<StartScanResponse>("/api/active-scan", data);
}

export async function startScan(data: StartScanRequest): Promise<StartScanResponse> {
  const ALL_VULN_TYPES: ActiveVulnType[] = [
    "reflected_xss", "stored_xss", "dom_xss",
    "sql_injection_error", "sql_injection_union", "sql_injection_blind_time", "sql_injection_blind_boolean",
    "command_injection", "command_injection_blind",
    "local_file_inclusion", "remote_file_inclusion", "path_traversal",
    "server_side_request_forgery", "xml_external_entity", "open_redirect",
    "jwt_none_algorithm", "jwt_weak_secret", "no_sql_injection",
    "server_side_template_injection", "graphql_introspection_enabled", "graphql_abuse"
  ];

  // Fire passive scan in background so SSE stream receives events.
  // We swallow the error silently to prevent Next.js dev overlays from popping up
  // if the backend is missing the /analyze endpoint or returns an error.
  startPassiveScan(data).catch(() => {});

  return startActiveScan({
    target_url: data.target_url,
    vuln_types: ALL_VULN_TYPES,
    profile_id: data.profile_id,
    max_duration_seconds: 3600,
    rate_limit_rps: 50,
    follow_redirects: true
  });
}
