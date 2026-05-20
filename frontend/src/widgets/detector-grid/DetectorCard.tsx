"use client";

import { cn } from "@/shared/lib/utils";
import { type DetectorInfo } from "@/entities/finding/model/types";
import { Shield, AlertTriangle, CheckCircle, Loader2 } from "lucide-react";

interface DetectorCardProps {
  detector: DetectorInfo;
  /** Real signal count from findings data — replaces hardcoded mock data */
  signals?: number;
  onClick?: (detector: DetectorInfo) => void;
}

const statusIcons = { idle: Shield, running: Loader2, completed: CheckCircle, error: AlertTriangle };

export function DetectorCard({ detector, signals = 0, onClick }: DetectorCardProps) {
  const Icon = statusIcons[detector.status];

  return (
    <div
      className={cn(
        "panel cursor-pointer p-0 transition-colors hover:bg-muted/20",
        detector.status === "running" && "border-primary/30",
        detector.status === "error" && "border-critical/30",
        detector.status === "completed" && "border-verified/15"
      )}
      onClick={() => onClick?.(detector)}
    >
      {/* Header row */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border/50">
        <Icon className={cn(
          "h-3 w-3 shrink-0",
          detector.status === "idle" && "text-muted-foreground/40",
          detector.status === "running" && "text-primary animate-spin",
          detector.status === "completed" && "text-verified",
          detector.status === "error" && "text-critical"
        )} />
        <h4 className="text-[11px] font-semibold text-foreground truncate flex-1">{detector.name}</h4>
        {detector.findingCount > 0 && (
          <span className="text-[9px] font-bold sev-badge sev-badge-medium">{detector.findingCount}</span>
        )}
      </div>
      {/* Body */}
      <div className="px-3 py-1.5 space-y-1">
        <p className="text-[10px] text-muted-foreground line-clamp-1">{detector.description}</p>
        <div className="flex items-center justify-between text-[9px]">
          <span className="text-muted-foreground/50 uppercase tracking-wider font-medium">{detector.category}</span>
          <span className="text-muted-foreground/40 font-mono tabular-nums">{signals} signals</span>
        </div>
        <div className="flex items-center justify-between text-[9px] pt-0.5">
          <span className={cn(
            "font-medium capitalize flex items-center gap-1",
            detector.status === "running" && "text-primary",
            detector.status === "completed" && "status-verified",
            detector.status === "error" && "sev-critical",
            detector.status === "idle" && "text-muted-foreground/40"
          )}>
            <span className={cn(
              "h-1 w-1 rounded-full",
              detector.status === "running" && "bg-primary animate-pulse",
              detector.status === "completed" && "bg-verified",
              detector.status === "error" && "bg-critical",
              detector.status === "idle" && "bg-muted-foreground/20"
            )} />
            {detector.status}
          </span>
          <span className="text-muted-foreground/30">
            {detector.lastRun ? new Date(detector.lastRun).toLocaleTimeString("en-US", { hour12: false }) : "—"}
          </span>
        </div>
      </div>
    </div>
  );
}
