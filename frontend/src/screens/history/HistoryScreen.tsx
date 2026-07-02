"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import { useScanHistory } from "@/entities/scan/model/use-scan-history";
import { useScanTrends } from "@/entities/scan/model/use-scan-trends";
import { useScans } from "@/entities/scan/model/use-scans";
import { deleteScan, deleteHistoryScan } from "@/entities/scan/api/scan-api";
import { httpClient } from "@/shared/api/http-client";
import { TrendChart } from "@/screens/analysis/components/TrendChart";
import type { ApiDeltaResult } from "@/shared/types/api";
import { History, Loader2, XCircle, Trash2, GitCompare, Play } from "lucide-react";
import { toast } from "sonner";
import { EmptyState } from "@/shared/ui/empty-state";
import { LoadingSpinner } from "@/shared/ui/loading-spinner";
import { StatusBadge } from "@/shared/ui/badges";
import { useRouter } from "next/navigation";
import { useQueryClient } from "@tanstack/react-query";
import { ConfirmationDialog } from "@/shared/ui/confirmation-dialog";

export function HistoryScreen() {
  const { data: historyScans = [], isLoading } = useScanHistory();
  const { data: activeScans = [] } = useScans();
  const targetUrl = activeScans[0]?.targetUrl;
  const { data: trends = [] } = useScanTrends(targetUrl);
  const [delta, setDelta] = useState<ApiDeltaResult | null>(null);
  const [, setDeltaLoading] = useState(false);
  const [selectedScans, setSelectedScans] = useState<Set<string>>(new Set());
  const [isBulkDeleting, setIsBulkDeleting] = useState(false);
  const [scanPendingDelete, setScanPendingDelete] = useState<string | null>(null);
  const [isDeletingScan, setIsDeletingScan] = useState(false);
  const [isBulkDeleteDialogOpen, setIsBulkDeleteDialogOpen] = useState(false);
  const router = useRouter();
  const queryClient = useQueryClient();

  const allScans = [...activeScans, ...historyScans].reduce((acc, s) => {
    if (!acc.find(x => x.id === s.id)) acc.push(s);
    return acc;
  }, [] as typeof activeScans);

  const sortedScans = [...allScans].sort((a, b) =>
    new Date(b.createdAt || 0).getTime() - new Date(a.createdAt || 0).getTime()
  );

  async function handleDelete(scanId: string) {
    setIsDeletingScan(true);
    try {
      // Force delete from both to ensure it's gone regardless of where it lives
      try {
        await deleteHistoryScan(scanId);
      } catch {
        // Ignore errors if it wasn't in history
      }
      try {
        await deleteScan(scanId);
      } catch {
        // Ignore errors if it wasn't in active scans
      }
      
      await queryClient.invalidateQueries();
      router.refresh();
      setScanPendingDelete(null);
      toast.success("Scan deleted successfully");
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Delete scan failed");
    } finally {
      setIsDeletingScan(false);
    }
  }

  async function handleBulkDelete() {
    if (selectedScans.size === 0) return;
    setIsBulkDeleting(true);
    let successCount = 0;
    try {
      for (const scanId of Array.from(selectedScans)) {
        // Send delete to both endpoints since a scan could be in either (or both).
        // The backend returns 200 OK even if 0 rows are affected, so we don't rely on try/catch to switch endpoints.
        await Promise.allSettled([
          deleteHistoryScan(scanId),
          deleteScan(scanId)
        ]);
        successCount++;
      }
      
      await queryClient.invalidateQueries();
      router.refresh();
      setSelectedScans(new Set());
      setIsBulkDeleteDialogOpen(false);
      
      if (successCount > 0) {
        toast.success(`Successfully deleted ${successCount} scans`);
      }
    } catch {
      toast.error("Bulk delete encountered an error");
    } finally {
      setIsBulkDeleting(false);
    }
  }

  async function handleDelta(currentId: string, previousId: string, targetUrl: string) {
    setDeltaLoading(true);
    try {
      const result = await httpClient.get<ApiDeltaResult>("/api/history/delta", {
        params: { 
          target_url: targetUrl,
          current_scan_id: currentId, 
          previous_scan_id: previousId 
        },
      });
      setDelta(result);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Delta comparison failed");
    } finally {
      setDeltaLoading(false);
    }
  }

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden">
      <div className="shrink-0 flex items-center justify-between gap-4 px-4 py-2.5 border-b border-border bg-card/50">
        <div className="flex items-center gap-1.5">
          <History className="h-4 w-4 text-primary" />
          <h2 className="text-[13px] font-bold tracking-tight">Scan History</h2>
          <span className="text-[10px] text-muted-foreground ml-2">{sortedScans.length} scans</span>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Trend chart */}
        {trends.length >= 2 && (
          <div className="max-w-6xl mx-auto panel border-white/[0.06]">
            <div className="panel-header border-white/[0.06]"><span className="panel-title">Security Score Over Time</span></div>
            <div className="panel-body"><TrendChart points={trends} /></div>
          </div>
        )}

        <div className="max-w-6xl mx-auto">
          {/* Metrics HUD */}
          {sortedScans.length > 0 && (
            <div className="flex flex-wrap items-center gap-3 px-1 mb-6 mt-4">
              {[
                { label: "Total Scans", value: sortedScans.length, color: "text-foreground" },
                { label: "Running", value: sortedScans.filter(s => s.status?.toLowerCase() === "running").length, color: "text-primary" },
                { label: "Completed", value: sortedScans.filter(s => s.status?.toLowerCase() === "completed").length, color: "text-verified" },
                { label: "Failed", value: sortedScans.filter(s => s.status?.toLowerCase() === "failed").length, color: "text-risk" },
              ].map(m => (
                <div key={m.label} className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-3 py-1.5 rounded shadow-sm">
                  <span className={cn("text-[15px] font-mono font-bold", m.color)}>{m.value}</span>
                  <span className="text-[9px] uppercase tracking-widest text-muted-foreground/60 font-bold">{m.label}</span>
                </div>
              ))}
              
              {/* Select All Toggle */}
              <button
                onClick={() => {
                  if (selectedScans.size === sortedScans.length) {
                    setSelectedScans(new Set());
                  } else {
                    setSelectedScans(new Set(sortedScans.map(s => s.id)));
                  }
                }}
                className="ml-auto flex items-center gap-2 px-3 py-1.5 rounded bg-transparent hover:bg-white/[0.03] text-[10px] uppercase tracking-wider font-semibold text-muted-foreground transition-colors border border-transparent hover:border-white/[0.05]"
              >
                {selectedScans.size === sortedScans.length ? "Deselect All" : "Select All"}
              </button>
            </div>
          )}



          {/* Scan list */}
          {isLoading ? (
            <LoadingSpinner message="Loading history…" />
          ) : sortedScans.length === 0 ? (
            <EmptyState
              icon={<History className="h-5 w-5" />}
              title="No scan history"
              description="You haven't run any scans yet. Go to the Scanner module to start a new scan."
              action={
                <button
                  type="button"
                  onClick={() => router.push("/scanner")}
                  className="flex items-center gap-2 px-3 py-1.5 bg-primary/20 text-primary hover:bg-primary/30 border border-primary/30 rounded text-[11px] font-semibold transition-colors mt-2"
                >
                  <Play className="h-3 w-3" />
                  Start a Scan
                </button>
              }
            />
          ) : (
            <div className="relative pl-6 space-y-2">
              {/* Vertical timeline */}
              <div className="absolute top-3 bottom-3 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/40 to-transparent" />

              {sortedScans.map((scan) => {
                const status = scan.status?.toLowerCase() || "";
                const isRunning = status === "running";
                const isCompleted = status === "completed";
                const isFailed = status === "failed";
                const isIdle = status === "idle";
                
                return (
                  <div key={scan.id} className="relative group">
                    {/* Timeline Node */}
                    <div className={cn(
                      "absolute -left-5 top-4 w-3 h-3 rounded-full border-2 border-background z-10 transition-transform duration-300",
                      isCompleted ? "bg-verified shadow-[0_0_8px_var(--verified)]" :
                      isFailed ? "bg-risk shadow-[0_0_8px_var(--risk)]" :
                      isIdle ? "bg-muted-foreground shadow-[0_0_8px_var(--muted-foreground)]" :
                      "bg-primary shadow-[0_0_8px_var(--primary)]",
                      isRunning && "animate-pulse"
                    )} />

                    {/* Scan Card */}
                    <div 
                      onClick={() => router.push(`/scanner?scanId=${scan.id}`)}
                      className={cn(
                        "bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2 border-border/20 hover:border-border/50 cursor-pointer",
                        selectedScans.has(scan.id) && "border-primary/50 bg-primary/[0.02]"
                      )}>
                      <div className="p-3.5 flex items-center justify-between gap-3 hover:bg-white/[0.02]">
                        <div className="flex items-center gap-3 min-w-0">
                          <div 
                            className="relative flex items-center justify-center shrink-0"
                            onClick={(e) => e.stopPropagation()}
                          >
                            <input
                              type="checkbox"
                              className="peer absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
                              checked={selectedScans.has(scan.id)}
                              onChange={(e) => {
                                const newSet = new Set(selectedScans);
                                if (e.target.checked) {
                                  newSet.add(scan.id);
                                } else {
                                  newSet.delete(scan.id);
                                }
                                setSelectedScans(newSet);
                              }}
                            />
                            <div className="w-4 h-4 rounded border border-white/20 bg-black/40 peer-checked:bg-primary peer-checked:border-primary flex items-center justify-center transition-all duration-200">
                              <svg 
                                className={cn(
                                  "w-2.5 h-2.5 text-black pointer-events-none transition-transform duration-200", 
                                  selectedScans.has(scan.id) ? "scale-100" : "scale-0"
                                )} 
                                viewBox="0 0 14 14" fill="none"
                              >
                                <path d="M3 8L6 11L11 3.5" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                              </svg>
                            </div>
                          </div>
                          <span className={cn("text-[9px] font-mono font-bold uppercase px-2 py-0.5 rounded border shrink-0 w-[80px] text-center", 
                            isCompleted ? "bg-verified/10 text-verified border-verified/20" :
                            isFailed ? "bg-risk/10 text-risk border-risk/20" :
                            isIdle ? "bg-muted/10 text-muted-foreground border-border/20" :
                            "bg-primary/10 text-primary border-primary/20"
                          )}>
                            {scan.status}
                          </span>
                          <div className="flex flex-col">
                            <span className={cn("text-[12px] font-mono font-bold truncate transition-colors text-foreground/90 max-w-[250px] md:max-w-[350px]")}>
                              {scan.targetUrl}
                            </span>
                            <span className="text-[10px] text-muted-foreground/40 font-mono mt-0.5">
                              ID: {scan.id?.slice(0, 8) || "—"} • Phase: {scan.currentPhase}
                            </span>
                          </div>
                        </div>
                        
                        <div className="flex items-center gap-4 shrink-0">
                          <div className="flex items-center gap-3 text-[10px] text-muted-foreground/60 mr-4">
                            <span className="flex items-center gap-1">
                              <span className={cn("font-bold text-[11px]", (scan.result?.criticalCount ?? 0) > 0 ? "text-risk" : "")}>{scan.result?.criticalCount ?? 0}</span> Crit
                            </span>
                            <span className="flex items-center gap-1">
                              <span className={cn("font-bold text-[11px]", (scan.result?.highCount ?? 0) > 0 ? "text-attention" : "")}>{scan.result?.highCount ?? 0}</span> High
                            </span>
                            <span>·</span>
                            <span className="font-bold text-[11px] text-foreground/70">{scan.result?.totalFindings ?? 0}</span> Total
                          </div>
                          
                          <div className="text-right mr-4 w-[100px]">
                            <p className="text-[9px] text-muted-foreground/40 font-medium uppercase tracking-wider">
                              {scan.duration > 0 ? `${Math.round(scan.duration / 1000)}s` : "—"}
                            </p>
                            <p className="text-[10px] font-mono text-muted-foreground/70 truncate">
                              {scan.startedAt ? new Date(scan.startedAt).toLocaleString(undefined, {
                                month: "short", day: "numeric", hour: "2-digit", minute: "2-digit"
                              }) : "—"}
                            </p>
                          </div>
                          
                          <div className="flex items-center gap-2">
                            {sortedScans.length >= 2 && sortedScans[0]?.id !== scan.id && (
                              <button
                                className="flex items-center justify-center h-8 w-8 rounded-md bg-white/[0.02] border border-white/[0.05] hover:bg-primary/10 hover:text-primary hover:border-primary/20 text-muted-foreground transition-colors"
                                title="Compare with latest"
                                onClick={(e) => { e.preventDefault(); e.stopPropagation(); handleDelta(sortedScans[0].id, scan.id, scan.targetUrl); }}
                              >
                                <GitCompare className="h-4 w-4" />
                              </button>
                            )}
                            <button
                              className="flex items-center justify-center h-8 w-8 rounded-md bg-white/[0.02] border border-white/[0.05] hover:bg-risk/10 hover:text-risk hover:border-risk/20 text-muted-foreground transition-colors"
                              title="Delete scan"
                              onClick={(e) => { e.preventDefault(); e.stopPropagation(); setScanPendingDelete(scan.id); }}
                            >
                              <Trash2 className="h-4 w-4" />
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {/* Delta Compare Modal */}
      {delta && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4" onClick={() => setDelta(null)}>
          <div 
            className="bg-[#0a0a0c] border border-white/[0.06] rounded-xl shadow-2xl w-full max-w-4xl max-h-[85vh] flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between px-4 py-3 border-b border-white/[0.06] shrink-0">
              <div className="flex items-center gap-2">
                <GitCompare className="h-4 w-4 text-primary" />
                <h3 className="text-[13px] font-bold text-foreground">Scan Comparison</h3>
              </div>
              <button 
                onClick={() => setDelta(null)}
                className="text-muted-foreground/50 hover:text-foreground transition-colors"
              >
                <XCircle className="h-4 w-4" />
              </button>
            </div>
            
            <div className="p-4 space-y-6 overflow-y-auto min-h-0">
              {/* Summary Cards */}
              <div className="grid grid-cols-3 gap-3">
                <div className="bg-risk/5 border border-risk/10 p-3 rounded-lg text-center">
                  <div className="text-[20px] font-mono font-bold text-risk mb-1">
                    +{delta.new_findings?.length || 0}
                  </div>
                  <div className="text-[10px] uppercase tracking-wider font-bold text-risk/70">New Findings</div>
                </div>
                
                <div className="bg-verified/5 border border-verified/10 p-3 rounded-lg text-center">
                  <div className="text-[20px] font-mono font-bold text-verified mb-1">
                    -{delta.resolved_findings?.length || 0}
                  </div>
                  <div className="text-[10px] uppercase tracking-wider font-bold text-verified/70">Resolved</div>
                </div>
                
                <div className="bg-attention/5 border border-attention/10 p-3 rounded-lg text-center">
                  <div className="text-[20px] font-mono font-bold text-attention mb-1">
                    +{delta.new_endpoints?.length || 0}
                  </div>
                  <div className="text-[10px] uppercase tracking-wider font-bold text-attention/70">New Endpoints</div>
                </div>
              </div>

              {/* New Findings Table */}
              <div className="space-y-2">
                <h4 className="text-[11px] font-bold uppercase tracking-widest text-risk">New Findings</h4>
                <div className="border border-white/[0.06] rounded-md overflow-hidden bg-black/20">
                  {(delta.new_findings?.length || 0) > 0 ? (
                    <div className="overflow-x-auto">
                      <table className="w-full text-left text-[11px] whitespace-nowrap">
                        <thead className="bg-white/[0.02] border-b border-white/[0.06] text-muted-foreground/60 font-mono">
                          <tr>
                            <th className="px-3 py-2 font-medium">Severity</th>
                            <th className="px-3 py-2 font-medium">Name</th>
                            <th className="px-3 py-2 font-medium w-full">URL</th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-white/[0.06]">
                          {delta.new_findings.map((f, i) => (
                            <tr key={i} className="hover:bg-white/[0.02] transition-colors">
                              <td className="px-3 py-2"><StatusBadge status={f.severity} /></td>
                              <td className="px-3 py-2 font-medium text-foreground/80">{f.name}</td>
                              <td className="px-3 py-2 font-mono text-muted-foreground/50 truncate max-w-[300px]">{f.url}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  ) : (
                    <div className="px-3 py-4 text-[11px] text-muted-foreground/40 text-center font-mono">No new findings detected.</div>
                  )}
                </div>
              </div>

              {/* Resolved Findings Table */}
              <div className="space-y-2">
                <h4 className="text-[11px] font-bold uppercase tracking-widest text-verified">Resolved Findings</h4>
                <div className="border border-white/[0.06] rounded-md overflow-hidden bg-black/20">
                  {(delta.resolved_findings?.length || 0) > 0 ? (
                    <div className="overflow-x-auto">
                      <table className="w-full text-left text-[11px] whitespace-nowrap">
                        <thead className="bg-white/[0.02] border-b border-white/[0.06] text-muted-foreground/60 font-mono">
                          <tr>
                            <th className="px-3 py-2 font-medium">Severity</th>
                            <th className="px-3 py-2 font-medium">Name</th>
                            <th className="px-3 py-2 font-medium w-full">URL</th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-white/[0.06]">
                          {delta.resolved_findings.map((f, i) => (
                            <tr key={i} className="hover:bg-white/[0.02] transition-colors">
                              <td className="px-3 py-2"><StatusBadge status={f.severity} /></td>
                              <td className="px-3 py-2 font-medium text-foreground/80 line-through opacity-50">{f.name}</td>
                              <td className="px-3 py-2 font-mono text-muted-foreground/50 truncate max-w-[300px]">{f.url}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  ) : (
                    <div className="px-3 py-4 text-[11px] text-muted-foreground/40 text-center font-mono">No resolved findings.</div>
                  )}
                </div>
              </div>

              {/* New Endpoints Table */}
              <div className="space-y-2">
                <h4 className="text-[11px] font-bold uppercase tracking-widest text-attention">New Endpoints</h4>
                <div className="border border-white/[0.06] rounded-md overflow-hidden bg-black/20">
                  {(delta.new_endpoints?.length || 0) > 0 ? (
                    <div className="overflow-x-auto">
                      <table className="w-full text-left text-[11px] whitespace-nowrap">
                        <thead className="bg-white/[0.02] border-b border-white/[0.06] text-muted-foreground/60 font-mono">
                          <tr>
                            <th className="px-3 py-2 font-medium w-24">Method</th>
                            <th className="px-3 py-2 font-medium w-full">Endpoint URL</th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-white/[0.06]">
                          {delta.new_endpoints.map((ep, i) => (
                            <tr key={i} className="hover:bg-white/[0.02] transition-colors">
                              <td className="px-3 py-2">
                                <span className={cn("px-2 py-0.5 rounded text-[9px] font-bold border",
                                  ep.method === "GET" ? "text-verified bg-verified/10 border-verified/20" :
                                  ep.method === "POST" ? "text-attention bg-attention/10 border-attention/20" :
                                  "text-primary bg-primary/10 border-primary/20"
                                )}>
                                  {ep.method}
                                </span>
                              </td>
                              <td className="px-3 py-2 font-mono text-foreground/80">{ep.url}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  ) : (
                    <div className="px-3 py-4 text-[11px] text-muted-foreground/40 text-center font-mono">No new endpoints discovered.</div>
                  )}
                </div>
              </div>
              
            </div>
            
            <div className="px-4 py-3 border-t border-white/[0.06] bg-white/[0.01] shrink-0 text-[10px] text-muted-foreground/40 text-center uppercase tracking-wider font-bold">
              Comparison generated between selected scan and its predecessor.
            </div>
          </div>
        </div>
      )}

      {/* Modern Floating Bulk Action Bar */}
      {selectedScans.size > 0 && (
        <div className="fixed bottom-8 left-1/2 -translate-x-1/2 z-50 animate-in slide-in-from-bottom-10 fade-in duration-300">
          <div className="flex items-center gap-4 px-4 py-3 rounded-full bg-black/80 backdrop-blur-xl border border-white/[0.1] shadow-[0_8px_32px_rgba(0,0,0,0.4)]">
            <div className="flex items-center gap-2">
              <div className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/20 text-primary text-[11px] font-bold">
                {selectedScans.size}
              </div>
              <span className="text-[12px] font-medium text-white/90">
                scan{selectedScans.size !== 1 ? 's' : ''} selected
              </span>
            </div>
            
            <div className="w-[1px] h-4 bg-white/[0.15]" />
            
            <div className="flex items-center gap-2">
              <button
                onClick={() => setSelectedScans(new Set())}
                className="px-3 py-1.5 rounded-full text-[11px] font-semibold text-white/60 hover:text-white hover:bg-white/[0.05] transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => setIsBulkDeleteDialogOpen(true)}
                disabled={isBulkDeleting}
                className="flex items-center gap-1.5 px-4 py-1.5 rounded-full bg-risk/20 text-risk border border-risk/30 hover:bg-risk/30 text-[11px] font-semibold transition-colors disabled:opacity-50 shadow-[0_0_15px_rgba(239,68,68,0.15)]"
              >
                {isBulkDeleting ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <Trash2 className="h-3 w-3" />
                )}
                Delete
              </button>
            </div>
          </div>
        </div>
      )}
      <ConfirmationDialog
        open={scanPendingDelete !== null}
        onOpenChange={(open) => !open && setScanPendingDelete(null)}
        title="Delete scan record?"
        description="This scan will be removed from active scans and history. This action cannot be undone."
        confirmLabel="Delete Scan"
        destructive
        isPending={isDeletingScan}
        onConfirm={() => scanPendingDelete && handleDelete(scanPendingDelete)}
      />
      <ConfirmationDialog
        open={isBulkDeleteDialogOpen}
        onOpenChange={setIsBulkDeleteDialogOpen}
        title={`Delete ${selectedScans.size} scan records?`}
        description="All selected scan records will be permanently removed. This action cannot be undone."
        confirmLabel="Delete Selected"
        destructive
        isPending={isBulkDeleting}
        onConfirm={handleBulkDelete}
      />
    </div>
  );
}
