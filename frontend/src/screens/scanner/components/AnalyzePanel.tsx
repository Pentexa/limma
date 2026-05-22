"use client";

import { cn } from "@/shared/lib/utils";
import type { ApiWebScanResult, ApiSecurityHeaderResult, ApiRiskInsight, ApiCorrelationReport } from "@/shared/types/api";
import { TechStackPanel } from "./TechStackPanel";
import { ShieldAlert, AlertTriangle, GitMerge, Check, X, Info } from "lucide-react";

interface AnalyzePanelProps {
  data: ApiWebScanResult;
}

export function AnalyzePanel({ data }: AnalyzePanelProps) {
  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-500">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="space-y-6">
          <SecurityHeadersPanel headers={data.security_headers || []} />
          <RiskInsightsPanel risks={data.risk_insights || []} />
        </div>
        <div className="space-y-6">
          <CorrelationPanel correlation={data.correlation} />
          <TechStackPanel technologies={data.detected_technologies || []} />
        </div>
      </div>
    </div>
  );
}

function SecurityHeadersPanel({ headers }: { headers: ApiSecurityHeaderResult[] }) {
  if (!headers?.length) return null;

  return (
    <div className="bg-[#080808] border border-border/20 rounded-xl overflow-hidden shadow-lg">
      <div className="p-4 border-b border-border/10 bg-white/[0.01] flex items-center gap-3">
        <ShieldAlert className="h-4 w-4 text-primary/80" />
        <h3 className="text-[13px] font-bold uppercase tracking-wider text-foreground/90">Security Headers</h3>
        <span className="ml-auto text-[10px] font-mono bg-white/[0.04] px-2 py-0.5 rounded-md text-muted-foreground">
          {headers.length} Checked
        </span>
      </div>
      <div className="divide-y divide-border/10 max-h-[300px] overflow-y-auto">
        {headers.map((header, idx) => (
          <div key={idx} className="p-4 hover:bg-white/[0.01] transition-colors">
            <div className="flex items-start justify-between gap-4">
              <div>
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-[12px] font-mono font-bold text-foreground">{header.name}</span>
                  {header.status === "present" ? (
                    <span className="flex items-center gap-1 text-[9px] uppercase font-bold text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded">
                      <Check className="h-3 w-3" /> Present
                    </span>
                  ) : header.status === "weak" || header.status === "misconfigured" ? (
                    <span className="flex items-center gap-1 text-[9px] uppercase font-bold text-yellow-400 bg-yellow-500/10 px-1.5 py-0.5 rounded">
                      <AlertTriangle className="h-3 w-3" /> {header.status}
                    </span>
                  ) : (
                    <span className="flex items-center gap-1 text-[9px] uppercase font-bold text-red-400 bg-red-500/10 px-1.5 py-0.5 rounded">
                      <X className="h-3 w-3" /> Missing
                    </span>
                  )}
                </div>
                {header.value && (
                  <div className="text-[10px] font-mono text-muted-foreground/60 break-all mb-2 bg-black/40 p-1.5 rounded">
                    {header.value}
                  </div>
                )}
                <p className="text-[11px] text-muted-foreground/80 leading-relaxed">{header.explanation}</p>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function RiskInsightsPanel({ risks }: { risks: ApiRiskInsight[] }) {
  if (!risks?.length) return null;

  return (
    <div className="bg-[#080808] border border-border/20 rounded-xl overflow-hidden shadow-lg">
      <div className="p-4 border-b border-border/10 bg-white/[0.01] flex items-center gap-3">
        <AlertTriangle className="h-4 w-4 text-orange-400/80" />
        <h3 className="text-[13px] font-bold uppercase tracking-wider text-foreground/90">Risk Insights</h3>
        <span className="ml-auto text-[10px] font-mono bg-white/[0.04] px-2 py-0.5 rounded-md text-muted-foreground">
          {risks.length} Insights
        </span>
      </div>
      <div className="divide-y divide-border/10 max-h-[400px] overflow-y-auto">
        {risks.map((risk, idx) => (
          <div key={idx} className="p-4 hover:bg-white/[0.01] transition-colors">
            <div className="flex items-center gap-2 mb-2">
              <span className={cn(
                "text-[9px] uppercase font-bold px-1.5 py-0.5 rounded",
                risk.severity.toLowerCase() === "high" ? "bg-red-500/10 text-red-400" :
                risk.severity.toLowerCase() === "medium" ? "bg-orange-500/10 text-orange-400" :
                risk.severity.toLowerCase() === "low" ? "bg-yellow-500/10 text-yellow-400" :
                "bg-blue-500/10 text-blue-400"
              )}>
                {risk.severity}
              </span>
              <span className="text-[12px] font-bold text-foreground">{risk.title}</span>
            </div>
            <p className="text-[11px] text-muted-foreground/80 leading-relaxed mb-2">{risk.explanation}</p>
            {risk.evidence && (
              <div className="flex items-start gap-2 bg-black/40 p-2 rounded border border-white/[0.02]">
                <Info className="h-3 w-3 text-primary/50 shrink-0 mt-0.5" />
                <span className="text-[10px] font-mono text-muted-foreground/60 break-all">{risk.evidence}</span>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function CorrelationPanel({ correlation }: { correlation?: ApiCorrelationReport }) {
  if (!correlation?.correlated_risks?.length) return null;

  return (
    <div className="bg-primary/5 border border-primary/20 rounded-xl overflow-hidden shadow-[0_0_15px_rgba(var(--primary-rgb),0.05)]">
      <div className="p-4 border-b border-primary/10 bg-primary/[0.02] flex items-center gap-3">
        <GitMerge className="h-4 w-4 text-primary" />
        <h3 className="text-[13px] font-bold uppercase tracking-wider text-primary">Correlated Findings</h3>
        <div className="ml-auto flex items-center gap-2">
          <span className="text-[10px] uppercase font-bold text-primary/50 tracking-wider">Overall Risk</span>
          <span className="text-[12px] font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">
            {correlation.overall_risk_score} / 100
          </span>
        </div>
      </div>
      <div className="divide-y divide-primary/10 max-h-[300px] overflow-y-auto">
        {correlation.correlated_risks.map((risk, idx) => (
          <div key={idx} className="p-4 hover:bg-primary/[0.02] transition-colors">
            <div className="flex items-center gap-2 mb-2">
              <span className="text-[13px] font-bold text-foreground">{risk.title}</span>
            </div>
            <p className="text-[11px] text-muted-foreground/80 leading-relaxed mb-3">{risk.explanation}</p>
            {risk.evidences?.length > 0 && (
              <div className="space-y-1.5">
                <span className="text-[9px] uppercase tracking-widest font-bold text-primary/50">Combined Evidence</span>
                {risk.evidences.map((ev, i) => (
                  <div key={i} className="text-[10px] font-mono text-muted-foreground/60 bg-black/40 p-1.5 rounded border border-white/[0.02] break-all">
                    {ev}
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
