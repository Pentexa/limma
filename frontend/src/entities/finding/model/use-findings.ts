"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchFindings, fetchFilteredFindings, fetchFinding, fetchMasterReportFindings } from "../api/finding-api";
import type { Finding } from "../model/types";

/** Query keys for finding data */
export const findingKeys = {
  all: ["findings"] as const,
  lists: () => [...findingKeys.all, "list"] as const,
  listByScan: (scanId: string) => [...findingKeys.lists(), { scanId }] as const,
  listFiltered: (filters?: Record<string, string>) => [...findingKeys.lists(), filters] as const,
  details: () => [...findingKeys.all, "detail"] as const,
  detail: (id: string) => [...findingKeys.details(), id] as const,
};

/** Fetch findings for a specific scan */
export function useScanFindings(scanId: string | undefined) {
  return useQuery<Finding[], Error>({
    queryKey: findingKeys.listByScan(scanId ?? ""),
    queryFn: () => fetchFindings(scanId!),
    enabled: !!scanId,
    refetchInterval: 20000,
  });
}

/** Fetch all findings with optional filters */
export function useFindings(filters?: Record<string, string>) {
  return useQuery<Finding[], Error>({
    queryKey: findingKeys.listFiltered(filters),
    queryFn: () => fetchFilteredFindings(filters),
    refetchInterval: 30000,
  });
}

/** Fetch a single finding by ID */
export function useFinding(id: string | undefined) {
  return useQuery<Finding, Error>({
    queryKey: findingKeys.detail(id ?? ""),
    queryFn: () => fetchFinding(id!),
    enabled: !!id,
  });
}
/** Fetch deep intelligence findings from Master Report */
export function useMasterReportFindings(targetUrl: string | undefined, scanId: string | undefined) {
  return useQuery<Finding[], Error>({
    queryKey: [...findingKeys.all, "master-report", targetUrl, scanId],
    queryFn: () => fetchMasterReportFindings(targetUrl!, scanId!),
    enabled: !!targetUrl && !!scanId,
    refetchOnWindowFocus: false, // It's an expensive operation
    staleTime: Infinity, // Don't refetch automatically
  });
}
import { useScans } from "@/entities/scan/model/use-scans";

/** Fetch ALL findings (Active + Master) globally for the current active scan */
export function useGlobalFindings() {
  const { data: scans = [], isLoading: scansLoading, isFetching: scansFetching } = useScans();
  const activeScan = scans.find((s) => s.status === "running") ?? scans[0] ?? { id: "", targetUrl: "" };

  const hasActiveScan = activeScan.id !== "";
  const hasTarget = activeScan.targetUrl !== "";

  const {
    data: activeScanFindings = [],
    isLoading: findingsLoading,
    isFetching: findingsFetching,
  } = useScanFindings(
    hasActiveScan ? activeScan.id : undefined
  );

  const {
    data: masterFindings = [],
    isLoading: masterLoading,
    isFetching: masterFetching,
  } = useMasterReportFindings(
    hasTarget ? activeScan.targetUrl : undefined,
    hasActiveScan ? activeScan.id : undefined
  );

  const findings = Array.from(
    new Map([...activeScanFindings, ...masterFindings].map(f => [f.id, f])).values()
  );

  // Only report loading when queries are actually enabled AND fetching.
  // Disabled queries (no active scan yet) should NOT cause a loading state.
  const isTrulyLoading = scansLoading || (hasActiveScan && findingsLoading);

  // isFetching = true during ANY fetch (initial or refetch after invalidation).
  // This is the signal TopBar needs to know data isn't ready yet.
  const isTrulyFetching = scansFetching || (hasActiveScan && (findingsFetching || masterFetching));

  return {
    data: findings,
    isLoading: isTrulyLoading,
    isFetching: isTrulyFetching,
    masterLoading: hasActiveScan ? masterLoading : false,
    activeScanFindings,
    masterFindings
  };
}
