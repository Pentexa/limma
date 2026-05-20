"use client";

import { cn } from "@/shared/lib/utils";
import { FileQuestion } from "lucide-react";
import { type ReactNode } from "react";

interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  description?: string;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({ icon, title, description, action, className }: EmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center gap-3 py-12 px-6 text-center", className)}>
      <div className="flex h-10 w-10 items-center justify-center rounded bg-muted text-muted-foreground">
        {icon ?? <FileQuestion className="h-5 w-5" />}
      </div>
      <div className="space-y-1">
        <h3 className="text-[13px] font-semibold text-foreground">{title}</h3>
        {description && <p className="text-[11px] text-muted-foreground max-w-xs">{description}</p>}
      </div>
      {action && <div className="mt-1">{action}</div>}
    </div>
  );
}
