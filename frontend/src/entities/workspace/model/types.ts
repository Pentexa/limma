import type { ID, Timestamp } from "@/shared/types/common";

/** Workspace entity */
export interface Workspace {
  id: ID;
  name: string;
  description: string;
  targetUrls: string[];
  scanCount: number;
  findingCount: number;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}
