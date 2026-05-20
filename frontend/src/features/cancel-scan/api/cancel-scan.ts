import { httpClient } from "@/shared/api/http-client";
export async function cancelScan(scanId: string): Promise<void> {
  return httpClient.delete(`/api/active-scans/${scanId}`);
}
