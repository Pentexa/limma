import { httpClient } from "@/shared/api/http-client";
import type { Report } from "../model/types";

const LOCAL_STORAGE_KEY = "limma_reports_history";

export function getLocalReports(): Report[] {
  if (typeof window === "undefined") return [];
  try {
    const data = localStorage.getItem(LOCAL_STORAGE_KEY);
    return data ? JSON.parse(data) : [];
  } catch {
    return [];
  }
}

export function saveLocalReport(report: Report) {
  if (typeof window === "undefined") return;
  const reports = getLocalReports();
  localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify([report, ...reports]));
}

export async function fetchReports(): Promise<Report[]> {
  try {
    const apiReports = await httpClient.get<Report[]>("/api/reports");
    return [...getLocalReports(), ...(Array.isArray(apiReports) ? apiReports : [])];
  } catch (err) {
    return getLocalReports();
  }
}

export async function fetchReport(id: string): Promise<Report> {
  const local = getLocalReports().find((r) => r.id === id);
  if (local) return local;
  return httpClient.get<Report>(`/api/reports/${id}`);
}
