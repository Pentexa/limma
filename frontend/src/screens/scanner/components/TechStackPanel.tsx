"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import type { ApiDetectedTechnology } from "@/shared/types/api";
import { Cpu, ChevronDown, Code2 } from "lucide-react";

interface TechStackPanelProps {
  technologies: ApiDetectedTechnology[] | undefined;
}

export function TechStackPanel({ technologies }: TechStackPanelProps) {
  const [expandedCat, setExpandedCat] = useState<string | null>(null);

  if (!technologies || technologies.length === 0) {
    return <div className="text-[11px] text-muted-foreground/50 py-8 text-center">No technologies detected</div>;
  }

  // Group by category
  const grouped = technologies.reduce<Record<string, ApiDetectedTechnology[]>>((acc, tech) => {
    const cat = tech.category || "Other";
    if (!acc[cat]) acc[cat] = [];
    acc[cat].push(tech);
    return acc;
  }, {});

  const categories = Object.entries(grouped);

  return (
    <div className="space-y-4">
      {/* Summary HUD */}
      <div className="flex flex-wrap items-center gap-3 p-3 bg-[#080808] rounded-md border border-border/30 shadow-inner">
        <div className="flex items-center gap-2">
          <Cpu className="h-4 w-4 text-primary" />
          <span className="text-[11px] font-bold text-foreground uppercase tracking-widest">Technology Stack</span>
        </div>
        <div className="flex items-center gap-3 ml-auto">
          <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded">
            <span className="text-[15px] font-mono font-bold text-primary">{technologies.length}</span>
            <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">Detected</span>
          </div>
          <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded">
            <span className="text-[15px] font-mono font-bold text-primary">{categories.length}</span>
            <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">Categories</span>
          </div>
        </div>
      </div>

      {/* Category Chain */}
      <div className="relative pl-6 space-y-2">
        {/* Vertical timeline */}
        <div className="absolute top-3 bottom-3 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/40 to-transparent" />

        {categories.map(([category, techs]) => {
          const isExpanded = expandedCat === category;

          return (
            <div key={category} className="relative group">
              {/* Timeline Node */}
              <div className={cn(
                "absolute -left-5 top-4 w-3 h-3 rounded-full border-2 border-background z-10 transition-transform duration-300",
                "bg-primary shadow-[0_0_8px_var(--primary)]",
                isExpanded && "scale-125"
              )} />

              {/* Category Card */}
              <div className={cn(
                "bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2",
                isExpanded ? "border-primary/40" : "border-border/20 hover:border-border/50"
              )}>
                <div
                  className="p-3.5 cursor-pointer flex items-center justify-between gap-3 hover:bg-white/[0.02]"
                  onClick={() => setExpandedCat(isExpanded ? null : category)}
                >
                  <div className="flex items-center gap-3">
                    <Code2 className={cn("h-4 w-4", isExpanded ? "text-primary drop-shadow-[0_0_4px_var(--primary)]" : "text-muted-foreground/60")} />
                    <span className={cn("text-[12px] font-bold uppercase tracking-wider transition-colors", isExpanded ? "text-primary" : "text-foreground/90")}>
                      {category}
                    </span>
                    <span className="text-[9px] font-mono text-muted-foreground/50 bg-[#111] border border-border/10 px-1.5 py-0.5 rounded">{techs.length}</span>
                  </div>
                  <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-300", isExpanded && "rotate-180")} />
                </div>

                {/* Expandable Technologies */}
                <div className={cn("grid transition-all duration-300 ease-in-out", isExpanded ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")}>
                  <div className="overflow-hidden">
                    <div className="px-4 pb-4 pt-2 border-t border-border/10 bg-black/40 space-y-3">
                      {techs.map((tech, i) => (
                        <div key={i} className="bg-[#0c0c0c] border border-border/15 rounded p-3 space-y-2">
                          <div className="flex items-center justify-between">
                            <span className="text-[12px] font-bold text-foreground">{tech.name}</span>
                            <span className={cn("text-[10px] font-mono tabular-nums font-bold",
                              tech.confidence_score >= 80 ? "text-verified" : tech.confidence_score >= 50 ? "text-foreground" : "text-muted-foreground"
                            )}>{tech.confidence_score}%</span>
                          </div>
                          {tech.evidences?.length > 0 && (
                            <div className="space-y-1.5">
                              <span className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Evidence</span>
                              {tech.evidences.slice(0, 3).map((ev, j) => (
                                <div key={j} className="flex items-start gap-2 text-[10px]">
                                  <span className="text-[9px] text-primary/50 uppercase tracking-widest font-bold w-[45px] shrink-0 mt-0.5">{ev.source}</span>
                                  <span className="font-mono text-muted-foreground/80 break-all">{ev.snippet}</span>
                                </div>
                              ))}
                            </div>
                          )}
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
