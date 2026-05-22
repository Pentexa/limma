"use client";

import { cn } from "@/shared/lib/utils";
import type { ApiSecurityReport } from "@/shared/types/api";
import { ShieldCheck, Target, FileWarning, Lightbulb, Activity, CheckCircle, AlertTriangle } from "lucide-react";

interface SecurityAuditPanelProps {
  data: ApiSecurityReport;
}

export function SecurityAuditPanel({ data }: SecurityAuditPanelProps) {
  const getScoreColor = (score: number) => {
    if (score >= 80) return "text-emerald-400 bg-emerald-500/10 border-emerald-500/20 shadow-[0_0_15px_rgba(52,211,153,0.15)]";
    if (score >= 50) return "text-yellow-400 bg-yellow-500/10 border-yellow-500/20 shadow-[0_0_15px_rgba(250,204,21,0.15)]";
    return "text-red-400 bg-red-500/10 border-red-500/20 shadow-[0_0_15px_rgba(248,113,113,0.15)]";
  };

  const getScoreIndicator = (score: number) => {
    if (score >= 80) return "bg-emerald-400 shadow-[0_0_8px_#34d399]";
    if (score >= 50) return "bg-yellow-400 shadow-[0_0_8px_#facc15]";
    return "bg-red-400 shadow-[0_0_8px_#f87171]";
  };

  return (
    <div className="space-y-4 w-full">
      {/* Top HUD */}
      <div className="flex flex-wrap items-center gap-3 p-3 bg-[#080808] rounded-md border border-border/30 shadow-inner">
        <div className="flex items-center gap-2">
          <ShieldCheck className="h-4 w-4 text-primary" />
          <span className="text-[11px] font-bold text-foreground uppercase tracking-widest">Security Audit</span>
        </div>
        <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded ml-auto">
          <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">Target</span>
          <span className="font-mono text-[11px] text-foreground font-bold">{data.url}</span>
        </div>
        <div className={cn("flex items-baseline gap-1.5 px-3 py-1 rounded border", getScoreColor(data.security_score))}>
          <span className="text-[15px] font-mono font-bold">{data.security_score}</span>
          <span className="text-[9px] uppercase tracking-widest opacity-70 font-bold">Score</span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Missing Headers */}
        <div className="bg-[#080808] border border-border/20 rounded-md shadow-lg overflow-hidden hover:border-orange-500/30 transition-colors">
          <div className="p-3.5 flex items-center justify-between gap-3 bg-white/[0.01]">
            <div className="flex items-center gap-3">
              <FileWarning className="h-4 w-4 text-orange-400 drop-shadow-[0_0_4px_#f97316]" />
              <span className="text-[12px] font-bold uppercase tracking-wider text-orange-400">Missing Headers</span>
              <span className="text-[9px] font-mono text-muted-foreground/50 bg-[#111] border border-border/10 px-1.5 py-0.5 rounded">
                {data.missing_headers?.length || 0}
              </span>
            </div>
          </div>
          <div className="p-4 border-t border-border/10 bg-black/40 h-full max-h-[250px] overflow-y-auto">
            {data.missing_headers?.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {data.missing_headers.map((header, i) => (
                  <div key={i} className="flex items-center gap-1.5 text-[10px] font-mono bg-orange-500/10 text-orange-400 border border-orange-500/20 px-2 py-1 rounded shadow-inner">
                    <AlertTriangle className="h-2.5 w-2.5" />
                    {header}
                  </div>
                ))}
              </div>
            ) : (
              <div className="flex items-center gap-2 text-[10px] text-emerald-400/80 font-mono">
                <CheckCircle className="h-3 w-3" /> No critical headers missing
              </div>
            )}
          </div>
        </div>

        {/* Robot Rules */}
        <div className="bg-[#080808] border border-border/20 rounded-md shadow-lg overflow-hidden hover:border-primary/40 transition-colors">
          <div className="p-3.5 flex items-center justify-between gap-3 bg-white/[0.01]">
            <div className="flex items-center gap-3">
              <Target className="h-4 w-4 text-primary drop-shadow-[0_0_4px_var(--primary)]" />
              <span className="text-[12px] font-bold uppercase tracking-wider text-primary">Disallowed Paths</span>
              <span className="text-[9px] font-mono text-muted-foreground/50 bg-[#111] border border-border/10 px-1.5 py-0.5 rounded">
                {data.robot_rules_disallowed?.length || 0}
              </span>
            </div>
          </div>
          <div className="p-4 border-t border-border/10 bg-black/40 h-full max-h-[250px] overflow-y-auto">
            {data.robot_rules_disallowed?.length > 0 ? (
              <div className="space-y-1.5">
                {data.robot_rules_disallowed.map((rule, i) => (
                  <div key={i} className="flex items-start gap-2 text-[10px] font-mono text-muted-foreground/80 bg-[#1a1a1a] px-2 py-1.5 rounded border border-white/[0.02] shadow-inner">
                    <Activity className="h-3 w-3 text-primary/50 mt-0.5 shrink-0" />
                    <span className="break-all">{rule}</span>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-[10px] text-muted-foreground/50 font-mono">No disallowed rules found in robots.txt</div>
            )}
          </div>
        </div>

        {/* Recommendations */}
        <div className="bg-[#080808] border border-border/20 rounded-md shadow-lg overflow-hidden hover:border-emerald-500/30 transition-colors lg:col-span-2">
          <div className="p-3.5 flex items-center justify-between gap-3 bg-white/[0.01]">
            <div className="flex items-center gap-3">
              <Lightbulb className="h-4 w-4 text-emerald-400 drop-shadow-[0_0_4px_#34d399]" />
              <span className="text-[12px] font-bold uppercase tracking-wider text-emerald-400">Actionable Recommendations</span>
              <span className="text-[9px] font-mono text-muted-foreground/50 bg-[#111] border border-border/10 px-1.5 py-0.5 rounded">
                {data.recommendations?.length || 0}
              </span>
            </div>
          </div>
          <div className="p-0 border-t border-border/10 bg-black/40">
            {data.recommendations?.length > 0 ? (
              <div className="divide-y divide-border/10">
                {data.recommendations.map((rec, i) => (
                  <div key={i} className="p-4 hover:bg-white/[0.01] transition-colors flex items-start gap-3">
                    <div className="mt-1 h-1.5 w-1.5 rounded-full bg-emerald-500/50 shrink-0 shadow-[0_0_4px_#34d399]" />
                    <span className="text-[11px] text-muted-foreground/80 leading-relaxed font-mono">{rec}</span>
                  </div>
                ))}
              </div>
            ) : (
              <div className="p-4 text-[10px] text-muted-foreground/50 font-mono text-center">No recommendations available. Your security posture is solid.</div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
