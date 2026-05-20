import { httpClient } from "@/shared/api/http-client";
import type { ApiExploitResult } from "@/shared/types/api";
import type { FindingId } from "@/shared/types/common";

/** Verify a finding via the active detection engine */
export async function verifyFinding(findingId: FindingId): Promise<ApiExploitResult> {
  return httpClient.post<ApiExploitResult>(`/api/active-findings/${findingId}/verify`, {});
}

/** Mark a finding as verified or false positive */
export async function updateFindingStatus(
  findingId: FindingId,
  data: { verified: boolean; false_positive: boolean }
): Promise<void> {
  return httpClient.patch(`/api/active-findings/${findingId}`, data);
}
