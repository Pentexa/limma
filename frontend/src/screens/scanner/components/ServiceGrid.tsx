"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import type { ApiCollectorSnapshot, ApiVerifyPortResponse } from "@/shared/types/api";
import { httpClient } from "@/shared/api/http-client";
import { Server, Clock, AlertTriangle, RefreshCw, CheckCircle, XCircle, ChevronDown, Activity } from "lucide-react";

interface ServiceGridProps {
  collector: ApiCollectorSnapshot | undefined;
}

export function ServiceGrid({ collector }: ServiceGridProps) {
  const [verifyResults, setVerifyResults] = useState<Record<number, ApiVerifyPortResponse | null>>({});
  const [verifying, setVerifying] = useState<Record<number, boolean>>({});
  const [expandedPort, setExpandedPort] = useState<number | null>(null);

  async function handleVerifyPort(port: number) {
    if (!collector) return;
    setVerifying(prev => ({ ...prev, [port]: true }));
    try {
      const result = await httpClient.post<ApiVerifyPortResponse>("/verify-port", {
        host: collector.target_input.host,
        port,
      });
      setVerifyResults(prev => ({ ...prev, [port]: result }));
    } catch {
      setVerifyResults(prev => ({ ...prev, [port]: null }));
    } finally {
      setVerifying(prev => ({ ...prev, [port]: false }));
    }
  }

  if (!collector || !collector.port_results?.length) {
    return <div className="text-[11px] text-muted-foreground/50 py-8 text-center">No services discovered</div>;
  }

  return (
    <div className="space-y-4">
      {/* Target info HUD */}
      <div className="flex flex-wrap items-center gap-3 p-3 bg-[#080808] rounded-md border border-border/30 shadow-inner">
        <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded">
          <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">Target</span>
          <span className="font-mono text-[11px] text-foreground font-bold">{collector.target_input.normalized_url}</span>
        </div>
        {collector.resolved_target.primary_ip && (
          <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded">
            <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">IP</span>
            <span className="font-mono text-[11px] text-foreground font-bold">{collector.resolved_target.primary_ip}</span>
          </div>
        )}
        <span className={cn("text-[9px] font-bold uppercase tracking-widest px-2 py-0.5 rounded border",
          collector.overall_status === "completed" ? "bg-verified/10 text-verified border-verified/20" : "bg-attention/10 text-attention border-attention/20"
        )}>{collector.overall_status}</span>
        <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded ml-auto">
          <span className="text-[15px] font-mono font-bold text-primary">{collector.port_results.length}</span>
          <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">Ports</span>
        </div>
      </div>

      {/* Service port list as vertical chain */}
      <div className="relative pl-6 space-y-2">
        {/* Vertical timeline */}
        <div className="absolute top-3 bottom-3 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/40 to-transparent" />

        {collector.port_results.map((port) => {
          const topCandidate = port.service_candidates[0];
          const finalScore = topCandidate?.confidence_breakdown.final_score ?? 0;
          const isExpanded = expandedPort === port.port;

          return (
            <div key={port.port} className="relative group">
              {/* Timeline Node */}
              <div className={cn(
                "absolute -left-5 top-4 w-3 h-3 rounded-full border-2 border-background z-10 transition-transform duration-300",
                port.state === "open" ? "bg-verified shadow-[0_0_8px_var(--verified)]" : "bg-muted-foreground shadow-[0_0_4px_rgba(100,100,100,0.3)]",
                isExpanded && "scale-125"
              )} />

              {/* Service Card */}
              <div className={cn(
                "bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2",
                isExpanded ? "border-primary/40" : "border-border/20 hover:border-border/50"
              )}>
                <div
                  className="p-3.5 cursor-pointer flex items-center justify-between gap-3 hover:bg-white/[0.02]"
                  onClick={() => setExpandedPort(isExpanded ? null : port.port)}
                >
                  <div className="flex items-center gap-3">
                    <span className="font-mono text-[14px] font-bold text-primary">{port.port}</span>
                    <span className={cn("text-[9px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded border",
                      port.state === "open" ? "bg-verified/10 text-verified border-verified/20" : "bg-muted/10 text-muted-foreground border-border/20"
                    )}>{port.state}</span>
                    {topCandidate && (
                      <span className="text-[12px] font-bold text-foreground/90">{topCandidate.service_name}</span>
                    )}
                  </div>
                  <div className="flex items-center gap-3 shrink-0">
                    <span className="flex items-center gap-1 text-[9px] text-muted-foreground/40">
                      <Clock className="h-2.5 w-2.5" />{port.probe_duration_ms}ms
                    </span>
                    {topCandidate && (
                      <span className={cn("text-[10px] font-mono font-bold tabular-nums",
                        finalScore >= 80 ? "text-verified" : finalScore >= 50 ? "text-foreground" : "text-muted-foreground"
                      )}>{finalScore}%</span>
                    )}
                    <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-300", isExpanded && "rotate-180")} />
                  </div>
                </div>

                {/* Expandable Details */}
                <div className={cn("grid transition-all duration-300 ease-in-out", isExpanded ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")}>
                  <div className="overflow-hidden">
                    <div className="px-4 pb-4 pt-2 border-t border-border/10 bg-black/40 space-y-3">
                      {topCandidate && (
                        <>
                          <p className="text-[11px] text-muted-foreground/80 leading-relaxed font-mono">{topCandidate.reasoning}</p>
                          <div className="flex items-center gap-4 text-[10px]">
                            <span className={cn("font-bold uppercase tracking-wider text-[9px]",
                              topCandidate.decision === "confirmed" ? "text-verified" : "text-attention"
                            )}>{topCandidate.decision}</span>
                          </div>
                        </>
                      )}

                      {/* Evidence */}
                      {port.all_evidence.length > 0 && (
                        <div className="space-y-1.5">
                          <span className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Evidence ({port.all_evidence.length})</span>
                          <div className="flex flex-wrap gap-1.5">
                            {port.all_evidence.slice(0, 6).map((ev, j) => (
                              <span key={j} className="text-[9px] font-mono bg-[#1a1a1a] border border-border/20 rounded px-2 py-1 text-muted-foreground/80 shadow-inner break-all">
                                {typeof ev === "string" ? ev : JSON.stringify(ev)}
                              </span>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Verify button */}
                      <div className="flex items-center justify-between pt-2 border-t border-border/10">
                        {port.fallback_used && (
                          <span className="text-[9px] text-attention flex items-center gap-1"><AlertTriangle className="h-3 w-3" /> Fallback used</span>
                        )}
                        <button
                          className={cn(
                            "flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider px-3 py-1.5 rounded border transition-all duration-200",
                            verifyResults[port.port]?.is_active
                              ? "bg-verified/10 text-verified border-verified/20"
                              : verifyResults[port.port] !== undefined && verifyResults[port.port]?.is_active === false
                                ? "bg-risk/10 text-risk border-risk/20"
                                : "bg-primary/10 text-primary border-primary/20 hover:bg-primary/20"
                          )}
                          onClick={(e) => { e.stopPropagation(); handleVerifyPort(port.port); }}
                          disabled={verifying[port.port]}
                        >
                          {verifying[port.port] ? (
                            <><RefreshCw className="h-3 w-3 animate-spin" /> Verifying…</>
                          ) : verifyResults[port.port] ? (
                            verifyResults[port.port]?.is_active ? (
                              <><CheckCircle className="h-3 w-3" /> Active</>
                            ) : (
                              <><XCircle className="h-3 w-3" /> Inactive</>
                            )
                          ) : (
                            <><Activity className="h-3 w-3" /> Verify Port</>
                          )}
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* Errors */}
      {(collector.errors?.length ?? 0) > 0 && (
        <div className="space-y-1.5 px-1 mt-4">
          <span className="text-[9px] text-risk uppercase font-bold tracking-widest">Errors</span>
          {collector.errors.map((err, i) => (
            <div key={i} className="flex items-center gap-2 text-[10px] text-risk bg-risk/5 border border-risk/10 rounded px-3 py-1.5">
              <AlertTriangle className="h-3 w-3 shrink-0" /> {err}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
