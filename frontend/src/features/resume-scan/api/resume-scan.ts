import { httpClient } from "@/shared/api/http-client";

export async function resumeScan(scanId: string): Promise<void> {
  return httpClient.post(`/api/active-scans/${scanId}/resume`);
}
