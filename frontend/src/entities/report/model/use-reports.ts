"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchReports, fetchReport } from "../api/report-api";
import type { Report } from "../model/types";

export const reportKeys = {
  all: ["reports"] as const,
  lists: () => [...reportKeys.all, "list"] as const,
  details: () => [...reportKeys.all, "detail"] as const,
  detail: (id: string) => [...reportKeys.details(), id] as const,
};

/** Fetch all reports */
export function useReports() {
  return useQuery<Report[], Error>({
    queryKey: reportKeys.lists(),
    queryFn: fetchReports,
  });
}

/** Fetch single report by ID */
export function useReport(id: string | undefined) {
  return useQuery<Report, Error>({
    queryKey: reportKeys.detail(id ?? ""),
    queryFn: () => fetchReport(id!),
    enabled: !!id,
  });
}
