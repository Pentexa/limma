"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import type { ApiCorrelationReport } from "@/shared/types/api";
import { Activity, Shield, ChevronDown } from "lucide-react";

interface RiskCorrelationPanelProps {
  correlation: ApiCorrelationReport | undefined;
}

export function RiskCorrelationPanel({ correlation }: RiskCorrelationPanelProps) {
  const [expandedIdx, setExpandedIdx] = useState<number | null>(null);

  if (!correlation) {
    return <div className="text-[11px] text-muted-foreground/50 py-8 text-center">No correlation data</div>;
  }

  const score = correlation.overall_risk_score;
  const scoreColor = score >= 80 ? "text-verified" : score >= 50 ? "text-attention" : "text-risk";

  return (
    <div className="w-full">
      {/* HUD Score Gauge */}
      <div className="flex flex-col md:flex-row items-center gap-6 p-4 bg-[#080808] rounded-md border border-border/30 shadow-inner">
        <div className="relative w-24 h-24 flex items-center justify-center shrink-0">
          {/* Outer rotating dashed ring */}
          <svg className="absolute inset-0 w-full h-full animate-[spin_12s_linear_infinite] opacity-30" viewBox="0 0 100 100">
            <circle cx="50" cy="50" r="46" fill="none" stroke="currentColor" strokeWidth="1" strokeDasharray="4 8" className="text-primary" />
          </svg>
          {/* Inner static track */}
          <svg className="absolute inset-0 w-full h-full transform -rotate-90" viewBox="0 0 100 100">
            <circle cx="50" cy="50" r="38" fill="none" stroke="currentColor" strokeWidth="4" className="text-muted/20" />
            <circle 
              cx="50" cy="50" r="38" fill="none" stroke="currentColor" strokeWidth="4" strokeLinecap="round"
              className={cn("transition-all duration-1000", scoreColor)}
              strokeDasharray={`${(score / 100) * (2 * Math.PI * 38)} ${2 * Math.PI * 38}`}
              style={{ filter: `drop-shadow(0 0 6px currentColor)` }}
            />
          </svg>
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <span className={cn("text-[24px] font-bold leading-none font-mono drop-shadow-[0_0_8px_currentColor]", scoreColor)}>{score}</span>
            <span className="text-[9px] text-muted-foreground/60 uppercase tracking-widest mt-1 font-bold">Risk</span>
          </div>
        </div>
        <div className="flex-1 text-center md:text-left">
          <h3 className="text-[13px] font-bold text-foreground mb-2 uppercase tracking-widest flex items-center justify-center md:justify-start gap-2">
            <Activity className="h-4 w-4 text-primary" /> Threat Intelligence
          </h3>
          <p className="text-[11px] text-muted-foreground/80 leading-relaxed md:border-l-2 md:border-primary/40 md:pl-3">
            {score >= 80 ? "System integrity is high. Minimal correlated attack vectors detected." :
             score >= 50 ? "Multiple vulnerabilities identified. Correlation indicates moderate exploit potential." :
             "Critical threat chain detected. Immediate remediation required to break attack paths."}
          </p>
        </div>
      </div>

      {/* Attack Path Chain */}
      {correlation.correlated_risks?.length > 0 && (
        <div className="mt-6">
          <div className="flex items-center gap-2 mb-4 px-1">
            <Shield className="h-4 w-4 text-primary" />
            <span className="text-[11px] font-bold text-foreground uppercase tracking-widest">
              Identified Attack Path ({correlation.correlated_risks.length} Nodes)
            </span>
          </div>
          
          <div className="relative pl-6 space-y-3">
            {/* Vertical timeline laser */}
            <div className="absolute top-4 bottom-4 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/50 to-transparent" />
            
            {correlation.correlated_risks.map((risk, i) => {
              const isExpanded = expandedIdx === i;
              const isCrit = risk.severity === "critical";
              const isHigh = risk.severity === "high";
              const nodeColor = isCrit ? "bg-critical shadow-[0_0_10px_var(--critical)]" : isHigh ? "bg-high shadow-[0_0_10px_var(--high)]" : "bg-attention shadow-[0_0_10px_var(--attention)]";

              return (
                <div key={i} className="relative group">
                  {/* Timeline Node */}
                  <div className={cn("absolute -left-5 top-4 w-3.5 h-3.5 rounded-full border-2 border-background z-10 transition-transform duration-300", nodeColor, isExpanded && "scale-125")} />
                  
                  {/* Threat Card */}
                  <div className={cn("bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2", isExpanded ? "border-primary/40" : "border-border/20 hover:border-border/50")}>
                    <div 
                      className="p-3.5 cursor-pointer flex items-center justify-between gap-3 hover:bg-white/[0.02]"
                      onClick={() => setExpandedIdx(isExpanded ? null : i)}
                    >
                      <div className="flex items-center gap-3">
                        <span className="font-mono text-[10px] text-muted-foreground/40 font-bold">0{i + 1}</span>
                        <span className={cn("text-[12px] font-bold transition-colors", isExpanded ? "text-primary" : "text-foreground/90")}>{risk.title}</span>
                      </div>
                      <div className="flex items-center gap-3 shrink-0">
                        <span className={cn("text-[9px] px-2 py-0.5 rounded font-bold uppercase tracking-wider", 
                          isCrit ? "bg-critical/10 text-critical border border-critical/20" :
                          isHigh ? "bg-high/10 text-high border border-high/20" : 
                          "bg-attention/10 text-attention border border-attention/20"
                        )}>
                          {risk.severity}
                        </span>
                        <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-300", isExpanded && "rotate-180")} />
                      </div>
                    </div>
                    
                    {/* Expandable Details */}
                    <div className={cn("grid transition-all duration-300 ease-in-out", isExpanded ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")}>
                      <div className="overflow-hidden">
                        <div className="px-4 pb-4 pt-2 border-t border-border/10 bg-black/40 space-y-3">
                          <p className="text-[11px] text-muted-foreground/80 leading-relaxed font-mono">{risk.explanation}</p>
                          {risk.evidences?.length > 0 && (
                            <div className="flex flex-wrap gap-2 pt-2 border-t border-border/10">
                              <span className="w-full text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest mt-1">Evidence Vectors</span>
                              {risk.evidences.map((ev, j) => (
                                <span key={j} className="text-[10px] font-mono bg-[#1a1a1a] border border-border/20 rounded px-2 py-1 text-muted-foreground/90 break-all shadow-inner">
                                  {ev}
                                </span>
                              ))}
                            </div>
                          )}
                        </div>
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
  );
}
