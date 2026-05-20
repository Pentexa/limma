import type { ID, Timestamp } from "@/shared/types/common";

/** Detection rule */
export interface Rule {
  id: ID;
  name: string;
  description: string;
  detector: string;
  severity: string;
  enabled: boolean;
  pattern: string;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}
