"use client";

import { useMemo } from "react";
import { cn } from "@/shared/lib/utils";
import { useScans } from "@/entities/scan/model/use-scans";
import { useScanTrends } from "@/entities/scan/model/use-scan-trends";
import { useActiveMasterReport } from "@/entities/discovery/model/use-master-report";
import { TrendChart } from "./components/TrendChart";
import { SecurityHeadersTable } from "./components/SecurityHeadersTable";
import { RiskCorrelationPanel } from "./components/RiskCorrelationPanel";
import { BarChart3, Loader2, TrendingUp, Shield } from "lucide-react";
import { WafBadge } from "@/shared/ui/WafBadge";

export function AnalysisScreen() {
  const { data: scans = [] } = useScans();
  const activeScan = scans.find((s) => s.status === "running") ?? scans[0];
  const targetUrl = activeScan?.targetUrl;

  const { data: trends = [], isLoading: trendsLoading } = useScanTrends(targetUrl);
  const { data: report, isLoading: reportLoading } = useActiveMasterReport();

  const isLoading = trendsLoading || reportLoading;

  // Find if WAF is detected
  const wafFinding = report?.analysis?.risk_insights?.find((f: any) => f.title?.startsWith("WAF Detected:"));
  const wafName = wafFinding ? wafFinding.title.replace("WAF Detected: ", "") : null;

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-4 py-2.5 border-b border-border bg-card/50">
        <div className="flex items-center gap-1.5">
          <BarChart3 className="h-4 w-4 text-primary" />
          <h2 className="text-[13px] font-bold tracking-tight">Analysis</h2>
          {targetUrl && <span className="text-[10px] font-mono text-muted-foreground ml-2">{targetUrl}</span>}
          {wafName && <WafBadge wafName={wafName} className="ml-2" />}
        </div>
        {isLoading && (
          <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground animate-pulse">
            <Loader2 className="h-3 w-3 animate-spin" /> Loading analysis…
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {!targetUrl ? (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground/50">
            <BarChart3 className="h-8 w-8 mb-3 opacity-40" />
            <span className="text-[12px]">No scan data available</span>
            <span className="text-[10px] mt-1">Start a scan to see analysis results</span>
          </div>
        ) : (
          <>
            {/* Row 1: Risk correlation (Attack Path) */}
            <div className="bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col">
              <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent flex items-center gap-2">
                <Shield className="h-4 w-4 text-primary" />
                <span className="text-[13px] font-bold tracking-wide text-foreground">Risk Correlation</span>
              </div>
              <div className="p-4 flex-1">
                <RiskCorrelationPanel correlation={report?.analysis?.correlation} />
              </div>
            </div>

            {/* Row 2: Trend chart */}
            <div className="bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col">
              <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <TrendingUp className="h-4 w-4 text-primary" />
                  <span className="text-[13px] font-bold tracking-wide text-foreground">Security Score Trend</span>
                </div>
                <span className="text-[11px] font-mono font-bold text-primary/80 bg-primary/10 px-2 py-0.5 rounded">{trends.length} scans recorded</span>
              </div>
              <div className="p-5">
                <TrendChart points={trends} />
              </div>
            </div>

            {/* Row 3: Security headers */}
            <div className="bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col">
              <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent flex items-center gap-2">
                <Shield className="h-4 w-4 text-primary" />
                <span className="text-[13px] font-bold tracking-wide text-foreground">Security Headers Analysis</span>
              </div>
              <div className="p-0">
                <SecurityHeadersTable headers={report?.analysis?.security_headers} />
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
