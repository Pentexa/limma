"use client";

import { SCAN_PHASES, PHASE_LABELS, type ScanPhase } from "@/shared/config/constants";
import { cn } from "@/shared/lib/utils";

interface PhaseProgressProps {
  phaseProgress: Record<ScanPhase, number>;
  currentPhase: ScanPhase;
}

const phaseColors: Record<ScanPhase, string> = {
  recon: "bg-recon",
  analysis: "bg-analysis",
  scan: "bg-attention",
  exploit: "bg-risk",
};

export function PhaseProgress({ phaseProgress, currentPhase }: PhaseProgressProps) {
  return (
    <div className="space-y-2.5">
      {SCAN_PHASES.map((phase) => {
        const progress = phaseProgress[phase];
        const isCurrent = phase === currentPhase;
        const isDone = progress === 100;
        return (
          <div key={phase} className="space-y-1">
            <div className="flex items-center justify-between text-[11px]">
              <span className={cn("font-medium", isCurrent ? "text-foreground" : "text-muted-foreground")}>
                {isDone ? "✓ " : isCurrent ? "● " : ""}{PHASE_LABELS[phase]}
              </span>
              <span className="font-mono text-muted-foreground">{progress}%</span>
            </div>
            <div className="progress-bar">
              <div className={cn("progress-fill", phaseColors[phase])} style={{ width: `${progress}%` }} />
            </div>
          </div>
        );
      })}
    </div>
  );
}
