import { describe, expect, it } from "vitest";
import type { ApiActiveScanFinding } from "@/shared/types/api";
import { mapActiveFindingList, mapActiveFindingToFinding } from "./finding-mapper";

const API_FINDING: ApiActiveScanFinding = {
  id: "finding-1",
  scan_id: "scan-1",
  timestamp: "2026-06-20T10:00:00.000Z",
  vuln_type: "sql_injection_union",
  target_url: "https://example.test/search?q=test",
  affected_parameter: "q",
  http_method: "GET",
  payload_used: "' UNION SELECT NULL--",
  evidence: {
    request_raw: "GET /search?q=test HTTP/1.1",
    response_raw: "database error",
    response_time_ms: 42,
    matched_indicator: "SQL syntax",
    additional_notes: ["Union response shape changed"],
  },
  severity: "HIGH",
  confidence: "firm",
  exploitability: "actionable",
  poc_generated: false,
  verified: true,
  false_positive: false,
};

describe("finding mapper", () => {
  it("maps backend finding fields and security metadata", () => {
    const finding = mapActiveFindingToFinding(API_FINDING);

    expect(finding).toMatchObject({
      id: "finding-1",
      scanId: "scan-1",
      detector: "sqli",
      title: "Sql Injection Union",
      severity: "high",
      confidence: "high",
      verification: "verified",
      parameter: "q",
      cwe: "CWE-89",
      cvss: 7.5,
    });
    expect(finding.evidence).toEqual(["SQL syntax", "Union response shape changed"]);
  });

  it("maps lists without changing item order", () => {
    const second = { ...API_FINDING, id: "finding-2", false_positive: true };

    const findings = mapActiveFindingList([API_FINDING, second]);

    expect(findings.map((finding) => finding.id)).toEqual(["finding-1", "finding-2"]);
    expect(findings[1].verification).toBe("false_positive");
  });
});
