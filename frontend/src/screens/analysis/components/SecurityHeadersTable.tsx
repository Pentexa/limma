"use client";

import { cn } from "@/shared/lib/utils";
import type { ApiSecurityHeaderResult } from "@/shared/types/api";
import { ShieldCheck, ShieldAlert, ShieldX } from "lucide-react";

interface SecurityHeadersTableProps {
  headers: ApiSecurityHeaderResult[] | undefined;
}

const STATUS_CONFIG = {
  present: { label: "Present", cls: "status-verified", icon: ShieldCheck },
  missing: { label: "Missing", cls: "text-risk", icon: ShieldX },
  weak: { label: "Weak", cls: "text-attention", icon: ShieldAlert },
  misconfigured: { label: "Misconfig", cls: "text-attention", icon: ShieldAlert },
} as const;

export function SecurityHeadersTable({ headers }: SecurityHeadersTableProps) {
  if (!headers || headers.length === 0) {
    return <div className="text-[11px] text-muted-foreground/50 py-8 text-center">No security header data</div>;
  }

  const presentCount = headers.filter(h => h.status === "present").length;
  const missingCount = headers.filter(h => h.status === "missing").length;

  return (
    <div className="space-y-3">
      {/* Summary & Coverage */}
      <div className="flex flex-col gap-3 px-1 mb-4 mt-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="flex items-baseline gap-1.5 bg-verified/10 border border-verified/20 px-2.5 py-1 rounded shadow-sm">
              <span className="text-[14px] font-mono font-bold text-verified">{presentCount}</span>
              <span className="text-[9px] uppercase tracking-wider text-verified/80 font-bold">Present</span>
            </div>
            <div className="flex items-baseline gap-1.5 bg-risk/10 border border-risk/20 px-2.5 py-1 rounded shadow-sm">
              <span className="text-[14px] font-mono font-bold text-risk">{missingCount}</span>
              <span className="text-[9px] uppercase tracking-wider text-risk/80 font-bold">Missing</span>
            </div>
          </div>
          <span className="text-[10px] font-mono uppercase tracking-widest text-muted-foreground/60 font-bold">
            {headers.length} Total Checked
          </span>
        </div>
        
        {/* Coverage bar */}
        <div className="h-1.5 w-full bg-[#111] rounded-sm overflow-hidden flex border border-border/20">
          <div 
            className="h-full bg-verified transition-all duration-1000 ease-out" 
            style={{ 
              width: `${(presentCount / headers.length) * 100}%`,
              boxShadow: presentCount > 0 ? '0 0 8px var(--verified)' : 'none'
            }} 
          />
        </div>
      </div>

      {/* List */}
      <div className="flex flex-col gap-1.5 mt-2 px-1">
        {headers.sort((a, b) => {
          const order = { missing: 0, weak: 1, misconfigured: 2, present: 3 };
          return (order[a.status] ?? 4) - (order[b.status] ?? 4);
        }).map((h, i) => {
          const config = STATUS_CONFIG[h.status] ?? STATUS_CONFIG.present;
          const Icon = config.icon;
          return (
            <div key={i} className="group bg-[#0f0f0f] border border-border/40 rounded shadow-sm hover:border-border transition-colors flex flex-col md:flex-row md:items-center gap-3 p-3">
              {/* Status Badge */}
              <div className="shrink-0 w-24">
                <span className={cn("inline-flex items-center gap-1.5 text-[9px] font-bold uppercase tracking-wider px-2 py-1 rounded", 
                  h.status === "present" ? "bg-verified/10 text-verified border border-verified/20" :
                  h.status === "missing" ? "bg-risk/10 text-risk border border-risk/20 drop-shadow-[0_0_3px_rgba(var(--risk),0.5)]" :
                  "bg-attention/10 text-attention border border-attention/20 drop-shadow-[0_0_3px_rgba(var(--attention),0.5)]"
                )}>
                  <Icon className="h-3 w-3" /> {config.label}
                </span>
              </div>
              
              {/* Header Info */}
              <div className="flex-1 min-w-0 flex flex-col gap-0.5">
                <span className="text-[12px] font-mono font-bold text-foreground/90">{h.name}</span>
                <span className="text-[10px] text-muted-foreground/70 leading-relaxed">{h.explanation}</span>
              </div>
              
              {/* Value Snippet */}
              <div className="shrink-0 md:w-64">
                <div className="bg-[#1a1a1a] border border-border/30 rounded p-1.5 max-h-[40px] overflow-hidden relative">
                  <span className="text-[9px] font-mono text-muted-foreground/80 leading-tight block pr-4">
                    {h.value || "—"}
                  </span>
                  {/* Fader to hide overflow text smoothly */}
                  {h.value && h.value.length > 50 && (
                    <div className="absolute top-0 right-0 bottom-0 w-8 bg-gradient-to-l from-[#1a1a1a] to-transparent pointer-events-none" />
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
