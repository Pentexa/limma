"use client";

import { cn } from "@/shared/lib/utils";

interface SkeletonPanelProps {
  lines?: number;
  className?: string;
}

/** Deterministic widths to avoid impure Math.random in render */
const SKELETON_WIDTHS = [72, 88, 64, 80, 68, 92, 76, 84];

export function SkeletonPanel({ lines = 4, className }: SkeletonPanelProps) {
  return (
    <div className={cn("panel animate-pulse", className)}>
      <div className="panel-header">
        <div className="h-3 w-24 bg-muted/40 rounded" />
      </div>
      <div className="panel-body space-y-2.5">
        {Array.from({ length: lines }).map((_, i) => (
          <div key={i} className="flex items-center gap-3">
            <div className="h-2.5 bg-muted/30 rounded" style={{ width: `${SKELETON_WIDTHS[i % SKELETON_WIDTHS.length]}%` }} />
          </div>
        ))}
      </div>
    </div>
  );
}
