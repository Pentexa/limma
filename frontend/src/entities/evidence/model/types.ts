import type { ID, Timestamp } from "@/shared/types/common";

/** Evidence weight for confidence scoring */
export type EvidenceWeight = "strong" | "moderate" | "weak" | "circumstantial";

/** Evidence type */
export type EvidenceType =
  | "response_analysis"
  | "timing_analysis"
  | "error_based"
  | "out_of_band"
  | "behavioral"
  | "manual";

/** Main evidence entity */
export interface Evidence {
  id: ID;
  findingId: ID;
  type: EvidenceType;
  weight: EvidenceWeight;
  title: string;
  description: string;
  request: string;
  response: string;
  highlights: string[];
  screenshot?: string;
  createdAt: Timestamp;
}
