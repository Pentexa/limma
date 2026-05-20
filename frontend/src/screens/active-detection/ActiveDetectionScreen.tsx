"use client";

import { useState, useMemo } from "react";
import { cn } from "@/shared/lib/utils";
import { useGlobalFindings } from "@/entities/finding/model/use-findings";
import { DETECTOR_META } from "@/entities/finding/model/types";
import type { Finding, VerificationStatus } from "@/entities/finding/model/types";
import type { Severity } from "@/shared/types/common";
import { compareBySeverity } from "@/shared/config/priority";
import { SeverityFilterBar } from "./components/SeverityFilterBar";
import { FindingDetailPanel } from "./components/FindingDetailPanel";
import {
  Loader2, Zap, CheckCircle, Eye, AlertTriangle, ChevronRight, Shield
} from "lucide-react";

const ALL_SEVERITIES = new Set<Severity>(["critical", "high", "medium", "low", "info"]);

export function ActiveDetectionScreen() {
  const { data: findings = [], isLoading } = useGlobalFindings();

  /* ── Filter state ── */
  const [selectedSeverities, setSelectedSeverities] = useState<Set<Severity>>(new Set(ALL_SEVERITIES));
  const [selectedDetector, setSelectedDetector] = useState<string | null>(null);
  const [selectedStatus, setSelectedStatus] = useState<VerificationStatus | "all">("all");
  const [selectedFindingId, setSelectedFindingId] = useState<string | null>(null);

  /* ── Normalize ── */
  const normalized = useMemo(() =>
    findings.map(f => ({
      ...f,
      severity: (f.severity?.toLowerCase() ?? "info") as Severity,
      verification: (f.verification?.toLowerCase() ?? "unverified") as VerificationStatus,
    })),
    [findings]
  );

  /* ── Counts for filter sidebar ── */
  const counts = useMemo(() => {
    const c: Record<string, number> = {};
    for (const f of normalized) {
      c[f.severity] = (c[f.severity] ?? 0) + 1;
      c[f.verification] = (c[f.verification] ?? 0) + 1;
    }
    c["all"] = normalized.length;
    return c;
  }, [normalized]);

  /* ── Filtered list ── */
  const filtered = useMemo(() => {
    return normalized.filter(f => {
      if (!selectedSeverities.has(f.severity)) return false;
      if (selectedDetector && f.detector !== selectedDetector) return false;
      if (selectedStatus !== "all" && f.verification !== selectedStatus) return false;
      return true;
    }).sort(compareBySeverity);
  }, [normalized, selectedSeverities, selectedDetector, selectedStatus]);

  const toggleSeverity = (sev: Severity) => {
    setSelectedSeverities(prev => {
      const next = new Set(prev);
      if (next.has(sev)) { next.delete(sev); } else { next.add(sev); }
      if (next.size === 0) return new Set(ALL_SEVERITIES);
      return next;
    });
  };

  /* ── Stats ── */
  const criticalCount = normalized.filter(f => f.severity === "critical").length;
  const verifiedCount = normalized.filter(f => f.verification === "verified").length;
  const fpCount = normalized.filter(f => f.verification === "false_positive").length;
  const totalCount = normalized.length;

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden bg-[#0a0a0c]">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-4 py-2.5 border-b border-white/[0.06]">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center h-6 w-6 rounded-md bg-primary/10 border border-primary/20">
            <Zap className="h-3 w-3 text-primary" />
          </div>
          <div>
            <h2 className="text-[13px] font-semibold text-foreground">Active Detection</h2>
            <p className="text-[10px] text-muted-foreground/60">
              {isLoading ? "Loading findings…" : `${filtered.length} visible · ${totalCount} total`}
            </p>
          </div>
        </div>
        
        {isLoading ? (
          <Loader2 className="h-4 w-4 animate-spin text-primary" />
        ) : (
          <div className="flex items-center gap-2">
            {criticalCount > 0 && (
              <span className="text-[10px] font-medium text-muted-foreground/70 bg-red-500/[0.04] border border-red-500/10 rounded px-2 py-0.5">
                <span className="text-red-400 font-semibold">{criticalCount}</span> critical
              </span>
            )}
            <span className="text-[10px] font-medium text-muted-foreground/70 bg-emerald-500/[0.04] border border-emerald-500/10 rounded px-2 py-0.5">
              <span className="text-emerald-400 font-semibold">{verifiedCount}</span> verified
            </span>
            {fpCount > 0 && (
              <span className="text-[10px] font-medium text-muted-foreground/70 bg-amber-500/[0.04] border border-amber-500/10 rounded px-2 py-0.5">
                <span className="text-amber-400 font-semibold">{fpCount}</span> false pos
              </span>
            )}
          </div>
        )}
      </div>

      {/* Main content: 3-panel layout */}
      <div className="flex-1 flex min-h-0 overflow-hidden">
        {/* Left: Filter sidebar */}
        <div className="shrink-0 w-[200px] border-r border-white/[0.06] overflow-y-auto p-4">
          <SeverityFilterBar
            selectedSeverities={selectedSeverities}
            onToggleSeverity={toggleSeverity}
            selectedDetector={selectedDetector}
            onSelectDetector={setSelectedDetector}
            selectedStatus={selectedStatus}
            onSelectStatus={setSelectedStatus}
            counts={counts}
          />
        </div>

        {/* Center: Finding list */}
        <div className="flex-1 min-w-0 overflow-y-auto">
          {filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground/30 px-6 text-center">
              <div className="h-16 w-16 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-4">
                <AlertTriangle className="h-7 w-7 opacity-50" />
              </div>
              <span className="text-[14px] font-medium text-muted-foreground/50">{isLoading ? "Loading findings…" : "No findings match filters"}</span>
              {!isLoading && <span className="text-[12px] mt-1 text-muted-foreground/30">Adjust severity, detector, or status filters.</span>}
            </div>
          ) : (
            <div className="divide-y divide-white/[0.04]">
              {filtered.map((f) => {
                const isSelected = selectedFindingId === f.id;
                const detectorName = DETECTOR_META[f.detector]?.name ?? f.detector;
                return (
                  <div
                    key={f.id}
                    className={cn(
                      "px-3 py-2.5 cursor-pointer transition-all duration-150 group flex items-start justify-between gap-3",
                      isSelected
                        ? "bg-primary/[0.06] border-l-2 border-l-primary"
                        : "hover:bg-white/[0.02] border-l-2 border-l-transparent"
                    )}
                    onClick={() => setSelectedFindingId(f.id)}
                  >
                    <div className="flex-1 min-w-0 space-y-1">
                      <div className="flex items-center gap-2">
                        <span className={cn("sev-badge text-[9px] font-semibold", `sev-badge-${f.severity}`)}>{f.severity}</span>
                        {f.verification === "verified" && (
                          <span className="flex items-center gap-1 text-[9px] text-emerald-400 font-medium">
                            <CheckCircle className="h-2.5 w-2.5" /> Verified
                          </span>
                        )}
                        {f.verification === "false_positive" && (
                          <span className="flex items-center gap-1 text-[9px] text-amber-400 font-medium">
                            <Eye className="h-2.5 w-2.5" /> False Pos
                          </span>
                        )}
                      </div>
                      <p className={cn("text-[12px] font-medium truncate transition-colors",
                        isSelected ? "text-primary" : "text-foreground/90 group-hover:text-foreground"
                      )}>{f.title}</p>
                      <div className="flex items-center gap-2 text-[9px] text-muted-foreground/50 font-mono">
                        <span className="truncate max-w-[300px]">{f.url} → {f.parameter}</span>
                        <span>·</span>
                        <span className="text-muted-foreground/70">{detectorName}</span>
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-3 shrink-0 mt-0.5">
                      {f.cvss != null && (
                        <div className="text-right">
                          <p className="text-[8px] text-muted-foreground/40 font-medium uppercase">CVSS</p>
                          <p className={cn("text-[11px] font-semibold font-mono tabular-nums",
                            f.cvss >= 9 ? "text-red-400" : f.cvss >= 7 ? "text-orange-400" : "text-foreground/60"
                          )}>{f.cvss}</p>
                        </div>
                      )}
                      <div className="text-right">
                        <p className="text-[8px] text-muted-foreground/40 font-medium uppercase">Evid</p>
                        <p className={cn("text-[11px] font-semibold font-mono tabular-nums",
                          f.evidence.length >= 2 ? "text-primary/80" : "text-foreground/50"
                        )}>{f.evidence.length}</p>
                      </div>
                      <ChevronRight className={cn("h-3.5 w-3.5 shrink-0 transition-colors ml-1",
                        isSelected ? "text-primary" : "text-muted-foreground/15 group-hover:text-muted-foreground/40"
                      )} />
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Right: Detail panel */}
        {selectedFindingId && (
          <div className="shrink-0 w-[380px] border-l border-white/[0.06] overflow-hidden">
            <FindingDetailPanel
              finding={normalized.find(f => f.id === selectedFindingId) ?? null}
              onClose={() => setSelectedFindingId(null)}
            />
          </div>
        )}
      </div>
    </div>
  );
}
