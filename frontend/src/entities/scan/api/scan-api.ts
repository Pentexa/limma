import { httpClient } from "@/shared/api/http-client";
import type { Scan } from "../model/types";
import type { ApiActiveScanResult, ApiTrendPoint } from "@/shared/types/api";
import { mapActiveScanToScan, mapActiveScanList } from "../lib/scan-mapper";

/** Fetch all active scans (running + completed) */
export async function fetchScans(): Promise<Scan[]> {
  const raw = await httpClient.get<ApiActiveScanResult[]>("/api/active-scans", {
    params: { _t: Date.now() }
  });
  const arr = Array.isArray(raw) ? raw : [];
  return mapActiveScanList(arr);
}

/** Fetch all scans from history */
export async function fetchHistoryScans(): Promise<Scan[]> {
  const raw = await httpClient.get<ApiActiveScanResult[]>("/api/history/scans", {
    params: { _t: Date.now() }
  });
  return mapActiveScanList(raw);
}

/** Fetch a single active scan by ID */
export async function fetchScan(id: string): Promise<Scan> {
  const raw = await httpClient.get<ApiActiveScanResult>(`/api/active-scan/${id}`);
  return mapActiveScanToScan(raw);
}

/** Fetch a single scan from history by ID */
export async function fetchHistoryScan(id: string): Promise<Scan> {
  const raw = await httpClient.get<ApiActiveScanResult>(`/api/history/scan/${id}`);
  return mapActiveScanToScan(raw);
}

/** Fetch scan history trends for a target URL */
export async function fetchScanTrends(targetUrl: string): Promise<ApiTrendPoint[]> {
  return httpClient.get<ApiTrendPoint[]>("/api/history/trends", {
    params: { target_url: targetUrl },
  });
}

/** Delete an active scan */
export async function deleteScan(id: string): Promise<void> {
  return httpClient.delete(`/api/active-scans/${id}`);
}

/** Delete a scan from history */
export async function deleteHistoryScan(id: string): Promise<void> {
  return httpClient.delete(`/api/history/scan/${id}`);
}
