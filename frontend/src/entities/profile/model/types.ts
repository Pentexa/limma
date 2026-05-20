import type { ID } from "@/shared/types/common";
import type { DetectorType } from "@/entities/finding/model/types";

/** Scan profile preset */
export interface ScanProfile {
  id: ID;
  name: string;
  description: string;
  enabledDetectors: DetectorType[];
  maxDepth: number;
  maxPages: number;
  isDefault: boolean;
}
