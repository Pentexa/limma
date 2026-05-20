"use client";

import { cn } from "@/shared/lib/utils";
import { Badge } from "@/shared/ui/badge";
import { getSeverityBgColor, formatSeverity } from "@/entities/finding/lib/severity-utils";
import type { Finding } from "@/entities/finding/model/types";
import { DETECTOR_META } from "@/entities/finding/model/types";

interface FindingCardProps { finding: Finding; onClick?: () => void; }

export function FindingCard({ finding, onClick }: FindingCardProps) {
  return (
    <div className="flex items-center gap-3 px-3 py-2.5 rounded-md hover:bg-muted/30 cursor-pointer transition-colors" onClick={onClick}>
      <Badge className={cn("text-[10px] shrink-0", getSeverityBgColor(finding.severity))}>{formatSeverity(finding.severity)}</Badge>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium truncate">{finding.title}</p>
        <p className="text-xs text-muted-foreground truncate">{DETECTOR_META[finding.detector]?.name}</p>
      </div>
      <span className={cn("text-xs font-medium shrink-0", finding.verification === "verified" ? "text-verified" : "text-unverified")}>
        {finding.verification === "verified" ? "✓" : "○"}
      </span>
    </div>
  );
}
