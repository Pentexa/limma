"use client";

import { cn } from "@/shared/lib/utils";
import type { Severity } from "@/shared/types/common";

export function StatusBadge({ severity, count }: { severity: Severity; count: number }) {
  return (
    <span className={cn("sev-badge", `sev-badge-${severity}`)}>
      {count} {severity}
    </span>
  );
}
