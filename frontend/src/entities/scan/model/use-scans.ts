"use client";

import { useQuery, keepPreviousData } from "@tanstack/react-query";
import { fetchScans, fetchScan } from "../api/scan-api";
import type { Scan } from "../model/types";

/** Query keys for scan data */
export const scanKeys = {
  all: ["scans"] as const,
  lists: () => [...scanKeys.all, "list"] as const,
  list: (filters?: Record<string, string>) => [...scanKeys.lists(), filters] as const,
  details: () => [...scanKeys.all, "detail"] as const,
  detail: (id: string) => [...scanKeys.details(), id] as const,
};

/** Fetch all active scans — polls faster when a scan is running */
export function useScans() {
  return useQuery<Scan[], Error>({
    queryKey: scanKeys.lists(),
    queryFn: fetchScans,
    // Keep previous data during refetch to avoid UI flash/flicker
    placeholderData: keepPreviousData,
    refetchInterval: (query) => {
      const scans = query.state.data;
      const hasRunning = scans?.some((s) => s.status === "running");
      // Poll every 5s during active scans, 30s otherwise
      return hasRunning ? 5000 : 30000;
    },
  });
}

/** Fetch a single scan by ID */
export function useScan(id: string | undefined) {
  return useQuery<Scan, Error>({
    queryKey: scanKeys.detail(id ?? ""),
    queryFn: () => fetchScan(id!),
    enabled: !!id,
    placeholderData: keepPreviousData,
    refetchInterval: (query) => {
      const scan = query.state.data;
      return scan?.status === "running" ? 5000 : 30000;
    },
  });
}
