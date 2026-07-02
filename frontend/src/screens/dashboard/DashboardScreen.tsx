"use client";

import { useScans } from "@/entities/scan/model/use-scans";
import { useGlobalFindings } from "@/entities/finding/model/use-findings";
import { SCAN_PHASES, PHASE_LABELS, type ScanPhase } from "@/shared/config/constants";
import { DETECTOR_META, type DetectorType, type Confidence, type VerificationStatus } from "@/entities/finding/model/types";
import type { Severity } from "@/shared/types/common";
import { cn } from "@/shared/lib/utils";
import { Shield, AlertTriangle, CheckCircle, Clock, Eye, Layers, Target } from "lucide-react";
import { compareBySeverity } from "@/shared/config/priority";
import { useEffect, useMemo, useState } from "react";
import { useStreamStore } from "@/features/stream-scan-events/model/stream-store";
import { startScanStream, getCurrentStreamTarget } from "@/features/stream-scan-events/model/scan-stream-manager";
import { EMPTY_SCAN } from "@/entities/scan/model/constants";
import { LoadingSpinner } from "@/shared/ui/loading-spinner";
import { SPACING } from "@/shared/ui/tokens";
import { FindingFilter } from "@/features/finding-filter/FindingFilter";
import { exportFindingsAsCsv, exportFindingsAsJson } from "@/features/export/export-utils";
import { Download } from "lucide-react";
import { SeverityBadge, StatusBadge } from "@/shared/ui/badges";

const PHASE_COLORS: Record<ScanPhase, string> = {
  recon: "bg-recon",
  analysis: "bg-analysis",
  scan: "bg-attention",
  exploit: "bg-risk",
};

/* ── Helpers ── */
function confidenceLabel(c: string) {
  return { confirmed: "Confirmed", high: "High", medium: "Medium", low: "Low", tentative: "Tentative" }[c] ?? c;
}
function confidenceColor(c: string) {
  return { confirmed: "status-verified", high: "text-foreground", medium: "text-attention", low: "text-muted-foreground", tentative: "status-tentative" }[c] ?? "";
}
function fpRisk(confidence: string, verification: string): { label: string; cls: string } {
  if (verification === "verified") return { label: "None", cls: "status-verified" };
  if (confidence === "confirmed" || confidence === "high") return { label: "Low", cls: "text-foreground" };
  if (confidence === "medium") return { label: "Moderate", cls: "status-tentative" };
  return { label: "High", cls: "sev-critical" };
}



