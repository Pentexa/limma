"use client";

import { AlertTriangle, RefreshCcw } from "lucide-react";

/**
 * Dashboard route-group error boundary.
 * Catches errors within the (dashboard) layout without disrupting the entire app.
 */
export default function DashboardError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div className="flex flex-col items-center justify-center h-full min-h-[400px] p-8">
      <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-risk/10 mb-5">
        <AlertTriangle className="h-6 w-6 text-risk" />
      </div>

      <h2 className="text-[15px] font-bold text-foreground mb-2">
        Module Error
      </h2>
      <p className="text-[11px] text-muted-foreground mb-1 text-center max-w-sm leading-relaxed">
        {error.message || "An unexpected error occurred in this module."}
      </p>
      {error.digest && (
        <p className="text-[9px] font-mono text-muted-foreground/40 mb-5">
          {error.digest}
        </p>
      )}

      <button
        onClick={reset}
        className="flex items-center gap-2 px-4 py-2 bg-muted/50 hover:bg-muted border border-border rounded-lg text-[11px] font-semibold transition-colors"
      >
        <RefreshCcw className="h-3 w-3" />
        Retry
      </button>
    </div>
  );
}
