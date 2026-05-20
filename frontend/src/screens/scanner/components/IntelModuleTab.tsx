"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import { Loader2, Play, CheckCircle, AlertTriangle } from "lucide-react";
import { SmartDataViewer } from "@/widgets/smart-data-viewer/SmartDataViewer";

interface IntelModuleTabProps {
  title: string;
  targetUrl: string | null;
  onExecute: (url: string) => Promise<Record<string, unknown>>;
  isPending: boolean;
  result: Record<string, unknown> | undefined;
  error: Error | null;
}

export function IntelModuleTab({
  title,
  targetUrl,
  onExecute,
  isPending,
  result,
  error,
}: IntelModuleTabProps) {
  const [customUrl, setCustomUrl] = useState("");
  const effectiveUrl = customUrl || targetUrl;

  function handleRun() {
    if (!effectiveUrl) return;
    onExecute(effectiveUrl);
  }

  return (
    <div className="space-y-6">

      {/* URL input + run button */}
      <div className="flex items-center gap-3">
        <div className="flex-1 relative">
          <input
            type="text"
            placeholder={targetUrl ?? "Enter target URL…"}
            value={customUrl}
            onChange={(e) => setCustomUrl(e.target.value)}
            className="w-full h-10 px-4 text-[13px] font-mono bg-white/[0.03] border border-white/[0.06] rounded-lg focus:border-primary/50 focus:ring-1 focus:ring-primary/20 focus:outline-none transition-all text-foreground placeholder:text-muted-foreground/40"
          />
        </div>
        <button
          className={cn(
            "flex items-center gap-2 px-5 h-10 rounded-lg text-[12px] font-semibold transition-all duration-200 active:scale-[0.98]",
            isPending
              ? "bg-primary/10 text-primary border border-primary/20 cursor-wait"
              : "bg-primary/10 text-primary border border-primary/20 hover:bg-primary/20",
            (!effectiveUrl || isPending) && "opacity-50"
          )}
          disabled={!effectiveUrl || isPending}
          onClick={handleRun}
        >
          {isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Play className="h-4 w-4 shrink-0" />}
          Run {title}
        </button>
      </div>

      {/* Error */}
      {error && (
        <div className="flex items-center gap-3 text-[12px] font-medium text-red-400 bg-red-500/[0.06] border border-red-500/10 rounded-xl px-4 py-3">
          <AlertTriangle className="h-4 w-4 shrink-0" /> {error.message}
        </div>
      )}

      {/* Result */}
      {result && (
        <div className="bg-white/[0.02] border border-white/[0.06] rounded-xl shadow-lg overflow-hidden transition-all duration-300">
          <div className="p-4 flex items-center justify-between gap-3 border-b border-white/[0.04] bg-white/[0.01]">
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center h-6 w-6 rounded-md bg-emerald-500/10">
                <CheckCircle className="h-3.5 w-3.5 text-emerald-400" />
              </div>
              <span className="text-[13px] font-semibold text-foreground/90">Scan Results</span>
              <span className="text-[11px] font-mono font-medium text-muted-foreground/60 bg-white/[0.04] border border-white/[0.06] px-2 py-0.5 rounded-md">
                {Object.keys(result).length} records found
              </span>
            </div>
          </div>
          <div className="bg-transparent">
            <SmartDataViewer data={result} />
          </div>
        </div>
      )}
    </div>
  );
}
