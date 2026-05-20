"use client";

import { cn } from "@/shared/lib/utils";
import { DETECTOR_META, type DetectorType } from "@/entities/finding/model/types";
import type { Severity } from "@/shared/types/common";
import type { VerificationStatus } from "@/entities/finding/model/types";

const SEVERITIES: { value: Severity; label: string; dot: string }[] = [
  { value: "critical", label: "Critical", dot: "sev-dot-critical" },
  { value: "high", label: "High", dot: "sev-dot-high" },
  { value: "medium", label: "Medium", dot: "sev-dot-medium" },
  { value: "low", label: "Low", dot: "sev-dot-low" },
  { value: "info", label: "Info", dot: "bg-muted-foreground/40" },
];

const STATUSES: { value: VerificationStatus | "all"; label: string }[] = [
  { value: "all", label: "All" },
  { value: "verified", label: "Verified" },
  { value: "unverified", label: "Unverified" },
  { value: "false_positive", label: "False Positive" },
];

interface SeverityFilterBarProps {
  selectedSeverities: Set<Severity>;
  onToggleSeverity: (severity: Severity) => void;
  selectedDetector: string | null;
  onSelectDetector: (detector: string | null) => void;
  selectedStatus: VerificationStatus | "all";
  onSelectStatus: (status: VerificationStatus | "all") => void;
  counts: Record<string, number>;
}

export function SeverityFilterBar({
  selectedSeverities,
  onToggleSeverity,
  selectedDetector,
  onSelectDetector,
  selectedStatus,
  onSelectStatus,
  counts,
}: SeverityFilterBarProps) {
  const detectors = Object.entries(DETECTOR_META) as [DetectorType, { name: string; category: string }][];

  return (
    <div className="space-y-5">
      {/* Severity Filters */}
      <div>
        <span className="text-[12px] font-semibold text-foreground/80 mb-3 block px-1">
          Severity
        </span>
        <div className="space-y-1">
          {SEVERITIES.map((s) => {
            const active = selectedSeverities.has(s.value);
            const count = counts[s.value] ?? 0;
            return (
              <button
                key={s.value}
                className={cn(
                  "w-full flex items-center justify-between px-3 py-2 rounded-lg transition-all duration-150",
                  active
                    ? "bg-primary/[0.08] text-primary"
                    : "text-muted-foreground/60 hover:bg-white/[0.02] hover:text-foreground/90"
                )}
                onClick={() => onToggleSeverity(s.value)}
              >
                <span className="flex items-center gap-2.5 text-[12px] font-medium">
                  <span className={cn("sev-dot", s.dot, !active && "opacity-30")} />
                  {s.label}
                </span>
                <span className={cn("font-mono text-[10px] tabular-nums px-2 py-0.5 rounded-md",
                  active ? "bg-white/[0.06] text-primary" : "text-muted-foreground/40 bg-white/[0.02]"
                )}>
                  {count}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Status Filter */}
      <div>
        <span className="text-[12px] font-semibold text-foreground/80 mb-3 block px-1">
          Status
        </span>
        <div className="space-y-1">
          {STATUSES.map((s) => {
            const count = counts[s.value] ?? 0;
            const active = selectedStatus === s.value;
            return (
              <button
                key={s.value}
                className={cn(
                  "w-full flex items-center justify-between px-3 py-2 rounded-lg transition-all duration-150",
                  active
                    ? "bg-primary/[0.08] text-primary"
                    : "text-muted-foreground/60 hover:bg-white/[0.02] hover:text-foreground/90"
                )}
                onClick={() => onSelectStatus(s.value)}
              >
                <span className="text-[12px] font-medium">{s.label}</span>
                <span className={cn("font-mono text-[10px] tabular-nums px-2 py-0.5 rounded-md",
                  active ? "bg-white/[0.06] text-primary" : "text-muted-foreground/40 bg-white/[0.02]"
                )}>
                  {count}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Detector Filter */}
      <div>
        <span className="text-[12px] font-semibold text-foreground/80 mb-3 block px-1">
          Detector
        </span>
        <div className="space-y-1 max-h-[240px] overflow-y-auto pr-1">
          <button
            className={cn(
              "w-full text-left px-3 py-2 rounded-lg text-[12px] font-medium transition-all duration-150",
              !selectedDetector
                ? "bg-primary/[0.08] text-primary"
                : "text-muted-foreground/60 hover:bg-white/[0.02] hover:text-foreground/90"
            )}
            onClick={() => onSelectDetector(null)}
          >
            All Detectors
          </button>
          {detectors.map(([id, meta]) => (
            <button
              key={id}
              className={cn(
                "w-full text-left px-3 py-2 rounded-lg text-[12px] font-medium transition-all duration-150 truncate",
                selectedDetector === id
                  ? "bg-primary/[0.08] text-primary"
                  : "text-muted-foreground/60 hover:bg-white/[0.02] hover:text-foreground/90"
              )}
              onClick={() => onSelectDetector(id)}
            >
              {meta.name}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
