import { httpClient } from "@/shared/api/http-client";

export interface ExportBurpResponse {
  xml: string;
  filename: string;
  item_count: number;
}

/** Export scan report to Burp Suite XML format */
export async function exportToBurp(scanId: string): Promise<ExportBurpResponse> {
  return httpClient.post<ExportBurpResponse>("/api/export/burp", { scan_id: scanId });
}
