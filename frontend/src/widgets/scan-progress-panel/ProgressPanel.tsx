"use client";

import type { ScanResult } from "@/entities/scan/model/types";
import type { Finding } from "@/entities/finding/model/types";
import { cn } from "@/shared/lib/utils";

interface ProgressPanelProps {
  result: ScanResult | null;
  findings?: Finding[];
}

export function ProgressPanel({ result, findings = [] }: ProgressPanelProps) {
  // Prefer actual findings data over scan summary (which may be empty)
  const hasFindings = findings.length > 0;
  const totalFindings = hasFindings ? findings.length : (result?.totalFindings ?? 0);
  const criticalCount = hasFindings ? findings.filter(f => f.severity === "critical").length : (result?.criticalCount ?? 0);
  const highCount = hasFindings ? findings.filter(f => f.severity === "high").length : (result?.highCount ?? 0);
  const mediumCount = hasFindings ? findings.filter(f => f.severity === "medium").length : (result?.mediumCount ?? 0);
  const lowCount = hasFindings ? findings.filter(f => f.severity === "low").length : (result?.lowCount ?? 0);

  if (totalFindings === 0 && !result) return <p className="text-[11px] text-muted-foreground py-4 text-center">No results</p>;

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between text-[11px]">
        <span className="text-muted-foreground">Total</span>
        <span className="text-[14px] font-bold tabular-nums">{totalFindings}</span>
      </div>
      {[
        { label: "Critical", count: criticalCount, cls: "sev-badge-critical" },
        { label: "High", count: highCount, cls: "sev-badge-high" },
        { label: "Medium", count: mediumCount, cls: "sev-badge-medium" },
        { label: "Low", count: lowCount, cls: "sev-badge-low" },
      ].filter(s => s.count > 0).map(s => (
        <div key={s.label} className="flex items-center justify-between text-[11px]">
          <span className="text-muted-foreground">{s.label}</span>
          <span className={cn("sev-badge", s.cls)}>{s.count}</span>
        </div>
      ))}
    </div>
  );
}

