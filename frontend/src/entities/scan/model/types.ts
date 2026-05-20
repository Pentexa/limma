import type { ID, ScanId, Timestamp } from "@/shared/types/common";

/** Scan execution status */
export type ScanStatus =
  | "idle"
  | "pending"
  | "starting"
  | "running"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

/** Pipeline phase */
export type ScanPhase = "recon" | "analysis" | "scan" | "exploit";

/** Scan configuration */
export interface ScanConfig {
  targetUrl: string;
  profileId?: ID;
  enabledDetectors: string[];
  maxDepth: number;
  maxPages: number;
  followRedirects: boolean;
  customHeaders?: Record<string, string>;
}

/** Scan result summary */
export interface ScanResult {
  totalFindings: number;
  criticalCount: number;
  highCount: number;
  mediumCount: number;
  lowCount: number;
  infoCount: number;
  verifiedCount: number;
  unverifiedCount: number;
}

/** Main scan entity */
export interface Scan {
  id: ScanId;
  targetUrl: string;
  status: ScanStatus;
  currentPhase: ScanPhase;
  phaseProgress: Record<ScanPhase, number>; // 0–100
  config: ScanConfig;
  result: ScanResult | null;
  startedAt: Timestamp | null;
  completedAt: Timestamp | null;
  duration: number; // ms
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

/** Phase info for pipeline visualization */
export interface PhaseInfo {
  phase: ScanPhase;
  label: string;
  progress: number;
  status: "pending" | "active" | "completed" | "error";
  startedAt: Timestamp | null;
  completedAt: Timestamp | null;
}
