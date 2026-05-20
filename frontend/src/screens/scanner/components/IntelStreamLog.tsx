"use client";

import { useEffect, useRef, useState } from "react";
import { cn } from "@/shared/lib/utils";
import { API_BASE_URL } from "@/shared/config/constants";
import { Radio, ChevronDown } from "lucide-react";

interface IntelStreamLogProps {
  streamPath: string;
  targetUrl: string | null;
  isActive: boolean;
}

interface StreamEvent {
  id: number;
  event_type: string;
  data: Record<string, unknown>;
  timestamp: number;
}

export function IntelStreamLog({ streamPath, targetUrl, isActive }: IntelStreamLogProps) {
  const [events, setEvents] = useState<StreamEvent[]>([]);
  const [connected, setConnected] = useState(false);
  const [isExpanded, setIsExpanded] = useState(true);
  const containerRef = useRef<HTMLDivElement>(null);
  const eventId = useRef(0);

  useEffect(() => {
    if (!isActive || !targetUrl) return;

    const url = `${API_BASE_URL}${streamPath}?url=${encodeURIComponent(targetUrl)}`;
    const source = new EventSource(url);

    source.onopen = () => setConnected(true);
    source.onerror = () => setConnected(false);

    source.onmessage = (e) => {
      try {
        const data = JSON.parse(e.data);
        setEvents((prev) => [
          ...prev.slice(-200), // Keep last 200
          { id: eventId.current++, event_type: data.event_type ?? "message", data, timestamp: Date.now() },
        ]);
      } catch {
        // skip non-JSON
      }
    };

    // Also listen to named events
    const eventTypes = ["phase_start", "phase_complete", "finding", "progress", "error", "complete"];
    for (const type of eventTypes) {
      source.addEventListener(type, (e: MessageEvent) => {
        try {
          const data = JSON.parse(e.data);
          setEvents((prev) => [
            ...prev.slice(-200),
            { id: eventId.current++, event_type: type, data, timestamp: Date.now() },
          ]);
        } catch {
          // skip
        }
      });
    }

    return () => {
      source.close();
      setConnected(false);
    };
  }, [streamPath, targetUrl, isActive]);

  // Auto-scroll
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [events.length]);

  const TYPE_COLORS: Record<string, string> = {
    finding: "text-risk",
    error: "text-risk",
    phase_start: "text-primary",
    phase_complete: "text-verified",
    progress: "text-foreground",
    complete: "text-verified",
  };

  return (
    <div className="relative pl-6">
      {/* Vertical timeline connector */}
      <div className="absolute top-3 bottom-3 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/40 to-transparent" />

      <div className="relative group">
        {/* Timeline Node */}
        <div className={cn(
          "absolute -left-5 top-4 w-3 h-3 rounded-full border-2 border-background z-10 transition-transform duration-300",
          connected ? "bg-primary shadow-[0_0_8px_var(--primary)] animate-pulse" : "bg-muted-foreground shadow-[0_0_4px_rgba(100,100,100,0.3)]",
          isExpanded && "scale-125"
        )} />

        {/* Stream Card */}
        <div className={cn(
          "bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2",
          isExpanded ? "border-primary/40" : "border-border/20 hover:border-border/50"
        )}>
          <div
            className="p-3.5 cursor-pointer flex items-center justify-between gap-3 hover:bg-white/[0.02]"
            onClick={() => setIsExpanded(!isExpanded)}
          >
            <div className="flex items-center gap-3">
              <Radio className={cn("h-4 w-4", connected ? "text-primary drop-shadow-[0_0_4px_var(--primary)]" : "text-muted-foreground/60")} />
              <span className={cn("text-[12px] font-bold uppercase tracking-wider transition-colors", isExpanded ? "text-primary" : "text-foreground/90")}>
                Live Stream
              </span>
              <span className={cn("text-[9px] font-mono px-1.5 py-0.5 rounded border",
                connected ? "bg-primary/10 text-primary border-primary/20" : "bg-muted/10 text-muted-foreground/50 border-border/10"
              )}>
                {connected ? "● Connected" : "○ Disconnected"}
              </span>
              {events.length > 0 && (
                <span className="text-[9px] font-mono text-muted-foreground/50 bg-[#111] border border-border/10 px-1.5 py-0.5 rounded">
                  {events.length}
                </span>
              )}
            </div>
            <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-300", isExpanded && "rotate-180")} />
          </div>

          {/* Expandable Content */}
          <div className={cn("grid transition-all duration-300 ease-in-out", isExpanded ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")}>
            <div className="overflow-hidden">
              <div ref={containerRef} className="max-h-[200px] overflow-y-auto font-mono text-[10px] leading-relaxed px-4 pb-4 pt-2 border-t border-border/10 bg-black/40">
                {events.length === 0 ? (
                  <div className="text-muted-foreground/30 text-center py-4">Waiting for events…</div>
                ) : (
                  events.map((evt) => (
                    <div key={evt.id} className="flex items-start gap-2 py-0.5">
                      <span className="text-muted-foreground/30 shrink-0 w-[52px] text-right tabular-nums">
                        {new Date(evt.timestamp).toLocaleTimeString("en", { hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit" })}
                      </span>
                      <span className={cn("shrink-0 w-[80px] uppercase text-[9px] font-bold tracking-wider", TYPE_COLORS[evt.event_type] ?? "text-muted-foreground")}>
                        {evt.event_type}
                      </span>
                      <span className="text-foreground/70 truncate">
                        {typeof evt.data === "object" ? JSON.stringify(evt.data).slice(0, 120) : String(evt.data)}
                      </span>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
