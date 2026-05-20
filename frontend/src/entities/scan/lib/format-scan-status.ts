import type { ScanStatus } from "../model/types";

/** Get a human-readable label for scan status */
export function formatScanStatus(status: ScanStatus): string {
  const labels: Record<ScanStatus, string> = {
    idle: "Waiting",
    pending: "Pending",
    starting: "Starting…",
    running: "Running",
    paused: "Paused",
    completed: "Completed",
    failed: "Failed",
    cancelled: "Cancelled",
  };
  return labels[status];
}

/** Get CSS color class for scan status */
export function getScanStatusColor(status: ScanStatus): string {
  const colors: Record<ScanStatus, string> = {
    idle: "text-muted-foreground",
    pending: "text-primary",
    starting: "text-attention",
    running: "text-recon",
    paused: "text-attention",
    completed: "text-analysis",
    failed: "text-risk",
    cancelled: "text-muted-foreground",
  };
  return colors[status];
}

/** Check if scan is in a terminal state */
export function isScanTerminal(status: ScanStatus): boolean {
  return ["completed", "failed", "cancelled"].includes(status);
}

/** Check if scan is active (can receive events) */
export function isScanActive(status: ScanStatus): boolean {
  return ["starting", "running", "paused"].includes(status);
}
