import type { ID } from "@/shared/types/common";
import type { Severity } from "@/shared/types/common";

/** Attack path step */
export interface AttackPathStep {
  order: number;
  findingId: ID;
  description: string;
  severity: Severity;
}

/** Attack path chain */
export interface AttackPath {
  id: ID;
  scanId: ID;
  title: string;
  description: string;
  steps: AttackPathStep[];
  overallSeverity: Severity;
  exploitability: number; // 0-10
}
