import { httpClient } from "@/shared/api/http-client";
import type { ApiExploitResult } from "@/shared/types/api";
import type { FindingId } from "@/shared/types/common";

import type { SafetyLevel } from "@/features/blind-scan/api/blind-scan-api";

/** Verify a finding via the active detection engine */
export async function verifyFinding(findingId: FindingId, execution_level?: SafetyLevel): Promise<ApiExploitResult> {
  return httpClient.post<ApiExploitResult>(`/api/active-findings/${findingId}/verify`, { execution_level });
}

/** Mark a finding as verified or false positive */
export async function updateFindingStatus(
  findingId: FindingId,
  data: { verified: boolean; false_positive: boolean }
): Promise<void> {
  return httpClient.patch(`/api/active-findings/${findingId}`, data);
}
