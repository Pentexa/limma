"use client";

import { AlertTriangle, RefreshCcw, Home } from "lucide-react";
import Link from "next/link";

/**
 * Next.js root error boundary page.
 * P1-006: Catches unhandled errors at the route level, preventing white screens.
 * This works alongside the React ErrorBoundary in AppShell for component-level errors.
 */
export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div className="flex items-center justify-center min-h-screen bg-background">
      <div className="flex flex-col items-center text-center max-w-md px-6">
        <div className="flex h-14 w-14 items-center justify-center rounded-xl bg-risk/10 mb-6">
          <AlertTriangle className="h-7 w-7 text-risk" />
        </div>

        <h1 className="text-[18px] font-bold text-foreground mb-2">
          Something went wrong
        </h1>
        <p className="text-[12px] text-muted-foreground mb-1 leading-relaxed">
          An unexpected error occurred. This has been logged for investigation.
        </p>
        {error.digest && (
          <p className="text-[10px] font-mono text-muted-foreground/50 mb-6">
            Error ID: {error.digest}
          </p>
        )}

        <div className="flex items-center gap-3">
          <button
            onClick={reset}
            className="flex items-center gap-2 px-4 py-2.5 bg-primary/10 hover:bg-primary/20 text-primary rounded-lg text-[12px] font-semibold transition-colors"
          >
            <RefreshCcw className="h-3.5 w-3.5" />
            Try Again
          </button>
          <Link
            href="/"
            className="flex items-center gap-2 px-4 py-2.5 bg-muted/50 hover:bg-muted text-foreground rounded-lg text-[12px] font-semibold transition-colors border border-border"
          >
            <Home className="h-3.5 w-3.5" />
            Go Home
          </Link>
        </div>
      </div>
    </div>
  );
}
