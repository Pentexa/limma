"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import type { ApiServerInfo } from "@/shared/types/api";
import { Globe, Fingerprint, ShieldCheck, Truck, Activity, ChevronDown, Zap } from "lucide-react";

interface ServerInfoPanelProps {
  serverInfo: ApiServerInfo | undefined;
}

export function ServerInfoPanel({ serverInfo }: ServerInfoPanelProps) {
  const [expandedSection, setExpandedSection] = useState<string | null>("fingerprints");

  if (!serverInfo) {
    return <div className="text-[11px] text-muted-foreground/50 py-8 text-center">No server information available</div>;
  }

  const sections = [
    {
      id: "fingerprints",
      label: "Fingerprints",
      icon: Fingerprint,
      count: serverInfo.fingerprints?.length ?? 0,
      items: serverInfo.fingerprints,
    },
    {
      id: "security",
      label: "Security Insights",
      icon: ShieldCheck,
      count: serverInfo.security_insights?.length ?? 0,
      items: serverInfo.security_insights,
    },
    {
      id: "delivery",
      label: "Delivery Insights",
      icon: Truck,
      count: serverInfo.delivery_insights?.length ?? 0,
      items: serverInfo.delivery_insights,
    },
    {
      id: "infrastructure",
      label: "Infrastructure Signals",
      icon: Activity,
      count: serverInfo.infrastructure_signals?.length ?? 0,
      items: serverInfo.infrastructure_signals,
    },
  ].filter(s => s.count > 0);

  return (
    <div className="space-y-4">
      {/* Server Overview HUD */}
      <div className="flex flex-wrap items-center gap-3 p-3 bg-[#080808] rounded-md border border-border/30 shadow-inner">
        <div className="flex items-center gap-2">
          <Globe className="h-4 w-4 text-primary" />
          <span className="text-[11px] font-bold text-foreground uppercase tracking-widest">Server Overview</span>
        </div>
        <div className="flex flex-wrap items-center gap-3 ml-auto">
          {[
            { label: "Target", value: serverInfo.original_target },
            { label: "Status", value: serverInfo.status_code?.toString() ?? "—" },
            { label: "Latency", value: `${serverInfo.latency_ms}ms` },
          ].map(item => (
            <div key={item.label} className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded">
              <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">{item.label}</span>
              <span className="font-mono text-[11px] text-foreground font-bold">{item.value}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Resolved URL */}
      {serverInfo.resolved_url && (
        <div className="flex items-center gap-2 px-1">
          <span className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Resolved</span>
          <span className="font-mono text-[10px] text-primary bg-primary/5 border border-primary/10 rounded px-2 py-0.5">{serverInfo.resolved_url}</span>
        </div>
      )}

      {/* Accordion sections */}
      <div className="relative pl-6 space-y-2">
        {/* Vertical timeline */}
        <div className="absolute top-3 bottom-3 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/40 to-transparent" />

        {sections.map((section) => {
          const isExpanded = expandedSection === section.id;
          const Icon = section.icon;

          return (
            <div key={section.id} className="relative group">
              {/* Timeline Node */}
              <div className={cn(
                "absolute -left-5 top-4 w-3 h-3 rounded-full border-2 border-background z-10 transition-transform duration-300",
                "bg-primary shadow-[0_0_8px_var(--primary)]",
                isExpanded && "scale-125"
              )} />

              {/* Section Card */}
              <div className={cn(
                "bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2",
                isExpanded ? "border-primary/40" : "border-border/20 hover:border-border/50"
              )}>
                <div
                  className="p-3.5 cursor-pointer flex items-center justify-between gap-3 hover:bg-white/[0.02]"
                  onClick={() => setExpandedSection(isExpanded ? null : section.id)}
                >
                  <div className="flex items-center gap-3">
                    <Icon className={cn("h-4 w-4", isExpanded ? "text-primary drop-shadow-[0_0_4px_var(--primary)]" : "text-muted-foreground/60")} />
                    <span className={cn("text-[12px] font-bold uppercase tracking-wider transition-colors", isExpanded ? "text-primary" : "text-foreground/90")}>
                      {section.label}
                    </span>
                    <span className="text-[9px] font-mono text-muted-foreground/50 bg-[#111] border border-border/10 px-1.5 py-0.5 rounded">{section.count}</span>
                  </div>
                  <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-300", isExpanded && "rotate-180")} />
                </div>

                {/* Expandable Content */}
                <div className={cn("grid transition-all duration-300 ease-in-out", isExpanded ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")}>
                  <div className="overflow-hidden">
                    <div className="px-4 pb-4 pt-2 border-t border-border/10 bg-black/40 space-y-3">
                      {section.id === "fingerprints" && serverInfo.fingerprints?.map((fp, i) => (
                        <div key={i} className="bg-[#0c0c0c] border border-border/15 rounded p-3 space-y-1.5">
                          <div className="flex items-center justify-between">
                            <span className="text-[11px] font-bold text-foreground">{fp.name}</span>
                            <span className={cn("text-[10px] font-mono tabular-nums font-bold",
                              fp.confidence_score >= 80 ? "text-verified" : "text-muted-foreground"
                            )}>{fp.confidence_score}%</span>
                          </div>
                          <span className="text-[9px] text-primary/60 uppercase tracking-widest font-bold">{fp.category}</span>
                          <p className="text-[10px] text-muted-foreground/70 leading-relaxed font-mono">{fp.explanation}</p>
                        </div>
                      ))}

                      {section.id === "security" && serverInfo.security_insights?.map((si, i) => (
                        <div key={i} className="bg-[#0c0c0c] border border-border/15 rounded p-3 space-y-1.5">
                          <div className="flex items-center justify-between">
                            <span className="text-[11px] font-bold text-foreground">{si.name}</span>
                            <span className={cn("text-[9px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded border",
                              si.status === "present" ? "bg-verified/10 text-verified border-verified/20" :
                              si.status === "missing" ? "bg-risk/10 text-risk border-risk/20" : "bg-attention/10 text-attention border-attention/20"
                            )}>{si.status}</span>
                          </div>
                          <span className="text-[9px] text-primary/60 uppercase tracking-widest font-bold">{si.category}</span>
                          <p className="text-[10px] text-muted-foreground/70 leading-relaxed font-mono">{si.explanation}</p>
                        </div>
                      ))}

                      {section.id === "delivery" && serverInfo.delivery_insights?.map((di, i) => (
                        <div key={i} className="bg-[#0c0c0c] border border-border/15 rounded p-3 space-y-1.5">
                          <div className="flex items-center justify-between">
                            <span className="text-[11px] font-bold text-foreground">{di.name}</span>
                            <span className="text-[10px] font-mono tabular-nums font-bold text-muted-foreground">{di.confidence_score}%</span>
                          </div>
                          <p className="text-[10px] text-muted-foreground/70 leading-relaxed font-mono">{di.explanation}</p>
                        </div>
                      ))}

                      {section.id === "infrastructure" && serverInfo.infrastructure_signals?.map((sig, i) => (
                        <div key={i} className="flex items-center gap-4 bg-[#0c0c0c] border border-border/15 rounded p-3">
                          <Zap className="h-3 w-3 text-primary shrink-0" />
                          <span className="text-[9px] text-primary/60 uppercase tracking-widest font-bold w-[80px] shrink-0">{sig.signal_type}</span>
                          <span className="font-mono text-[11px] text-foreground/90 break-all">{sig.value}</span>
                        </div>
                      ))}
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
