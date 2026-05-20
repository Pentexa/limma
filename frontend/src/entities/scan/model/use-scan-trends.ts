"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchScanTrends } from "../api/scan-api";
import type { ApiTrendPoint } from "@/shared/types/api";

export const trendKeys = {
  all: ["scan-trends"] as const,
  byTarget: (url: string) => [...trendKeys.all, url] as const,
};

/** Fetch scan trends for a target URL */
export function useScanTrends(targetUrl: string | undefined) {
  return useQuery<ApiTrendPoint[], Error>({
    queryKey: trendKeys.byTarget(targetUrl ?? ""),
    queryFn: () => fetchScanTrends(targetUrl!),
    enabled: !!targetUrl && targetUrl !== "—",
    staleTime: 60 * 1000,
    refetchOnWindowFocus: false,
  });
}
