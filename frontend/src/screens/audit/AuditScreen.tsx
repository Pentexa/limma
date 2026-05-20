"use client";

import { useMemo } from "react";
import { FindingTable } from "@/entities/finding/ui/FindingTable";
import { FindingFilters } from "@/features/filter-findings/ui/FindingFilters";
import { useGlobalFindings } from "@/entities/finding/model/use-findings";
import { Loader2, ShieldAlert, AlertTriangle, CheckCircle, FileWarning } from "lucide-react";
import { cn } from "@/shared/lib/utils";

export function AuditScreen() {
  const { data: findings = [], isLoading } = useGlobalFindings();

  const stats = useMemo(() => {
    const criticalCount = findings.filter(f => f.severity === "critical").length;
    const highCount = findings.filter(f => f.severity === "high").length;
    const verifiedCount = findings.filter(f => f.verification === "verified").length;
    return { criticalCount, highCount, verifiedCount, total: findings.length };
  }, [findings]);

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-4 py-2.5 border-b border-border/40 bg-[#050505]">
        <div className="flex items-center gap-1.5">
          <ShieldAlert className="h-4 w-4 text-primary" />
          <h2 className="text-[13px] font-bold tracking-tight text-foreground">Security Audit</h2>
        </div>
        {isLoading && (
          <div className="flex items-center gap-1.5 text-[10px] text-primary animate-pulse font-mono">
            <Loader2 className="h-3 w-3 animate-spin" /> Loading findings…
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-4 bg-[#030303] space-y-4">
        {/* Stats HUD */}
        <div className="flex flex-wrap items-center gap-3 p-3 bg-[#080808] rounded-md border border-border/30 shadow-inner">
          <div className="flex items-center gap-2">
            <FileWarning className="h-4 w-4 text-primary" />
            <span className="text-[11px] font-bold text-foreground uppercase tracking-widest">Findings Overview</span>
          </div>
          <div className="flex flex-wrap items-center gap-3 ml-auto">
            {stats.criticalCount > 0 && (
              <div className="flex items-baseline gap-1.5 bg-critical/10 border border-critical/20 px-2.5 py-1 rounded shadow-sm">
                <AlertTriangle className="h-3 w-3 text-critical self-center" />
                <span className="text-[15px] font-mono font-bold text-critical">{stats.criticalCount}</span>
                <span className="text-[9px] uppercase tracking-widest text-critical/80 font-bold">Critical</span>
              </div>
            )}
            {stats.highCount > 0 && (
              <div className="flex items-baseline gap-1.5 bg-high/10 border border-high/20 px-2.5 py-1 rounded shadow-sm">
                <span className="text-[15px] font-mono font-bold text-high">{stats.highCount}</span>
                <span className="text-[9px] uppercase tracking-widest text-high/80 font-bold">High</span>
              </div>
            )}
            <div className="flex items-baseline gap-1.5 bg-verified/10 border border-verified/20 px-2.5 py-1 rounded shadow-sm">
              <CheckCircle className="h-3 w-3 text-verified self-center" />
              <span className="text-[15px] font-mono font-bold text-verified">{stats.verifiedCount}</span>
              <span className="text-[9px] uppercase tracking-widest text-verified/80 font-bold">Verified</span>
            </div>
            <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded">
              <span className="text-[15px] font-mono font-bold text-primary">{stats.total}</span>
              <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">Total</span>
            </div>
          </div>
        </div>

        {/* Filter bar */}
        <FindingFilters />

        {/* Findings table */}
        <div className="bg-[#080808] border border-border/30 rounded-md shadow-lg overflow-hidden">
          <FindingTable findings={findings} />
        </div>
      </div>
    </div>
  );
}
