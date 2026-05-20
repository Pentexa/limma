"use client";

import { PhaseProgress } from "./PhaseProgress";
import { formatScanStatus, getScanStatusColor } from "@/entities/scan/lib/format-scan-status";
import type { Scan } from "@/entities/scan/model/types";
import { cn } from "@/shared/lib/utils";

export function ScannerControlPanel({ scan }: { scan: Scan | null }) {
  if (!scan) return <p className="text-[11px] text-muted-foreground py-4 text-center">No active scan</p>;

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between text-[11px]">
        <span className="text-muted-foreground">Status</span>
        <span className={cn("font-medium", getScanStatusColor(scan.status))}>{formatScanStatus(scan.status)}</span>
      </div>
      <PhaseProgress phaseProgress={scan.phaseProgress} currentPhase={scan.currentPhase} />
    </div>
  );
}