export function DashboardScreen() {
  const [filterQuery, setFilterQuery] = useState("");
  
  /* ── Real API data via React Query ── */
  const { data: scans = [], isLoading: scansLoading } = useScans();
  const activeScan = scans.find((s) => s.status === "running") ?? scans[0] ?? EMPTY_SCAN;

  // Fetch combined global findings (Active + Master)
  const { data: findings = [] } = useGlobalFindings();

  // Memoize all expensive calculations derived from findings
  const {
    normalizedFindings,
    criticalFindings,
    highFindings,
    mediumFindings,
    lowFindings,
    verifiedFindings,
    unverifiedFindings,
    priorityFindings,
    detectorSignals,
    riskScore,
    riskColor,
    authWeaknesses,
    endpointsCount,
    parametersCount,
    evidenceCoverage,
    authBoundsCount,
    apiRoutesCount,
    inputVectorsCount
  } = useMemo(() => {
    // Normalize severity/confidence to lowercase (defensive)
    const normalized = findings.map(f => ({
      ...f,
      severity: (f.severity?.toLowerCase() ?? "info") as Severity,
      confidence: (f.confidence?.toLowerCase() ?? "tentative") as Confidence,
      verification: (f.verification?.toLowerCase() ?? "unverified") as VerificationStatus,
    }));

    const critical = normalized.filter(f => f.severity === "critical");
    const high = normalized.filter(f => f.severity === "high");
    const medium = normalized.filter(f => f.severity === "medium");
    const low = normalized.filter(f => f.severity === "low");
    const verified = normalized.filter(f => f.verification === "verified");
    const unverified = normalized.filter(f => f.verification !== "verified");

    const priority = [...normalized].sort(compareBySeverity);

    const signals: Record<string, { signals: number; confidence: number }> = {};
    for (const det of Object.keys(DETECTOR_META) as DetectorType[]) {
      const detFindings = normalized.filter(f => f.detector === det);
      signals[det] = {
        signals: detFindings.length,
        confidence: detFindings.length > 0
          ? Math.round(detFindings.reduce((sum, f) => sum + (f.cvss ?? 0), 0) / detFindings.length * 10)
          : 0,
      };
    }

    const deduction = critical.length * 20 + high.length * 10 + medium.length * 5;
    const score = normalized.length > 0 ? Math.max(0, 100 - deduction) : 100;
    const color = score >= 80 ? "text-verified" : score >= 50 ? "text-attention" : "text-risk";
    
    // Safely access title and cwe
    const auth = normalized.filter(f => 
      f.cwe?.includes("Auth") || 
      f.title?.toLowerCase().includes("auth") || 
      f.title?.toLowerCase().includes("jwt")
    ).length;
    
    const epsCount = activeScan.result?.totalEndpoints;
    const paramsCount = activeScan.result?.totalParameters;
    
    const wEvidence = normalized.filter(f => f.evidence && f.evidence.length > 0).length;
    const coverage = normalized.length > 0 ? Math.round((wEvidence / normalized.length) * 100) : 0;

    return {
      normalizedFindings: normalized,
      criticalFindings: critical,
      highFindings: high,
      mediumFindings: medium,
      lowFindings: low,
      verifiedFindings: verified,
      unverifiedFindings: unverified,
      priorityFindings: priority,
      detectorSignals: signals,
      riskScore: score,
      riskColor: color,
      authWeaknesses: auth,
      endpointsCount: epsCount,
      parametersCount: paramsCount,
      evidenceCoverage: coverage,
      authBoundsCount: activeScan.result?.authBoundsIdentified,
      apiRoutesCount: activeScan.result?.apiRoutesMapped,
      inputVectorsCount: activeScan.result?.inputVectorsAnalyzed
    };
  }, [findings, activeScan.result]);

  // Apply filtering to priority findings
  const filteredPriorityFindings = useMemo(() => {
    if (!filterQuery) return priorityFindings;
    const q = filterQuery.toLowerCase();
    return priorityFindings.filter(f => 
      f.title.toLowerCase().includes(q) || 
      f.url.toLowerCase().includes(q) || 
      (f.parameter && f.parameter.toLowerCase().includes(q))
    );
  }, [priorityFindings, filterQuery]);

  /* ── SSE Stream (read-only from global store) ── */
  const { events, connectionStatus, localScanState, localScanTarget } = useStreamStore();

  const isScanning = 
    localScanState === "starting" || 
    localScanState === "running" || 
    activeScan.status === "running" || 
    activeScan.status === "pending" || 
    activeScan.status === "starting";

  const displayStatus = isScanning ? "running" : activeScan.status;
  const displayTargetUrl = (isScanning && localScanTarget) ? localScanTarget : activeScan.targetUrl;

  // Catch-up only: if page reloads while a scan is running, ensure SSE re-connects.
  // This does NOT manage the SSE lifecycle — StartScanButton does that.
  useEffect(() => {
    if (activeScan.status === "running" && activeScan.targetUrl && activeScan.targetUrl !== "—") {
      if (!getCurrentStreamTarget()) {
        startScanStream(activeScan.targetUrl);
        useStreamStore.getState().setScanRunning();
      }
    }
  }, [activeScan.status, activeScan.targetUrl]);

  // Show loading spinner ONLY on the very first fetch (no data yet).
  // Once we have data (even empty array), don't block the UI on refetches.
  const isFirstLoad = scansLoading && scans.length === 0;

  return (
    <div className={cn("w-full max-w-full min-w-0 flex flex-col", SPACING.panel, SPACING.gap.md)}>

      {/* Loading indicator — only on first page load, never on refetches */}
      {isFirstLoad && <LoadingSpinner message="Loading…" size="sm" className="py-2 justify-start" />}

      {/* ══════════════════════════════════════
          ROW 1: Scan Status + Summary Metrics
          ══════════════════════════════════════ */}
      <div className="grid grid-cols-12 gap-2">

        {/* Active Scan */}
        {/* Active Scan */}
        <div className="col-span-12 lg:col-span-5 bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col">
          <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent flex items-center justify-between">
            <span className="text-[13px] font-bold tracking-wide text-foreground flex items-center gap-2">
              <Target className="h-4 w-4 text-primary" /> Active Scan
            </span>
            <span className="flex items-center gap-1.5 text-[10px]">
              {displayStatus === "running" ? (
                <span className="relative flex h-1.5 w-1.5">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-60" />
                  <span className="relative inline-flex rounded-full h-1.5 w-1.5 bg-primary shadow-[0_0_6px_hsl(207_90%_54%/0.5)]" />
                </span>
              ) : (
                <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground/40" />
              )}
              <span className={cn(
                "font-semibold uppercase tracking-wider",
                displayStatus === "running" ? "text-primary" : "text-muted-foreground"
              )}>{displayStatus}</span>
            </span>
          </div>
          <div className="p-4 space-y-2.5 flex-1">
            <div className="flex items-baseline justify-between">
              <span className="text-[10px] text-muted-foreground uppercase tracking-widest font-semibold">Target</span>
              <code className="text-[11px] font-mono text-foreground font-medium">{displayTargetUrl}</code>
            </div>
            <div className="space-y-1.5">
              {SCAN_PHASES.map((phase) => {
                const progress = activeScan.phaseProgress[phase];
                const isActive = progress > 0 && progress < 100;
                const isDone = progress === 100;
                return (
                  <div key={phase} className="flex items-center gap-2.5">
                    <span className={cn(
                      "text-[10px] w-[96px] shrink-0 flex items-center gap-1 whitespace-nowrap",
                      isActive ? "text-foreground font-bold" : isDone ? "text-muted-foreground font-medium" : "text-muted-foreground/30 font-medium"
                    )}>
                      <span className="w-3 text-center shrink-0">
                        {isDone ? "✓" : isActive ? "›" : " "}
                      </span>
                      <span>{PHASE_LABELS[phase]}</span>
                    </span>
                    <div className="flex-1 progress-bar bg-muted/30">
                      <div className={cn("progress-fill relative", PHASE_COLORS[phase], 
                        isDone ? "opacity-50" : isActive ? "opacity-100" : "opacity-0"
                      )} style={{ width: `${progress}%` }}>
                        {isActive && (
                          <div className="absolute inset-0 bg-gradient-to-r from-transparent to-white/20" />
                        )}
                      </div>
                    </div>
                    <span className={cn("text-[10px] font-mono w-7 text-right tabular-nums",
                      isActive ? "text-foreground font-bold" : isDone ? "text-muted-foreground" : "text-muted-foreground/30"
                    )}>{progress}%</span>
                  </div>
                );
              })}
            </div>
            <div className="flex items-center gap-4 pt-2 border-t border-border text-[10px] text-muted-foreground/50">
              <span className="flex items-center gap-1"><Clock className="h-2.5 w-2.5" />{activeScan.startedAt ? new Date(activeScan.startedAt).toLocaleTimeString("en-US", { hour12: false }) : "—"}</span>
              <span>Profile: <span className="font-medium text-muted-foreground/90">{activeScan.config.profileId ?? "default"}</span></span>
              <span>ID: <span className="font-mono text-muted-foreground/90">{activeScan.id.slice(0, 8)}</span></span>
            </div>
          </div>
        </div>

        {/* Findings Count — computed from actual findings, not scan summary */}
        <div className="col-span-4 lg:col-span-2 bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col">
          <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent flex items-center justify-between"><span className="text-[13px] font-bold tracking-wide text-foreground">Findings</span></div>
          <div className="p-4 flex-1 flex flex-col">
            <div className="space-y-0.5 flex-1">
              {[
                { label: "Critical", count: criticalFindings.length, cls: "sev-critical", dot: "sev-dot-critical" },
                { label: "High", count: highFindings.length, cls: "sev-high", dot: "sev-dot-high" },
                { label: "Medium", count: mediumFindings.length, cls: "sev-medium", dot: "sev-dot-medium" },
                { label: "Low", count: lowFindings.length, cls: "sev-low", dot: "sev-dot-low" },
              ].map(s => (
                <div key={s.label} className="flex items-center justify-between py-1 group cursor-default">
                  <span className="text-[11px] font-medium text-muted-foreground flex items-center gap-2 transition-colors duration-200 group-hover:text-foreground/90">
                    <span className={cn("sev-dot opacity-70 group-hover:opacity-100 transition-opacity duration-200 shadow-sm", s.dot)} /> {s.label}
                  </span>
                  <span className={cn("text-[13px] font-bold tabular-nums drop-shadow-sm transition-all duration-200", s.cls)}>{s.count}</span>
                </div>
              ))}
            </div>
            <div className="pt-2 mt-2 border-t border-border/50 flex items-center justify-between shrink-0">
              <span className="text-[10px] text-muted-foreground uppercase tracking-widest font-semibold">Total</span>
              <span className="text-[14px] font-bold tabular-nums text-foreground drop-shadow-sm">{normalizedFindings.length}</span>
            </div>
          </div>
        </div>

        {/* Truth Layer */}
        <div className="col-span-4 lg:col-span-2 bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden">
          <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent flex items-center justify-between">
            <span className="text-[13px] font-bold tracking-wide text-foreground flex items-center gap-1.5">
              <Layers className="h-4 w-4 text-primary" /> Truth Layer
            </span>
          </div>
          <div className="p-4 space-y-1.5">
            <div className="flex items-center justify-between py-0.5">
              <span className="text-[10px] text-muted-foreground flex items-center gap-1.5">
                <CheckCircle className="h-2.5 w-2.5 text-verified" /> Verified
              </span>
              <span className="text-[13px] font-bold tabular-nums status-verified">{verifiedFindings.length}</span>
            </div>
            <div className="flex items-center justify-between py-0.5">
              <span className="text-[10px] text-muted-foreground flex items-center gap-1.5">
                <AlertTriangle className="h-2.5 w-2.5 text-tentative" /> Tentative
              </span>
              <span className="text-[13px] font-bold tabular-nums status-tentative">{unverifiedFindings.length}</span>
            </div>
            <div className="pt-1.5 mt-1 border-t border-border space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[10px] text-muted-foreground">Verification rate</span>
                <span className="text-[10px] font-semibold status-verified tabular-nums">
                  {normalizedFindings.length > 0 ? Math.round((verifiedFindings.length / normalizedFindings.length) * 100) : 0}%
                </span>
              </div>
              <div className="progress-bar">
                <div className="progress-fill bg-verified" style={{ width: `${normalizedFindings.length > 0 ? (verifiedFindings.length / normalizedFindings.length) * 100 : 0}%` }} />
              </div>
              <div className="flex items-center justify-between">
                <span className="text-[10px] text-muted-foreground">Evidence coverage</span>
                <span className="text-[10px] font-semibold text-foreground tabular-nums">{evidenceCoverage}%</span>
              </div>
              <div className="progress-bar">
                <div className="progress-fill bg-primary" style={{ width: `${evidenceCoverage}%` }} />
              </div>
            </div>
          </div>
        </div>

        {/* Risk Assessment */}
        <div className="col-span-4 lg:col-span-3 bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col">
          <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent flex items-center justify-between"><span className="text-[13px] font-bold tracking-wide text-foreground">Risk Assessment</span></div>
          <div className="p-4 flex-1 flex flex-col justify-center">
            <div className="flex items-center gap-6">
              
              {/* Modern SVG Donut Chart Score */}
              <div className="shrink-0 flex flex-col items-center gap-1.5">
                <div className="relative w-14 h-14 flex items-center justify-center">
                  <svg className="w-full h-full transform -rotate-90" viewBox="0 0 36 36">
                    {/* Track */}
                    <path className="text-muted/30" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="currentColor" strokeWidth="2.5" />
                    {/* Fill */}
                    <path className={cn("transition-all duration-1000", riskColor)} strokeDasharray={`${riskScore}, 100`} d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" />
                  </svg>
                  <div className="absolute inset-0 flex flex-col items-center justify-center">
                    <span className={cn("text-[17px] font-bold leading-none tracking-tight", riskColor)}>{riskScore}</span>
                  </div>
                </div>
                <span className="text-[9px] font-semibold text-muted-foreground/50 uppercase tracking-widest">Score</span>
              </div>

              {/* Modernized List with Indicators */}
              <div className="flex-1 space-y-2 text-[10.5px]">
                {[
                  {
                    label: "Attack Surface",
                    value: endpointsCount == null ? "Unavailable" : endpointsCount > 50 ? "High" : endpointsCount > 10 ? "Moderate" : "Low",
                    cls: endpointsCount == null ? "text-muted-foreground" : endpointsCount > 50 ? "text-risk" : "text-attention",
                  },
                  { label: "Exposure Level", value: riskScore < 50 ? "High" : riskScore < 80 ? "Moderate" : "Low", cls: riskColor },
                  { label: "Data Sensitivity", value: "Moderate", cls: "text-attention" },
                  { label: "Auth Weaknesses", value: authWeaknesses.toString(), cls: authWeaknesses > 0 ? "text-risk" : "text-muted-foreground", suffix: "found" },
                  { label: "FP Risk", value: `${100 - evidenceCoverage}%`, cls: evidenceCoverage < 50 ? "text-attention" : "status-verified", suffix: "score" },
                ].map(r => (
                  <div key={r.label} className="flex items-center justify-between group cursor-default">
                    <div className="flex items-center gap-2">
                      <span className={cn("flex items-center justify-center shrink-0", r.cls)}>
                        <span className="h-1.5 w-1.5 rounded-full bg-current opacity-40 group-hover:opacity-100 transition-opacity duration-300 drop-shadow-sm" />
                      </span>
                      <span className="text-muted-foreground/80 group-hover:text-foreground transition-colors duration-300">{r.label}</span>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <span className={cn("font-semibold tracking-wide drop-shadow-sm", r.cls)}>{r.value}</span>
                      {r.suffix && <span className="text-[9px] text-muted-foreground/40 font-medium uppercase tracking-wider">{r.suffix}</span>}
                    </div>
                  </div>
                ))}
              </div>
              
            </div>
          </div>
        </div>
      </div>

      {/* ══════════════════════════════════════
          ROW 2: Findings Table + Modules
          ══════════════════════════════════════ */}
      <div className="grid grid-cols-12 grid-rows-[1fr] gap-2" style={{ maxHeight: '520px' }}>

        {/* Priority Findings */}
        <div className="col-span-12 xl:col-span-8 bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg flex flex-col min-h-0 relative z-10">
          <div className="flex flex-col items-stretch gap-3 px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent shrink-0 rounded-t-md">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className="text-[13px] font-bold tracking-wide text-foreground">Priority Findings — Evidence Review</span>
                <span className="text-[10px] text-muted-foreground">{filteredPriorityFindings.length} findings</span>
              </div>
              <div className="flex items-center gap-2">
                <button onClick={() => exportFindingsAsCsv(filteredPriorityFindings)} aria-label="Export findings as CSV" className="flex items-center gap-1.5 px-2 py-1 bg-muted/20 border border-border rounded text-[10px] font-semibold text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors">
                  <Download className="h-3 w-3" /> CSV
                </button>
                <button onClick={() => exportFindingsAsJson(filteredPriorityFindings)} aria-label="Export findings as JSON" className="flex items-center gap-1.5 px-2 py-1 bg-muted/20 border border-border rounded text-[10px] font-semibold text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors">
                  <Download className="h-3 w-3" /> JSON
                </button>
              </div>
            </div>
            <FindingFilter onFilterChange={(f) => setFilterQuery(f.query)} />
          </div>
          {/* Grid Table */}
          <div className="overflow-x-auto overflow-y-auto flex-1 min-h-0">
            <table className="w-full text-left border-collapse" aria-label="Priority findings table">
              <thead className="sticky top-0 z-10">
                <tr className="text-[9px] font-semibold text-muted-foreground/70 uppercase tracking-widest bg-muted/20 border-b border-border">
                  <th className="px-3 py-1.5 w-[56px] font-semibold">Sev</th>
                  <th className="px-2 py-1.5 font-semibold">Finding</th>
                  <th className="px-2 py-1.5 w-[52px] text-center font-semibold">CVSS</th>
                  <th className="px-2 py-1.5 w-[72px] text-center font-semibold">Confidence</th>
                  <th className="px-2 py-1.5 w-[40px] text-center font-semibold">Evid</th>
                  <th className="px-2 py-1.5 w-[52px] text-center font-semibold">Status</th>
                </tr>
              </thead>
              <tbody>
                {filteredPriorityFindings.slice(0, 50).map((f) => {
                  const fp = fpRisk(f.confidence, f.verification);
                  return (
                    <tr key={f.id} className="border-b border-border/50 hover:bg-muted/20 transition-colors cursor-pointer group">
                      <td className="px-3 py-2 align-top">
                        <SeverityBadge severity={f.severity} />
                      </td>
                      <td className="px-2 py-2 min-w-0">
                        <p className="text-[11px] font-medium text-foreground truncate max-w-[360px] group-hover:text-primary transition-colors">{f.title}</p>
                        <p className="text-[9px] text-muted-foreground/50 font-mono truncate max-w-[360px]">{f.url} → {f.parameter}</p>
                      </td>
                      <td className="px-2 py-2 text-center align-top">
                        <span className={cn("text-[11px] font-bold tabular-nums block leading-tight",
                          (f.cvss ?? 0) >= 9 ? "sev-critical" : (f.cvss ?? 0) >= 7 ? "sev-high" : "text-foreground"
                        )}>{f.cvss ?? "—"}</span>
                        <span className="text-[8px] font-mono text-muted-foreground/30 block">{f.cwe.replace("CWE-", "")}</span>
                      </td>
                      <td className="px-2 py-2 text-center align-top">
                        <span className={cn("text-[10px] font-medium block leading-tight", confidenceColor(f.confidence))}>{confidenceLabel(f.confidence)}</span>
                        <span className={cn("text-[8px] block leading-tight", fp.cls)}>FP: {fp.label}</span>
                      </td>
                      <td className="px-2 py-2 text-center align-top">
                        <span className={cn(
                          "text-[11px] font-semibold tabular-nums",
                          f.evidence.length >= 2 ? "status-verified" : "text-muted-foreground"
                        )}>{f.evidence.length}</span>
                      </td>
                      <td className="px-2 py-2 text-center align-top">
                        <StatusBadge status={f.verification} icon={f.verification === "verified" ? CheckCircle : Eye} />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
            {filteredPriorityFindings.length > 50 && (
              <div className="py-2 text-center text-[10px] text-muted-foreground/50 border-t border-border/50 bg-muted/10">
                + {filteredPriorityFindings.length - 50} more findings. Use filters to view specific vulnerabilities.
              </div>
            )}
          </div>
        </div>

        {/* Detection Modules */}
        <div className="col-span-12 xl:col-span-4 bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col min-h-0">
          <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent flex items-center justify-between shrink-0">
            <span className="text-[13px] font-bold tracking-wide text-foreground flex items-center gap-2"><Shield className="h-4 w-4 text-primary" /> Detection Modules</span>
            <span className="text-[10px] text-muted-foreground">{Object.keys(DETECTOR_META).length} active</span>
          </div>
          <div className="overflow-x-auto overflow-y-auto flex-1 min-h-0">
            <table className="w-full text-left border-collapse" aria-label="Detection modules table">
              <thead>
                <tr className="text-[9px] font-semibold text-muted-foreground/70 uppercase tracking-widest bg-muted/20 border-b border-border">
                  <th className="px-3 py-1.5 font-semibold">Module</th>
                  <th className="px-2 py-1.5 w-[48px] text-center font-semibold">Type</th>
                  <th className="px-2 py-1.5 w-[48px] text-center font-semibold">Signals</th>
                  <th className="px-2 py-1.5 w-[44px] text-right font-semibold">Conf</th>
                </tr>
              </thead>
              <tbody>
                {(Object.entries(DETECTOR_META) as [DetectorType, { name: string; category: string; description: string }][])
                  .map(([id, meta]) => {
                    const sig = detectorSignals[id] ?? { signals: 0, confidence: 0 };
                    const findingCount = normalizedFindings.filter((f: { detector: string }) => f.detector === id).length;
                    return (
                      <tr key={id} className="border-b border-border/50 hover:bg-muted/20 transition-colors">
                        <td className="px-3 py-1.5">
                          <div className="flex items-center gap-2 min-w-0">
                            <span className={cn(
                              "h-1.5 w-1.5 rounded-full shrink-0",
                              findingCount > 0 ? "bg-attention" : "bg-muted-foreground/30"
                            )} />
                            <span className="text-[11px] font-medium text-foreground truncate">{meta.name}</span>
                            {findingCount > 0 && (
                              <span className="text-[9px] font-bold sev-badge sev-badge-medium">{findingCount}</span>
                            )}
                          </div>
                        </td>
                        <td className="px-2 py-1.5 text-center text-[9px] text-muted-foreground/60 uppercase tracking-wider">{meta.category.slice(0, 4)}</td>
                        <td className="px-2 py-1.5 text-center text-[10px] font-mono text-muted-foreground tabular-nums">{sig.signals}</td>
                        <td className={cn("px-2 py-1.5 text-right text-[10px] font-mono tabular-nums",
                          sig.confidence >= 90 ? "status-verified" : sig.confidence >= 80 ? "text-foreground" : "text-muted-foreground"
                        )}>{sig.confidence}%</td>
                      </tr>
                    );
                  })}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* ══════════════════════════════════════
          ROW 3: Distribution + Attack Surface + Event Timeline
          ══════════════════════════════════════ */}
      <div className="grid grid-cols-12 gap-2">

        {/* Severity Distribution */}
        <div className="col-span-12 lg:col-span-3 bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col">
          <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent"><span className="text-[13px] font-bold tracking-wide text-foreground">Severity Distribution</span></div>
          <div className="p-4 flex-1 flex flex-col py-3.5">
            {normalizedFindings.length > 0 && (
              <div className="flex flex-col h-full">
                {/* Unified Segmented Bar */}
                <div className="h-1.5 w-full bg-muted/20 flex overflow-hidden mb-4 shrink-0 rounded-sm">
                  {[
                    { count: criticalFindings.length, color: "bg-critical" },
                    { count: highFindings.length, color: "bg-high" },
                    { count: mediumFindings.length, color: "bg-medium" },
                    { count: lowFindings.length, color: "bg-low" },
                  ].map((item, i) => {
                    if (item.count === 0) return null;
                    const pct = (item.count / normalizedFindings.length) * 100;
                    return <div key={i} className={cn("h-full", item.color)} style={{ width: `${pct}%` }} />;
                  })}
                </div>

                {/* 2x2 Grid */}
                <div className="grid grid-cols-2 gap-2 flex-1">
                  {[
                    { label: "Critical", count: criticalFindings.length, total: normalizedFindings.length, color: "bg-critical" },
                    { label: "High", count: highFindings.length, total: normalizedFindings.length, color: "bg-high" },
                    { label: "Medium", count: mediumFindings.length, total: normalizedFindings.length, color: "bg-medium" },
                    { label: "Low", count: lowFindings.length, total: normalizedFindings.length, color: "bg-low" },
                  ].map(item => {
                    const percentage = item.total > 0 ? Math.round((item.count / item.total) * 100) : 0;
                    return (
                      <div key={item.label} className="flex flex-col justify-center p-3 bg-muted/5 border border-border/40 hover:bg-muted/10 hover:border-border transition-colors duration-200">
                        <div className="flex items-center gap-2 mb-2">
                          <span className={cn("h-1.5 w-1.5 rounded-full opacity-80", item.color)} />
                          <span className="text-[10px] font-semibold text-muted-foreground tracking-widest uppercase">{item.label}</span>
                        </div>
                        <div className="flex items-baseline justify-between mt-auto">
                          <span className={cn("font-mono text-[16px] font-bold tabular-nums", item.count > 0 ? "text-foreground" : "text-muted-foreground/40")}>
                            {item.count}
                          </span>
                          <span className="font-mono text-[10px] text-muted-foreground/60 font-medium">{percentage}%</span>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
            {normalizedFindings.length === 0 && (
              <div className="flex-1 flex items-center justify-center text-[11px] text-muted-foreground/50 font-mono">
                No findings to distribute
              </div>
            )}
          </div>
        </div>

        {/* Attack Surface */}
        <div className="col-span-12 lg:col-span-3 bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col">
          <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent"><span className="text-[13px] font-bold tracking-wide text-foreground">Attack Surface</span></div>
          <div className="p-4 flex-1 grid grid-cols-2 grid-rows-3 gap-2">
            {[
              { label: "Endpoints", value: endpointsCount?.toString() ?? "—", sub: "discovered" },
              { label: "Parameters", value: parametersCount?.toString() ?? "—", sub: "tested" },
              { label: "Auth Bounds", value: authBoundsCount?.toString() ?? "—", sub: "identified" },
              { label: "API Routes", value: apiRoutesCount?.toString() ?? "—", sub: "mapped" },
              { label: "Input Vectors", value: inputVectorsCount?.toString() ?? "—", sub: "analyzed" },
              { label: "Attack Paths", value: normalizedFindings.length.toString(), sub: "traced" },
            ].map(item => (
              <div key={item.label} className="flex flex-col justify-center p-3 bg-muted/5 border border-border/40 hover:bg-muted/10 hover:border-border transition-colors duration-200">
                <span className="text-[10px] uppercase tracking-widest text-muted-foreground font-semibold mb-1">{item.label}</span>
                <div className="flex items-baseline gap-1.5 mt-auto">
                  <span className="font-mono text-[16px] font-bold text-foreground">{item.value}</span>
                  <span className="text-[9px] text-muted-foreground/50 font-medium uppercase tracking-wider">{item.sub}</span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Scan Event Timeline — Premium Terminal */}
        <div className="col-span-12 lg:col-span-6 bg-[#050505] border border-border/40 border-t-2 border-t-primary/50 rounded-md shadow-lg overflow-hidden flex flex-col">
          <div className="px-4 py-3 border-b border-border/30 bg-gradient-to-r from-primary/10 to-transparent shrink-0 flex items-center justify-between">
            <span className="text-[13px] font-bold tracking-wide text-foreground flex items-center gap-2">
              <span className="flex items-center gap-1.5">
                {connectionStatus === "connected" ? (
                  <span className="relative flex h-2 w-2">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-60" />
                    <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-400 shadow-[0_0_6px_hsl(142_60%_50%/0.5)]" />
                  </span>
                ) : (
                  <span className="h-2 w-2 rounded-full bg-muted-foreground/30" />
                )}
                Event Stream
              </span>
              {connectionStatus === "connected" && (
                <span className="text-[8px] font-mono font-bold tracking-widest text-emerald-400/80 uppercase">Live</span>
              )}
            </span>
            <span className="flex items-center gap-2">
              <span className="text-[9px] font-mono text-muted-foreground/50 tabular-nums">{events.length} events</span>
            </span>
          </div>
          <div
            className="p-0 overflow-y-auto font-mono terminal-body"
          >
            {events.length > 0 ? (
              events.slice(0, 100).map((event, idx) => {
                const data = event.data as Record<string, unknown> | string | null;
                const dataObj = typeof data === "object" && data !== null ? data : null;
                let level = ((dataObj?.level as string) ?? event.type ?? "info").toLowerCase();
                const message = (dataObj?.message as string) ?? (typeof data === "string" ? data : JSON.stringify(data)) ?? "Event";

                // Enhanced Content-Based Color Parsing
                const msgLower = message.toLowerCase();
                if (level === "info" || level === "finding") {
                  if (msgLower.includes("critical") || msgLower.includes("vulnerable") || msgLower.includes("sql") || msgLower.includes("xss") || msgLower.includes("rce") || msgLower.includes("injection")) {
                    level = "critical";
                  } else if (msgLower.includes("high") || msgLower.includes("error") || msgLower.includes("failed") || msgLower.includes("timeout")) {
                    level = "error";
                  } else if (msgLower.includes("warning") || msgLower.includes("medium") || msgLower.includes("skip")) {
                    level = "warning";
                  } else if (msgLower.includes("success") || msgLower.includes("found") || msgLower.includes("verified") || msgLower.includes("ok")) {
                    level = "success";
                  }
                }

                const levelConfig: Record<string, { color: string; bg: string; glow: string }> = {
                  critical: { color: "hsl(0 85% 62%)", bg: "hsl(0 60% 15% / 0.15)", glow: "0 0 8px hsl(0 72% 51% / 0.2)" },
                  error:    { color: "hsl(0 85% 62%)", bg: "hsl(0 60% 15% / 0.15)", glow: "0 0 8px hsl(0 72% 51% / 0.2)" },
                  warning:  { color: "hsl(38 95% 60%)", bg: "hsl(38 60% 15% / 0.1)", glow: "none" },
                  success:  { color: "hsl(142 65% 52%)", bg: "hsl(142 40% 15% / 0.1)", glow: "none" },
                  complete: { color: "hsl(142 65% 52%)", bg: "hsl(142 40% 15% / 0.1)", glow: "none" },
                  info:     { color: "hsl(207 80% 62%)", bg: "transparent", glow: "none" },
                };
                const cfg = levelConfig[level] ?? { color: "hsl(215 15% 50%)", bg: "transparent", glow: "none" };

                return (
                  <div
                    key={event.id}
                    className="terminal-row items-start transition-colors duration-100 hover:!bg-white/[0.04] group"
                    style={{
                      background: cfg.bg,
                      boxShadow: cfg.glow,
                    }}
                  >
                    {/* Line number */}
                    <span
                      className="select-none tabular-nums text-right pr-2 terminal-line-num"
                    >
                      {idx + 1}
                    </span>
                    {/* Timestamp */}
                    <span
                      className="tabular-nums shrink-0 terminal-timestamp"
                    >
                      {new Date(event.timestamp).toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit" })}
                    </span>
                    {/* Level badge */}
                    <span
                      className="uppercase font-bold tracking-wider shrink-0"
                      style={{
                        color: cfg.color,
                        fontSize: "9px",
                        paddingTop: "2px",
                        textShadow: level === "critical" || level === "error" ? `0 0 6px ${cfg.color}` : "none",
                      }}
                    >
                      {level === "critical" ? "CRIT" : level.length > 5 ? level.slice(0, 5) : level}
                    </span>
                    {/* Message */}
                    <span
                      className={cn(
                        "truncate min-w-0",
                        level === "critical" || level === "error"
                          ? "terminal-msg-error"
                          : level === "success" || level === "complete"
                            ? "terminal-msg-success"
                            : level === "warning"
                              ? "text-amber-400"
                              : "terminal-msg-info"
                      )}
                    >
                      {message}
                    </span>
                  </div>
                );
              })
            ) : (
              <div className="flex flex-col items-center justify-center h-full gap-2 py-4">
                <div className="flex items-center gap-1.5">
                  <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground/20" />
                  <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground/15" />
                  <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground/10" />
                </div>
                <span className="text-[11px] text-muted-foreground/40 font-mono">Awaiting scan events…</span>
                <span className="text-[9px] text-muted-foreground/20 font-mono">Start a scan to begin streaming</span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
