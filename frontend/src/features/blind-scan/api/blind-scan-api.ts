import { httpClient } from "@/shared/api/http-client";
import type { FindingId } from "@/shared/types/common";

export interface BlindScanRequest {
  target_url: string;
  scan_id?: string;
  target_id?: string;
  detection_types?: string[];
  max_duration_seconds?: number;
}

export interface PocGenerateRequest {
  finding_id: string;
  preferred_language?: string;
}

export interface ExploitVerifyRequest {
  poc_id: string;
  execution_level?: string;
}

/** POST /api/blind-scan — Execute blind vulnerability detection */
export function runBlindScan(data: BlindScanRequest) {
  return httpClient.post<Record<string, unknown>>("/api/blind-scan", data);
}

/** POST /api/poc/generate — Generate a PoC for a finding */
export function generatePoc(data: PocGenerateRequest) {
  return httpClient.post<Record<string, unknown>>("/api/poc/generate", data);
}

/** POST /api/exploit/verify — Verify PoC in sandbox */
export function verifyExploit(data: ExploitVerifyRequest) {
  return httpClient.post<Record<string, unknown>>("/api/exploit/verify", data);
}

/** GET /api/poc/:id — Download PoC code */
export function downloadPoc(pocId: string) {
  return httpClient.get<Record<string, unknown>>(`/api/poc/${pocId}`);
}

/** POST /api/active-findings/:id/poc — Generate PoC for active finding */
export function generatePocForFinding(findingId: FindingId) {
  return httpClient.post<Record<string, unknown>>(`/api/active-findings/${findingId}/poc`, {});
}

/** POST /proxy-request — HTTP proxy */
export function proxyRequest(data: { url: string; method: string; body?: string }) {
  return httpClient.post<Record<string, unknown>>("/proxy-request", data);
}
