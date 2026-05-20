/**
 * Map backend ActiveScanResult (snake_case) → frontend Scan (camelCase).
 */
import type { Scan, ScanResult, ScanStatus } from "../model/types";
import type { ApiActiveScanResult } from "@/shared/types/api";
import { asScanId } from "@/shared/types/common";
import { EMPTY_SCAN } from "../model/constants";

/** Map backend status string to frontend ScanStatus */
function mapStatus(status?: string): ScanStatus {
  if (!status) return "completed";
  
  const statusMap: Record<string, ScanStatus> = {
    pending: "pending",
    running: "running",
    completed: "completed",
    failed: "failed",
    cancelled: "cancelled",
  };
  return statusMap[status] ?? "idle";
}

/** Infer current phase from status and summary */
function inferPhase(scan: ApiActiveScanResult) {
  if (scan.status === "completed" || scan.status === "failed") return "exploit" as const;
  if (scan.status === "running") return "scan" as const;
  return "recon" as const;
}

/** Build phase progress from scan status */
function buildPhaseProgress(scan: ApiActiveScanResult): Record<"recon" | "analysis" | "scan" | "exploit", number> {
  const isComplete = scan.status === "completed";
  const isRunning = scan.status === "running";

  if (isComplete) {
    return { recon: 100, analysis: 100, scan: 100, exploit: 100 };
  }

  if (isRunning) {
    // Estimate progress based on findings and total_requests
    const hasFindings = scan.findings && scan.findings.length > 0;
    return {
      recon: 100,
      analysis: 100,
      scan: hasFindings ? 65 : 30,
      exploit: 0,
    };
  }

  return { recon: 0, analysis: 0, scan: 0, exploit: 0 };
}

/** Map backend scan result to frontend ScanResult */
function mapScanResult(scan: ApiActiveScanResult): ScanResult | null {
  if (!scan.summary) return null;
  const s = scan.summary;
  return {
    totalFindings: scan.findings?.length ?? 0,
    criticalCount: s.critical_count,
    highCount: s.high_count,
    mediumCount: s.medium_count,
    lowCount: s.low_count,
    infoCount: s.info_count,
    verifiedCount: scan.findings?.filter((f) => f.verified).length ?? 0,
    unverifiedCount: scan.findings?.filter((f) => !f.verified).length ?? 0,
  };
}

/** Map a single backend scan to a frontend Scan entity */
export function mapActiveScanToScan(api: ApiActiveScanResult): Scan {
  if (!api) return { ...EMPTY_SCAN, id: asScanId(Math.random().toString()) };
  
  return {
    id: asScanId(api.scan_id || String(Date.now())),
    targetUrl: api.target_url || "—",
    status: mapStatus(api.status),
    currentPhase: inferPhase(api),
    phaseProgress: buildPhaseProgress(api),
    config: {
      targetUrl: api.target_url,
      enabledDetectors: [],
      maxDepth: 3,
      maxPages: 100,
      followRedirects: true,
    },
    result: mapScanResult(api),
    startedAt: api.start_time ?? null,
    completedAt: api.end_time ?? null,
    duration: api.start_time && api.end_time
      ? new Date(api.end_time).getTime() - new Date(api.start_time).getTime()
      : 0,
    createdAt: api.start_time ?? new Date().toISOString(),
    updatedAt: api.end_time ?? api.start_time ?? new Date().toISOString(),
  };
}

/** Map an array of backend scans */
export function mapActiveScanList(apiScans: ApiActiveScanResult[]): Scan[] {
  return apiScans.map(mapActiveScanToScan);
}
