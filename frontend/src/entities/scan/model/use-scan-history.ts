"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchHistoryScans } from "../api/scan-api";
import type { Scan } from "../model/types";

export const historyKeys = {
  all: ["scan-history"] as const,
  lists: () => [...historyKeys.all, "list"] as const,
};

/** Fetch all historical scans */
export function useScanHistory() {
  return useQuery<Scan[], Error>({
    queryKey: historyKeys.lists(),
    queryFn: fetchHistoryScans,
    refetchInterval: 60000,
  });
}
