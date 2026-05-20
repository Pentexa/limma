/** API base URL — defaults to env var or localhost fallback */
export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8900";

/** SSE stream endpoint */
export const SSE_STREAM_URL = `${API_BASE_URL}/analyze/stream`;

/** Application metadata */
export const APP_NAME = "LIMMA";
export const APP_DESCRIPTION = "Security Auditing Platform";
export const APP_VERSION = "0.1.0";

/** Scan phases */
export const SCAN_PHASES = [
  "recon",
  "analysis",
  "scan",
  "exploit",
] as const;

export type ScanPhase = (typeof SCAN_PHASES)[number];

/** Phase labels for UI display */
export const PHASE_LABELS: Record<ScanPhase, string> = {
  recon: "Reconnaissance",
  analysis: "Analysis",
  scan: "Scanning",
  exploit: "Exploitation",
};

/** Phase colors (tailwind class names) */
export const PHASE_COLORS: Record<ScanPhase, string> = {
  recon: "text-recon",
  analysis: "text-analysis",
  scan: "text-attention",
  exploit: "text-risk",
};

/** Pagination defaults */
export const DEFAULT_PAGE_SIZE = 25;
export const MAX_PAGE_SIZE = 100;

/** SSE reconnection config */
export const SSE_RECONNECT_DELAY = 3000;
export const SSE_MAX_RETRIES = 10;
