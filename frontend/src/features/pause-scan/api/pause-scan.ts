import { httpClient } from "@/shared/api/http-client";

export async function pauseScan(scanId: string): Promise<void> {
  return httpClient.post(`/api/active-scans/${scanId}/pause`);
}

export async function resumeScan(scanId: string): Promise<void> {
  return httpClient.post(`/api/active-scans/${scanId}/resume`);
}
