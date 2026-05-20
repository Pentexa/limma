/**
 * Centralized severity weight/priority definitions.
 * Used across the app for sorting findings, PoC candidates, etc.
 *
 * P1-011: Eliminates duplicate `{ critical: 4, high: 3, ... }` maps
 * scattered across PocLabScreen, ActiveDetectionScreen, etc.
 */
import type { Severity } from "@/shared/types/common";

/** Numeric weight for each severity level — higher = more critical */
export const SEVERITY_WEIGHT: Record<Severity, number> = {
  critical: 4,
  high: 3,
  medium: 2,
  low: 1,
  info: 0,
};

/**
 * Comparator to sort items by severity (descending — critical first).
 * Use with Array.prototype.sort:
 *
 * ```ts
 * findings.sort(compareBySeverity);
 * ```
 */
export function compareBySeverity<T extends { severity: string }>(
  a: T,
  b: T
): number {
  return (
    (SEVERITY_WEIGHT[b.severity as Severity] ?? 0) -
    (SEVERITY_WEIGHT[a.severity as Severity] ?? 0)
  );
}
