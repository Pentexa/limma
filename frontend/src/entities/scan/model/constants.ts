import type { Scan } from "./types";

export const EMPTY_SCAN: Scan = {
  id: "" as import("@/shared/types/common").ScanId,
  targetUrl: "—",
  status: "idle",
  currentPhase: "recon",
  phaseProgress: { recon: 0, analysis: 0, scan: 0, exploit: 0 },
  config: {
    targetUrl: "—",
    enabledDetectors: [],
    maxDepth: 3,
    maxPages: 100,
    followRedirects: true,
  },
  result: null,
  startedAt: null,
  completedAt: null,
  duration: 0,
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
};
