import { httpClient } from "@/shared/api/http-client";
export async function resumeScan(scanId: string): Promise<void> {
  return httpClient.post(`/api/scans/${scanId}/resume`);
}
