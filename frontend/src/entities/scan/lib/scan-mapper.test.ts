import { describe, expect, it } from "vitest";
import type { ApiActiveScanResult } from "@/shared/types/api";
import { mapActiveScanList, mapActiveScanToScan } from "./scan-mapper";

const API_SCAN: ApiActiveScanResult = {
  scan_id: "scan-1",
  target_url: "https://example.test",
  status: "completed",
  start_time: "2026-06-20T10:00:00.000Z",
  end_time: "2026-06-20T10:00:05.000Z",
  total_requests: 12,
  findings: [],
  errors: [],
  summary: {
    critical_count: 1,
    high_count: 2,
    medium_count: 3,
    low_count: 4,
    info_count: 5,
    vuln_type_breakdown: {},
    waf_detected: false,
    waf_blocked_requests: 0,
    total_endpoints: 8,
    total_parameters: 13,
  },
};

describe("scan mapper", () => {
  it("maps completed scans with counts, duration, and phase progress", () => {
    const scan = mapActiveScanToScan(API_SCAN);

    expect(scan).toMatchObject({
      id: "scan-1",
      targetUrl: "https://example.test",
      status: "completed",
      currentPhase: "exploit",
      duration: 5000,
      phaseProgress: { recon: 100, analysis: 100, scan: 100, exploit: 100 },
    });
    expect(scan.result).toMatchObject({
      criticalCount: 1,
      highCount: 2,
      totalEndpoints: 8,
      totalParameters: 13,
    });
  });

  it("maps scan lists and preserves running state", () => {
    const scans = mapActiveScanList([
      API_SCAN,
      { ...API_SCAN, scan_id: "scan-2", status: "running", end_time: undefined },
    ]);

    expect(scans).toHaveLength(2);
    expect(scans[1]).toMatchObject({
      id: "scan-2",
      status: "running",
      currentPhase: "scan",
    });
  });
});
