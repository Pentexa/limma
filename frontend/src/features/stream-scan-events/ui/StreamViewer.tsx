"use client";

import { useStreamStore } from "../model/stream-store";
import { cn } from "@/shared/lib/utils";

export function StreamViewer() {
  const events = useStreamStore((s) => s.events);

  return (
    <div className="flex flex-col h-full bg-[#0a0a0c]">
      <div className="flex-1 overflow-y-auto overflow-x-hidden p-0 font-mono text-xs custom-scrollbar">
        {events.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full gap-2 py-4">
            <div className="flex items-center gap-1.5">
              <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground/20" />
              <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground/15" />
              <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground/10" />
            </div>
            <span className="text-[11px] text-muted-foreground/40 font-mono">Awaiting scan events…</span>
            <span className="text-[9px] text-muted-foreground/20 font-mono">Start a scan to begin streaming</span>
          </div>
        ) : (
          events.slice(0, 100).map((event, idx) => {
            const data = event.data as any;
            let level = (data?.level ?? event.type ?? "info").toLowerCase();
            const message = data?.message ?? (typeof data === "string" ? data : JSON.stringify(data)) ?? "Event";

            // Enhanced Content-Based Color Parsing
            const msgLower = message.toLowerCase();
            if (level === "info" || level === "finding") {
              if (msgLower.includes("critical") || msgLower.includes("vulnerable") || msgLower.includes("sql") || msgLower.includes("xss") || msgLower.includes("rce") || msgLower.includes("injection")) {
                level = "critical";
              } else if (msgLower.includes("high") || msgLower.includes("error") || msgLower.includes("failed") || msgLower.includes("timeout")) {
                level = "error";
              } else if (msgLower.includes("warning") || msgLower.includes("medium") || msgLower.includes("skip")) {
                level = "warning";
              } else if (msgLower.includes("success") || msgLower.includes("found") || msgLower.includes("verified") || msgLower.includes("ok")) {
                level = "success";
              }
            }

            const levelConfig: Record<string, { color: string; bg: string; glow: string }> = {
              critical: { color: "hsl(0 85% 62%)", bg: "hsl(0 60% 15% / 0.15)", glow: "0 0 8px hsl(0 72% 51% / 0.2)" },
              error:    { color: "hsl(0 85% 62%)", bg: "hsl(0 60% 15% / 0.15)", glow: "0 0 8px hsl(0 72% 51% / 0.2)" },
              warning:  { color: "hsl(38 95% 60%)", bg: "hsl(38 60% 15% / 0.1)", glow: "none" },
              success:  { color: "hsl(142 65% 52%)", bg: "hsl(142 40% 15% / 0.1)", glow: "none" },
              complete: { color: "hsl(142 65% 52%)", bg: "hsl(142 40% 15% / 0.1)", glow: "none" },
              info:     { color: "hsl(207 80% 62%)", bg: "transparent", glow: "none" },
            };
            const cfg = levelConfig[level] ?? { color: "hsl(215 15% 50%)", bg: "transparent", glow: "none" };

            return (
              <div
                key={event.id}
                className="flex items-start px-3 py-1.5 transition-colors duration-100 hover:!bg-white/[0.04] group"
                style={{
                  background: cfg.bg,
                  boxShadow: cfg.glow,
                }}
              >
                {/* Line number */}
                <span className="select-none tabular-nums text-right pr-3 text-muted-foreground/30 min-w-[32px] text-[10px] mt-0.5">
                  {idx + 1}
                </span>
                
                {/* Timestamp */}
                <span className="tabular-nums shrink-0 text-muted-foreground/60 text-[10px] mr-3 mt-0.5">
                  {new Date(event.timestamp).toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit" })}
                </span>
                
                {/* Level badge */}
                <span
                  className="uppercase font-bold tracking-wider shrink-0 mr-3 mt-0.5"
                  style={{
                    color: cfg.color,
                    fontSize: "9px",
                    textShadow: level === "critical" || level === "error" ? `0 0 6px ${cfg.color}` : "none",
                  }}
                >
                  {level === "critical" ? "CRIT" : level.length > 5 ? level.slice(0, 5) : level}
                </span>
                
                {/* Message */}
                <span
                  className={cn(
                    "min-w-0 text-[11px] font-medium leading-relaxed break-words",
                    level === "critical" || level === "error"
                      ? "text-red-400"
                      : level === "success" || level === "complete"
                        ? "text-emerald-400"
                        : level === "warning"
                          ? "text-amber-400"
                          : "text-blue-300"
                  )}
                >
                  {message}
                </span>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
