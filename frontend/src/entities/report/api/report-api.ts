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
  return getLocalReports();
}

export async function fetchReport(id: string): Promise<Report> {
  const local = getLocalReports().find((r) => r.id === id);
  if (local) return local;
  throw new Error(`Report '${id}' was not found in local report history.`);
}
