"use client";

import { cn } from "@/shared/lib/utils";
import { AlertTriangle, RefreshCw } from "lucide-react";

interface ErrorPanelProps {
  message?: string;
  onRetry?: () => void;
  className?: string;
}

export function ErrorPanel({ message = "Failed to load data", onRetry, className }: ErrorPanelProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center py-12 text-center", className)}>
      <AlertTriangle className="h-8 w-8 text-risk/50 mb-3" />
      <p className="text-[12px] text-muted-foreground mb-3">{message}</p>
      {onRetry && (
        <button
          className="flex items-center gap-1.5 px-3 py-1.5 rounded text-[10px] font-semibold bg-muted/30 text-muted-foreground hover:bg-muted/50 hover:text-foreground transition-colors"
          onClick={onRetry}
        >
          <RefreshCw className="h-3 w-3" /> Retry
        </button>
      )}
    </div>
  );
}
