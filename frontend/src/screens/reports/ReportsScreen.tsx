"use client";

import { cn } from "@/shared/lib/utils";
import { useReports, reportKeys } from "@/entities/report/model/use-reports";
import { saveLocalReport } from "@/entities/report/api/report-api";
import { useScans } from "@/entities/scan/model/use-scans";
import { httpClient } from "@/shared/api/http-client";
import { FileText, Loader2, Download } from "lucide-react";
import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { Report } from "@/entities/report/model/types";
import type { ApiMasterReport } from "@/shared/types/api";

export function ReportsScreen() {
  const { data: reports = [], isLoading } = useReports();
  const { data: scans = [] } = useScans();
  const activeScan = scans.find(s => s.status === "running") ?? scans[0];

  const queryClient = useQueryClient();



  const FORMAT_BADGE: Record<string, { cls: string }> = {
    pdf: { cls: "bg-risk/10 text-risk border-risk/20" },
    html: { cls: "bg-primary/10 text-primary border-primary/20" },
    json: { cls: "bg-analysis/10 text-analysis border-analysis/20" },
  };

  const metrics = {
    total: reports.length,
    criticals: reports.reduce((sum, r) => sum + (r.criticalCount || 0), 0)
  };

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden bg-[#0a0a0c]">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-5 py-3 border-b border-white/[0.06]">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center h-7 w-7 rounded-lg bg-primary/10 border border-primary/20">
            <FileText className="h-3.5 w-3.5 text-primary" />
          </div>
          <div>
            <h2 className="text-sm font-semibold text-foreground">Reports</h2>
            {activeScan?.targetUrl ? (
              <p className="text-[11px] font-mono text-muted-foreground/60 truncate max-w-[300px]">
                {activeScan.targetUrl}
              </p>
            ) : (
              <p className="text-[11px] font-mono text-muted-foreground/60">
                {isLoading ? "Loading reports…" : `${reports.length} generated reports`}
              </p>
            )}
          </div>
        </div>
        {/* Export actions */}

      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-6xl mx-auto h-full">
          {isLoading ? (
            <div className="flex items-center justify-center h-48">
              <div className="flex items-center gap-2 text-primary animate-pulse">
                <Loader2 className="h-5 w-5 animate-spin" />
                <span className="text-[13px] font-medium">Loading reports…</span>
              </div>
            </div>
          ) : reports.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-[300px] text-muted-foreground/30 text-center">
              <div className="h-16 w-16 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-4">
                <FileText className="h-7 w-7 opacity-50" />
              </div>
              <span className="text-[14px] font-medium text-muted-foreground/50">No reports generated yet</span>
              <span className="text-[12px] mt-1 text-muted-foreground/30">Complete a scan to generate security reports.</span>
            </div>
          ) : (
            <div className="space-y-4">
              {/* Metrics HUD */}
              <div className="flex flex-wrap items-center gap-3 px-1 mb-6">
                {[
                  { label: "Total Reports", value: metrics.total, color: "text-foreground" },
                  { label: "Total Criticals", value: metrics.criticals, color: "text-risk" },
                ].map(m => (
                  <div key={m.label} className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-3 py-1.5 rounded shadow-sm">
                    <span className={cn("text-[15px] font-mono font-bold", m.color)}>{m.value}</span>
                    <span className="text-[9px] uppercase tracking-widest text-muted-foreground/60 font-bold">{m.label}</span>
                  </div>
                ))}
              </div>

              {/* Reports Timeline List */}
              <div className="relative pl-6 space-y-2 mt-4">
                {/* Vertical timeline */}
                <div className="absolute top-3 bottom-3 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/40 to-transparent" />

                {reports.map((r) => {
                  const formatStyle = FORMAT_BADGE[r.format]?.cls ?? "bg-muted/10 text-muted-foreground border-border/20";
                  
                  return (
                    <div key={r.id} className="relative group">
                      {/* Timeline Node */}
                      <div className={cn(
                        "absolute -left-5 top-4 w-3 h-3 rounded-full border-2 border-background z-10 transition-transform duration-300",
                        r.status === "completed" ? "bg-verified shadow-[0_0_8px_var(--verified)]" :
                        r.status === "failed" ? "bg-risk shadow-[0_0_8px_var(--risk)]" :
                        "bg-primary shadow-[0_0_8px_var(--primary)]",
                      )} />

                      {/* Report Card */}
                      <div className={cn(
                        "bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2 border-border/20 hover:border-border/50",
                      )}>
                        <div className="p-3.5 flex items-center justify-between gap-3 hover:bg-white/[0.02]">
                          <div className="flex items-center gap-3 min-w-0">
                            <span className={cn("text-[9px] font-mono font-bold uppercase px-2 py-0.5 rounded border shrink-0", formatStyle)}>
                              {r.format}
                            </span>
                            <div className="flex flex-col">
                              <span className={cn("text-[12px] font-mono font-bold truncate transition-colors text-foreground/90")}>
                                {r.title}
                              </span>
                              <span className="text-[10px] text-muted-foreground/40 font-mono mt-0.5">
                                Scan ID: {r.scanId?.slice(0, 8)}
                              </span>
                            </div>
                          </div>
                          
                          <div className="flex items-center gap-4 shrink-0">
                            <div className="flex items-center gap-3 text-[10px] text-muted-foreground/60 mr-4">
                              <span className="flex items-center gap-1">
                                <span className={cn("font-bold text-[11px]", r.criticalCount > 0 ? "text-risk" : "")}>{r.criticalCount}</span> Crit
                              </span>
                              <span>·</span>
                              <span className="font-bold text-[11px] text-foreground/70">{r.findingCount}</span> Total
                            </div>
                            
                            <div className="text-right mr-2">
                              <p className="text-[9px] text-muted-foreground/40 font-medium uppercase tracking-wider">Generated</p>
                              <p className="text-[10px] font-mono text-muted-foreground/70">
                                {r.createdAt ? new Date(r.createdAt).toLocaleString(undefined, {
                                  month: "short", day: "numeric", hour: "2-digit", minute: "2-digit"
                                }) : "—"}
                              </p>
                            </div>
                            
                            {r.fileUrl ? (
                              <a 
                                href={r.fileUrl} 
                                target="_blank" 
                                rel="noopener noreferrer" 
                                className="flex items-center justify-center h-8 w-8 rounded-md bg-primary/10 text-primary border border-primary/20 hover:bg-primary/20 transition-colors"
                                title="Download Report"
                              >
                                <Download className="h-4 w-4" />
                              </a>
                            ) : (
                              <div className="h-8 w-8 rounded-md bg-white/[0.02] border border-white/[0.05] flex items-center justify-center opacity-50 cursor-not-allowed">
                                <Download className="h-4 w-4 text-muted-foreground" />
                              </div>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
