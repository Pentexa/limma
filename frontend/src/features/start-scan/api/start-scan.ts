import { httpClient } from "@/shared/api/http-client";
import type { ActiveVulnType } from "@/shared/types/api";

export interface StartScanRequest {
  target_url: string;
  profile_id?: string;
  enabled_detectors?: string[];
  max_depth?: number;
  max_pages?: number;
  
  // New config fields passed from frontend UI
  scan_mode?: string;
  enable_headless_browser?: boolean;
  max_browser_tabs?: number;
  bearer_token?: string;
  cookie?: string;
  custom_headers?: string;
  basic_auth_user?: string;
  basic_auth_pass?: string;
  enable_json_fuzzing?: boolean;
  enable_xss_verification?: boolean;
  allow_destructive_methods?: boolean;
  l3_consent_accepted?: boolean;
  max_scan_duration_sec?: number;
  max_requests_per_endpoint?: number;
  vuln_types?: ActiveVulnType[];
}

export interface StartActiveScanRequest {
  target_url: string;
  vuln_types: ActiveVulnType[];
  scan_mode: string;
  enable_headless_browser: boolean;
  max_browser_tabs: number;
  bearer_token?: string;
  cookie?: string;
  custom_headers?: string;
  basic_auth_user?: string;
  basic_auth_pass?: string;
  enable_json_fuzzing: boolean;
  enable_xss_verification: boolean;
  allow_destructive_methods: boolean;
  l3_consent_accepted: boolean;
  max_scan_duration_sec?: number;
  max_requests_per_endpoint?: number;
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
    vuln_types: data.vuln_types ?? ALL_VULN_TYPES,
    profile_id: data.profile_id,
    scan_mode: data.scan_mode ?? "fast",
    enable_headless_browser: data.enable_headless_browser ?? false,
    max_browser_tabs: data.max_browser_tabs ?? 2,
    bearer_token: data.bearer_token,
    cookie: data.cookie,
    custom_headers: data.custom_headers,
    basic_auth_user: data.basic_auth_user,
    basic_auth_pass: data.basic_auth_pass,
    enable_json_fuzzing: data.enable_json_fuzzing ?? true,
    enable_xss_verification: data.enable_xss_verification ?? true,
    allow_destructive_methods: data.allow_destructive_methods ?? false,
    l3_consent_accepted: data.l3_consent_accepted ?? false,
    max_scan_duration_sec: data.max_scan_duration_sec ?? 3600,
    max_requests_per_endpoint: data.max_requests_per_endpoint,
    follow_redirects: true
  });
}
