"use client";

import { cn } from "@/shared/lib/utils";
import { type ReactNode } from "react";
import { ErrorBoundary } from "@/shared/ui/error-boundary";

interface AppShellProps {
  sidebar: ReactNode;
  topbar: ReactNode;
  children: ReactNode;
  bottomPanel?: ReactNode;
  className?: string;
}

export function AppShell({ sidebar, topbar, children, bottomPanel, className }: AppShellProps) {
  return (
    <div className={cn("h-screen w-full overflow-hidden flex bg-[#0a0a0c] text-foreground selection:bg-primary/20", className)}>
      <aside className="shrink-0">{sidebar}</aside>
      <div className="flex-1 min-w-0 flex flex-col overflow-hidden">
        <header className="shrink-0 border-b border-white/[0.06] bg-[#0a0a0c]">{topbar}</header>
        <main className="flex-1 min-h-0 overflow-y-auto overflow-x-hidden">
          <ErrorBoundary>
            {children}
          </ErrorBoundary>
        </main>
        {bottomPanel && (
          <div className="shrink-0 border-t border-white/[0.06] bg-[#0a0a0c]">{bottomPanel}</div>
        )}
      </div>
    </div>
  );
}
