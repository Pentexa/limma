import type { Severity } from "@/shared/types/common";

/** Get human-readable label for severity */
export function formatSeverity(severity: Severity): string {
  return severity.charAt(0).toUpperCase() + severity.slice(1);
}

/** Get CSS color class for severity */
export function getSeverityColor(severity: Severity): string {
  const colors: Record<Severity, string> = {
    critical: "text-critical",
    high: "text-high",
    medium: "text-medium",
    low: "text-low",
    info: "text-muted-foreground",
  };
  return colors[severity];
}

/** Get CSS background class for severity badges */
export function getSeverityBgColor(severity: Severity): string {
  const colors: Record<Severity, string> = {
    critical: "bg-critical/15 text-critical border-critical/30",
    high: "bg-high/15 text-high border-high/30",
    medium: "bg-medium/15 text-medium border-medium/30",
    low: "bg-low/15 text-low border-low/30",
    info: "bg-muted text-muted-foreground border-border",
  };
  return colors[severity];
}

/** Sort findings by severity (critical first) */
export function severityOrder(severity: Severity): number {
  const order: Record<Severity, number> = {
    critical: 0,
    high: 1,
    medium: 2,
    low: 3,
    info: 4,
  };
  return order[severity];
}
