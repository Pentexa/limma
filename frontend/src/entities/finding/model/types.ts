import type { ScanId, FindingId, Timestamp, Severity } from "@/shared/types/common";

/** Finding confidence level */
export type Confidence = "confirmed" | "high" | "medium" | "low" | "tentative";

/** Finding verification status */
export type VerificationStatus = "verified" | "unverified" | "false_positive";
export type Exploitability = "actionable" | "conditional" | "theoretical";

/** Detector that found the vulnerability */
export type DetectorType =
  | "xss"
  | "sqli"
  | "cmdi"
  | "lfi"
  | "rfi"
  | "traversal"
  | "ssrf"
  | "xxe"
  | "redirect"
  | "jwt"
  | "deser"
  | "idor"
  | "nosql"
  | "ssti"
  | "graphql"
  | "host_header"
  | "cors"
  | "smuggling"
  | "cache"
  | string; // Adding generic string fallback to prevent type errors for unknown future detectors

/** Main finding entity */
export interface Finding {
  id: FindingId;
  scanId: ScanId;
  detector: DetectorType;
  title: string;
  description: string;
  severity: Severity;
  confidence: Confidence;
  verification: VerificationStatus;
  url: string;
  parameter: string;
  method: string;
  payload: string;
  request?: string;
  response: string;
  responseTimeMs?: number | null;
  evidence: string[];
  exploitability?: Exploitability | null;
  pocGenerated?: boolean;
  pocId?: string | null;
  cwe: string;
  cvss: number | null;
  references: string[];
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

/** Detector info for grid display */
export interface DetectorInfo {
  id: DetectorType;
  name: string;
  description: string;
  category: string;
  findingCount: number;
  status: "idle" | "running" | "completed" | "error";
  lastRun: Timestamp | null;
}

/** Detector display metadata */
export const DETECTOR_META: Record<string, { name: string; description: string; category: string }> = {
  xss: { name: "XSS Detector", description: "Cross-Site Scripting (Reflected, Stored, DOM)", category: "Injection" },
  sqli: { name: "SQLi Detector", description: "SQL Injection (Error, Union, Blind)", category: "Injection" },
  cmdi: { name: "CMDi Detector", description: "Command Injection (Standard, Blind)", category: "Injection" },
  lfi: { name: "LFI Detector", description: "Local File Inclusion", category: "Inclusion" },
  rfi: { name: "RFI Detector", description: "Remote File Inclusion", category: "Inclusion" },
  traversal: { name: "Path Traversal", description: "Directory Traversal", category: "Inclusion" },
  ssrf: { name: "SSRF Detector", description: "Server-Side Request Forgery", category: "Forgery" },
  xxe: { name: "XXE Detector", description: "XML External Entity", category: "Injection" },
  redirect: { name: "Redirect Detector", description: "Open Redirect", category: "Redirect" },
  jwt: { name: "JWT Detector", description: "JWT Vulnerabilities", category: "Auth" },
  deser: { name: "Deser Detector", description: "Deserialization (Java, PHP, Python)", category: "Injection" },
  idor: { name: "IDOR Detector", description: "Insecure Direct Object Ref", category: "Access" },
  nosql: { name: "NoSQL Detector", description: "NoSQL Injection", category: "Injection" },
  ssti: { name: "SSTI Detector", description: "Server-Side Template Injection", category: "Injection" },
  graphql: { name: "GraphQL Detector", description: "GraphQL Abuse & Introspection", category: "API" },
  host_header: { name: "Host Header", description: "Host Header Injection", category: "Misconfig" },
  cors: { name: "CORS Detector", description: "CORS Misconfiguration", category: "Misconfig" },
  smuggling: { name: "Req Smuggling", description: "HTTP Request Smuggling", category: "Misconfig" },
  cache: { name: "Cache Deception", description: "Web Cache Deception", category: "Misconfig" },
};
