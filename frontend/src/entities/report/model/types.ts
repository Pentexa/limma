import type { ID, Timestamp } from "@/shared/types/common";

/** Report format */
export type ReportFormat = "pdf" | "html" | "json";

/** Report status */
export type ReportStatus = "generating" | "completed" | "failed";

/** Report entity */
export interface Report {
  id: ID;
  scanId: ID;
  title: string;
  format: ReportFormat;
  status: ReportStatus;
  fileUrl: string | null;
  findingCount: number;
  criticalCount: number;
  highCount: number;
  createdAt: Timestamp;
}
