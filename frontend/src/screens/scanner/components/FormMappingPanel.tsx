"use client";

import { cn } from "@/shared/lib/utils";
import type { ApiFormMapping } from "@/shared/types/api";
import { FileInput, KeyRound, ArrowRight, Type } from "lucide-react";

interface FormMappingPanelProps {
  data: ApiFormMapping;
}

export function FormMappingPanel({ data }: FormMappingPanelProps) {
  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500 max-w-5xl mx-auto">
      
      {/* Login Pages */}
      {data.login_pages_found?.length > 0 && (
        <div className="bg-primary/5 border border-primary/20 rounded-xl overflow-hidden shadow-[0_0_15px_rgba(var(--primary-rgb),0.05)]">
          <div className="p-4 border-b border-primary/10 bg-primary/[0.02] flex items-center gap-3">
            <KeyRound className="h-4 w-4 text-primary" />
            <h3 className="text-[13px] font-bold uppercase tracking-wider text-primary">Login Pages Found</h3>
            <span className="ml-auto text-[10px] font-mono bg-primary/10 text-primary px-2 py-0.5 rounded-md">
              {data.login_pages_found.length}
            </span>
          </div>
          <div className="p-4 flex flex-col gap-2">
            {data.login_pages_found.map((page, i) => (
              <div key={i} className="flex items-center gap-3 bg-black/40 px-3 py-2 rounded-lg border border-primary/10">
                <ArrowRight className="h-3 w-3 text-primary/50" />
                <span className="text-[12px] font-mono text-foreground/90">{page}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Forms Grid */}
      <div className="bg-[#080808] border border-border/20 rounded-xl overflow-hidden shadow-lg">
        <div className="p-4 border-b border-border/10 bg-white/[0.01] flex items-center gap-3">
          <FileInput className="h-4 w-4 text-foreground/60" />
          <h3 className="text-[13px] font-bold uppercase tracking-wider text-foreground/90">Detected Forms</h3>
          <span className="ml-auto text-[10px] font-mono bg-white/[0.04] px-2 py-0.5 rounded-md text-muted-foreground">
            {data.detected_forms?.length || 0} Forms
          </span>
        </div>
        <div className="p-4 grid grid-cols-1 md:grid-cols-2 gap-4">
          {data.detected_forms?.length > 0 ? (
            data.detected_forms.map((form, i) => (
              <div key={i} className="bg-white/[0.01] border border-white/[0.04] rounded-lg p-4 hover:border-white/[0.08] transition-colors">
                <div className="flex items-center justify-between gap-3 mb-3 pb-3 border-b border-border/10">
                  <div className="flex items-center gap-2 max-w-[70%]">
                    <span className={cn(
                      "text-[9px] font-mono font-bold uppercase px-1.5 py-0.5 rounded shrink-0",
                      form.method.toUpperCase() === "POST" ? "bg-blue-500/10 text-blue-400" :
                      form.method.toUpperCase() === "GET" ? "bg-emerald-500/10 text-emerald-400" :
                      "bg-white/10 text-foreground"
                    )}>
                      {form.method || "UNKNOWN"}
                    </span>
                    <span className="text-[12px] font-mono text-foreground truncate" title={form.action}>
                      {form.action || "No Action (Self)"}
                    </span>
                  </div>
                  <span className="text-[10px] text-muted-foreground/50 font-medium">
                    {form.fields.length} Fields
                  </span>
                </div>
                
                <div className="flex flex-wrap gap-2">
                  {form.fields.length > 0 ? form.fields.map((field, j) => (
                    <div key={j} className="flex items-center gap-1.5 bg-black/50 border border-white/[0.03] px-2 py-1 rounded text-[11px] font-mono text-muted-foreground/80">
                      <Type className="h-3 w-3 text-muted-foreground/40" />
                      {field}
                    </div>
                  )) : (
                    <span className="text-[11px] text-muted-foreground/40 italic">No named fields detected</span>
                  )}
                </div>
              </div>
            ))
          ) : (
            <div className="col-span-full py-8 text-center text-[12px] text-muted-foreground/50">
              No HTML forms detected on this target.
            </div>
          )}
        </div>
      </div>

    </div>
  );
}
