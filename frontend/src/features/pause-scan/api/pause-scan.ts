import { httpClient } from "@/shared/api/http-client";

export async function pauseScan(scanId: string): Promise<void> {
  return httpClient.post(`/api/scans/${scanId}/pause`);
}
