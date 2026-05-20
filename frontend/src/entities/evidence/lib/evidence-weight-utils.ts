import type { EvidenceWeight } from "../model/types";

/** Get human-readable label for evidence weight */
export function formatEvidenceWeight(weight: EvidenceWeight): string {
  const labels: Record<EvidenceWeight, string> = {
    strong: "Strong",
    moderate: "Moderate",
    weak: "Weak",
    circumstantial: "Circumstantial",
  };
  return labels[weight];
}

/** Get CSS color class for evidence weight */
export function getWeightColor(weight: EvidenceWeight): string {
  const colors: Record<EvidenceWeight, string> = {
    strong: "text-evidence-strong",
    moderate: "text-analysis",
    weak: "text-attention",
    circumstantial: "text-muted-foreground",
  };
  return colors[weight];
}
