"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import type { ApiDiscoveryResult } from "@/shared/types/api";
import { CheckCircle, XCircle, Shield, ChevronDown } from "lucide-react";

interface EndpointTableProps {
  discovery: ApiDiscoveryResult | undefined;
}

const METHOD_STYLE: Record<string, string> = {
  GET: "bg-verified/10 text-verified border-verified/20",
  POST: "bg-attention/10 text-attention border-attention/20",
  PUT: "bg-analysis/10 text-analysis border-analysis/20",
  PATCH: "bg-primary/10 text-primary border-primary/20",
  DELETE: "bg-risk/10 text-risk border-risk/20",
};

export function EndpointTable({ discovery }: EndpointTableProps) {
  const [expandedIdx, setExpandedIdx] = useState<number | null>(null);

  if (!discovery || !discovery.detected_endpoints?.length) {
    return <div className="text-[11px] text-muted-foreground/50 py-8 text-center">No endpoints discovered</div>;
  }

  const endpoints = discovery.detected_endpoints;

  return (
    <div className="space-y-4">
      {/* Metrics HUD */}
      {discovery.metrics && (
        <div className="flex flex-wrap items-center gap-3 px-1">
          {[
            { label: "Total", value: discovery.metrics.total_endpoints, color: "text-foreground" },
            { label: "Valid", value: discovery.metrics.valid_endpoints, color: "text-verified" },
            { label: "False Pos", value: discovery.metrics.false_positives, color: "text-risk" },
            { label: "Precision", value: `${(discovery.metrics.precision * 100).toFixed(0)}%`, color: "text-primary" },
          ].map(m => (
            <div key={m.label} className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-3 py-1.5 rounded shadow-sm">
              <span className={cn("text-[15px] font-mono font-bold", m.color)}>{m.value}</span>
              <span className="text-[9px] uppercase tracking-widest text-muted-foreground/60 font-bold">{m.label}</span>
            </div>
          ))}
        </div>
      )}

      {/* Suspected technologies */}
      {discovery.suspected_api_technologies?.length > 0 && (
        <div className="flex items-center gap-2 px-1">
          <span className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">API Tech:</span>
          {discovery.suspected_api_technologies.map(t => (
            <span key={t} className="bg-primary/10 text-primary border border-primary/20 rounded px-2 py-0.5 text-[9px] font-bold uppercase tracking-wider">{t}</span>
          ))}
        </div>
      )}

      {/* Endpoint List */}
      <div className="relative pl-6 space-y-2">
        {/* Vertical timeline */}
        <div className="absolute top-3 bottom-3 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/40 to-transparent" />

        {endpoints.map((ep, i) => {
          const isExpanded = expandedIdx === i;
          const rv = ep.runtime_verification;
          const methodCls = METHOD_STYLE[ep.method_prediction] ?? "bg-muted/10 text-foreground border-border/20";

          return (
            <div key={i} className="relative group">
              {/* Timeline Node */}
              <div className={cn(
                "absolute -left-5 top-4 w-3 h-3 rounded-full border-2 border-background z-10 transition-transform duration-300",
                rv?.is_valid ? "bg-verified shadow-[0_0_8px_var(--verified)]" :
                rv?.is_valid === false ? "bg-risk shadow-[0_0_8px_var(--risk)]" :
                "bg-primary shadow-[0_0_8px_var(--primary)]",
                isExpanded && "scale-125"
              )} />

              {/* Endpoint Card */}
              <div className={cn(
                "bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2",
                isExpanded ? "border-primary/40" : "border-border/20 hover:border-border/50"
              )}>
                <div
                  className="p-3 cursor-pointer flex items-center justify-between gap-3 hover:bg-white/[0.02]"
                  onClick={() => setExpandedIdx(isExpanded ? null : i)}
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <span className={cn("text-[9px] font-mono font-bold uppercase px-2 py-0.5 rounded border shrink-0", methodCls)}>
                      {ep.method_prediction}
                    </span>
                    <span className={cn("text-[12px] font-mono font-bold truncate transition-colors", isExpanded ? "text-primary" : "text-foreground/90")}>
                      {ep.path}
                    </span>
                  </div>
                  <div className="flex items-center gap-3 shrink-0">
                    {ep.auth_probability > 0.7 && (
                      <span className="flex items-center gap-1 text-[9px] font-bold text-attention bg-attention/10 border border-attention/20 px-1.5 py-0.5 rounded">
                        <Shield className="h-2.5 w-2.5" /> AUTH
                      </span>
                    )}
                    <span className={cn("text-[10px] font-mono tabular-nums font-bold",
                      ep.confidence_score >= 0.8 ? "text-verified" : ep.confidence_score >= 0.5 ? "text-foreground" : "text-muted-foreground/50"
                    )}>{(ep.confidence_score * 100).toFixed(0)}%</span>
                    {rv && (
                      rv.is_valid
                        ? <CheckCircle className="h-3.5 w-3.5 text-verified" />
                        : <XCircle className="h-3.5 w-3.5 text-risk" />
                    )}
                    <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-300", isExpanded && "rotate-180")} />
                  </div>
                </div>

                {/* Expandable Details */}
                <div className={cn("grid transition-all duration-300 ease-in-out", isExpanded ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")}>
                  <div className="overflow-hidden">
                    <div className="px-4 pb-4 pt-2 border-t border-border/10 bg-black/40 space-y-3">
                      {/* Parameters */}
                      {ep.parameters.length > 0 && (
                        <div className="space-y-1.5">
                          <span className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Parameters</span>
                          <div className="flex flex-wrap gap-1.5">
                            {ep.parameters.map((p, j) => (
                              <span key={j} className="text-[10px] font-mono bg-[#1a1a1a] border border-border/20 rounded px-2 py-1 text-muted-foreground/90 shadow-inner">
                                <span className="text-primary/70">{p.name}</span>
                                {p.param_type && <span className="text-muted-foreground/40 ml-1">: {p.param_type}</span>}
                              </span>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Auth & Confidence detail */}
                      <div className="flex items-center gap-6 text-[10px]">
                        <div className="flex items-center gap-1.5">
                          <span className="text-muted-foreground/50">Auth Prob:</span>
                          <span className={cn("font-mono font-bold", ep.auth_probability > 0.7 ? "text-attention" : "text-muted-foreground")}>
                            {(ep.auth_probability * 100).toFixed(0)}%
                          </span>
                        </div>
                        <div className="flex items-center gap-1.5">
                          <span className="text-muted-foreground/50">Confidence:</span>
                          <span className={cn("font-mono font-bold", ep.confidence_score >= 0.8 ? "text-verified" : "text-foreground")}>
                            {(ep.confidence_score * 100).toFixed(0)}%
                          </span>
                        </div>
                        {rv && (
                          <div className="flex items-center gap-1.5">
                            <span className="text-muted-foreground/50">Verified:</span>
                            <span className={cn("font-bold uppercase tracking-wider text-[9px]", rv.is_valid ? "text-verified" : "text-risk")}>
                              {rv.is_valid ? "Valid" : "Invalid"}
                            </span>
                            {rv.status_code && <span className="font-mono text-muted-foreground/60">({rv.status_code})</span>}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
