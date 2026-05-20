import { httpClient } from "@/shared/api/http-client";
import type { ApiBurpSession, ApiBurpFinding } from "@/shared/types/api";

/** Handshake with Burp Suite plugin */
export async function burpHandshake(pluginVersion: string): Promise<{ session_id: string }> {
  return httpClient.post("/api/burp/handshake", { plugin_version: pluginVersion });
}

/** Import traffic from Burp Suite */
export async function burpImportTraffic(
  sessionId: string,
  traffic: unknown
): Promise<{ imported: number }> {
  return httpClient.post("/api/burp/import-traffic", {
    session_id: sessionId,
    traffic,
  });
}

/** Get findings for a Burp session */
export async function burpGetFindings(sessionId: string): Promise<ApiBurpFinding[]> {
  return httpClient.get<ApiBurpFinding[]>(`/api/burp/findings/${sessionId}`);
}

/** List all Burp sessions */
export async function burpListSessions(): Promise<ApiBurpSession[]> {
  return httpClient.get<ApiBurpSession[]>("/api/burp/sessions");
}
