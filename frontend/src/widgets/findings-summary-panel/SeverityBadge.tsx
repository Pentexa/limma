"use client";

import { Badge } from "@/shared/ui/badge";
import { cn } from "@/shared/lib/utils";
import { getSeverityBgColor, formatSeverity } from "@/entities/finding/lib/severity-utils";
import type { Severity } from "@/shared/types/common";

interface SeverityBadgeProps { severity: Severity; className?: string; }

export function SeverityBadge({ severity, className }: SeverityBadgeProps) {
  return (
    <Badge className={cn("text-[10px]", getSeverityBgColor(severity), className)}>
      {formatSeverity(severity)}
    </Badge>
  );
}
