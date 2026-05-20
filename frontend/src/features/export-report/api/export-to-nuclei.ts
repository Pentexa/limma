import { httpClient } from "@/shared/api/http-client";

export interface ExportNucleiResponse {
  yaml: string;
  template_count: number;
}

/** Export scan report to Nuclei YAML template format */
export async function exportToNuclei(scanId: string): Promise<ExportNucleiResponse> {
  return httpClient.post<ExportNucleiResponse>("/api/export/nuclei", { scan_id: scanId });
}
