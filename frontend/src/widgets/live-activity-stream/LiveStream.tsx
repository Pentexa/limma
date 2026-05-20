"use client";

import { StreamViewer } from "@/features/stream-scan-events/ui/StreamViewer";
import { useStreamStore } from "@/features/stream-scan-events/model/stream-store";
import { cn } from "@/shared/lib/utils";
import { ChevronDown, ChevronUp } from "lucide-react";
import { useState, useRef, useEffect } from "react";

export function LiveStream({ className }: { className?: string }) {
  const [expanded, setExpanded] = useState(false);
  const [height, setHeight] = useState(140);
  
  const totalEvents = useStreamStore((s) => s.totalEvents);
  const connectionStatus = useStreamStore((s) => s.connectionStatus);
  const isConnected = connectionStatus === "connected";

  // Drag state
  const isDragging = useRef(false);
  const hasDragged = useRef(false);
  const startY = useRef(0);
  const startHeight = useRef(0);
  const isExpandedRef = useRef(expanded);
  isExpandedRef.current = expanded;

  const handlePointerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return; // Only left click
    
    isDragging.current = true;
    hasDragged.current = false;
    startY.current = e.clientY;
    startHeight.current = isExpandedRef.current ? height : 0;
    
    document.body.style.userSelect = "none";
    document.body.style.cursor = "ns-resize";

    const handlePointerMove = (ev: PointerEvent) => {
      if (!isDragging.current) return;
      
      const deltaY = ev.clientY - startY.current;
      
      if (Math.abs(deltaY) > 3) {
        hasDragged.current = true;
      }
      
      if (!hasDragged.current) return;

      let newHeight = startHeight.current - deltaY;
      const maxHeight = window.innerHeight * 0.8; // Max 80vh
      
      if (newHeight > maxHeight) newHeight = maxHeight;

      if (newHeight >= 60) {
        setHeight(newHeight);
        if (!isExpandedRef.current) setExpanded(true);
      } else {
        if (isExpandedRef.current) setExpanded(false);
      }
    };

    const cleanup = () => {
      isDragging.current = false;
      document.body.style.userSelect = "";
      document.body.style.cursor = "";
      document.removeEventListener("pointermove", handlePointerMove);
      document.removeEventListener("pointerup", cleanup);
      
      // Reset to a default height if closed so clicking opens it nicely next time
      setHeight(h => h < 60 ? 140 : h);
    };

    document.addEventListener("pointermove", handlePointerMove);
    document.addEventListener("pointerup", cleanup);
  };

  const handleClick = () => {
    if (!hasDragged.current) {
      if (!expanded && height < 60) setHeight(140);
      setExpanded(!expanded);
    }
  };

  return (
    <div className={cn("bg-[#050505] shadow-[0_-4px_20px_rgba(0,0,0,0.5)] flex flex-col transition-all duration-200", expanded ? "border-t-2 border-t-primary/50" : "border-t border-white/[0.06]", className)} style={{ transitionProperty: isDragging.current ? 'none' : 'all' }}>
      <button
        className="flex items-center justify-between px-4 py-1.5 bg-gradient-to-r from-primary/10 to-transparent hover:from-primary/20 border-b border-border/30 cursor-ns-resize"
        onPointerDown={handlePointerDown}
        onClick={handleClick}
      >
        <span className="text-[11px] font-bold tracking-wide text-foreground flex items-center gap-1.5">
          {isConnected ? (
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-60" />
              <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-400 shadow-[0_0_6px_hsl(142_60%_50%/0.5)]" />
            </span>
          ) : (
            <span className="h-2 w-2 rounded-full bg-muted-foreground/30" />
          )}
          <span>Event Stream</span>
          {isConnected && (
            <span className="text-[8px] font-mono font-bold tracking-widest text-emerald-400/80 uppercase ml-1">Live</span>
          )}
        </span>
        <span className="flex items-center gap-2">
          <span className="text-[9px] font-mono text-muted-foreground/40 tabular-nums">{totalEvents} events</span>
          {expanded ? <ChevronDown className="h-2.5 w-2.5" /> : <ChevronUp className="h-2.5 w-2.5" />}
        </span>
      </button>
      {expanded && (
        <div 
          className="flex flex-col relative custom-scrollbar overflow-hidden"
          style={{ height: `${height}px`, minHeight: '60px' }}
        >
          <div className="absolute inset-0 overflow-y-auto">
            <StreamViewer />
          </div>
        </div>
      )}
    </div>
  );
}
