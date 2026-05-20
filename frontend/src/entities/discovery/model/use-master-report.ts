"use client";

import { useQuery } from "@tanstack/react-query";
import { httpClient } from "@/shared/api/http-client";
import type { ApiMasterReport } from "@/shared/types/api";
import { useScans } from "@/entities/scan/model/use-scans";
import { useStreamStore } from "@/features/stream-scan-events/model/stream-store";

/** Fetch the master report for a given URL */
async function fetchMasterReport(url: string): Promise<ApiMasterReport> {
  return httpClient.post<ApiMasterReport>("/master-report", { url });
}

/** Query keys */
export const masterReportKeys = {
  all: ["master-report"] as const,
  byUrl: (url: string) => [...masterReportKeys.all, url] as const,
};

function normalizeReportUrl(url: string | undefined): string {
  if (!url) return "";
  const trimmed = url.trim();
  if (!trimmed.startsWith("http://") && !trimmed.startsWith("https://")) {
    return `https://${trimmed}`;
  }
  return trimmed;
}

/** Hook: fetch master report for a target URL */
export function useMasterReport(targetUrl: string | undefined) {
  const normalizedUrl = normalizeReportUrl(targetUrl);
  return useQuery<ApiMasterReport, Error>({
    queryKey: masterReportKeys.byUrl(normalizedUrl),
    queryFn: () => fetchMasterReport(normalizedUrl),
    enabled: !!normalizedUrl,
    staleTime: 5 * 60 * 1000, // 5 min — expensive call
    refetchOnWindowFocus: false,
  });
}

/**
 * Hook: auto-detect target URL from the best available scan, then fetch master report.
 *
 * Priority order:
 *  1. Running scan (actively in progress)
 *  2. Starting scan (from stream store — user just clicked Start)
 *  3. Most recently completed scan (has results to show)
 *  4. Most recent pending scan
 *
 * This ensures the Discovery page always shows data when available,
 * even after scans complete quickly.
 */
export function useActiveMasterReport() {
  const { data: scans = [] } = useScans();
  const localScanTarget = useStreamStore((s) => s.localScanTarget);
  const localScanState = useStreamStore((s) => s.localScanState);

  // 1. Prefer a running scan
  const runningScan = scans.find((s) => s.status === "running");

  // 2. Fall back to most recently completed scan (sorted by completedAt desc)
  const completedScans = scans
    .filter((s) => s.status === "completed" && s.targetUrl)
    .sort((a, b) => {
      const ta = a.completedAt ? new Date(a.completedAt).getTime() : 0;
      const tb = b.completedAt ? new Date(b.completedAt).getTime() : 0;
      return tb - ta;
    });
  const latestCompletedScan = completedScans[0] ?? null;

  // 3. Fall back to most recent pending scan
  const pendingScan = scans.find((s) => s.status === "pending" || s.status === "starting");

  // Determine best scan to use
  const bestScan = runningScan ?? latestCompletedScan ?? pendingScan ?? null;

  // Also consider localScanTarget from stream store (instant feedback)
  const targetUrl =
    bestScan?.targetUrl ??
    (localScanState !== "idle" ? localScanTarget : null) ??
    null;

  const isScanning = !!(runningScan || localScanState === "starting" || localScanState === "running");

  const report = useMasterReport(targetUrl ?? undefined);

  return {
    ...report,
    targetUrl,
    scanId: bestScan?.id ?? null,
    isScanning,
  };
}

