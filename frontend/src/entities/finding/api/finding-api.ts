import { httpClient } from "@/shared/api/http-client";
import type { Finding } from "../model/types";
import type { ApiActiveScanFinding } from "@/shared/types/api";
import { mapActiveFindingToFinding, mapActiveFindingList, mapMasterFindingList } from "../lib/finding-mapper";
import { toast } from "sonner";

/** Fetch all findings for a specific scan */
export async function fetchFindings(scanId: string): Promise<Finding[]> {
  const raw = await httpClient.get<ApiActiveScanFinding[]>("/api/active-findings", {
    params: { scan_id: scanId }
  });
  // Defensive: ensure raw is an array
  const arr = Array.isArray(raw) ? raw : [];
  const mapped = mapActiveFindingList(arr);
  return mapped;
}

/** Fetch all findings with optional filters (severity, vuln_type, etc.) */
export async function fetchFilteredFindings(
  filters?: Record<string, string>
): Promise<Finding[]> {
  const raw = await httpClient.get<ApiActiveScanFinding[]>("/api/active-findings", {
    params: filters,
  });
  const arr = Array.isArray(raw) ? raw : [];
  const mapped = mapActiveFindingList(arr);
  return mapped;
}

/** Fetch a single finding by ID */
export async function fetchFinding(id: string): Promise<Finding> {
  const raw = await httpClient.get<ApiActiveScanFinding>(`/api/active-finding/${id}`);
  return mapActiveFindingToFinding(raw);
}

/** Update a finding (mark as verified / false positive) */
export async function updateFinding(
  id: string,
  data: { verified: boolean; false_positive: boolean }
): Promise<Finding> {
  const raw = await httpClient.patch<ApiActiveScanFinding>(
    `/api/active-findings/${id}`,
    data
  );
  return mapActiveFindingToFinding(raw);
}
interface MasterReportResponse {
  normalized_audit?: {
    findings?: Record<string, unknown>[];
  };
  analysis?: {
    risk_insights?: Record<string, unknown>[];
  };
}

/** Fetch findings from the Master Report endpoint */
export async function fetchMasterReportFindings(url: string, scanId: string): Promise<Finding[]> {
  if (!url) return [];
  try {
    const raw = await httpClient.post<MasterReportResponse>("/master-report", { url });
    
    // Extract normalized_audit.findings or analysis.risk_insights
    let findingsList: Record<string, unknown>[] = [];
    if (raw?.normalized_audit?.findings && Array.isArray(raw.normalized_audit.findings)) {
      findingsList = raw.normalized_audit.findings;
    } else if (raw?.analysis?.risk_insights && Array.isArray(raw.analysis.risk_insights)) {
      // Fallback to risk insights if audit is missing
      findingsList = raw.analysis.risk_insights;
    }
    
    return mapMasterFindingList(findingsList, scanId);
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : "Unknown error occurred";
    toast.error(`Failed to fetch Master Report: ${errorMsg}`);
    return []; // Return empty gracefully if not available
  }
}
