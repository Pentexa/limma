"use client";

import { useGlobalFindings } from "@/entities/finding/model/use-findings";
import { useFinding } from "@/entities/finding/model/use-findings";
import { useAutoExploitFinding, useUpdateFindingStatus } from "@/entities/finding/model/use-verify-finding";
import { DETECTOR_META } from "@/entities/finding/model/types";
import { cn } from "@/shared/lib/utils";
import {
  ArrowLeft, CheckCircle, XCircle, Loader2, Globe, Code, FileCode,
  AlertTriangle, Link2, Copy, Shield, Clock,
} from "lucide-react";
import { useMemo, useState } from "react";
import Link from "next/link";
import { ExecutionLevelDialog } from "@/features/verify-finding/ui/ExecutionLevelDialog";

interface FindingDetailScreenProps {
  findingId: string;
}

function CopyBtn({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      className="text-muted-foreground/40 hover:text-foreground transition-colors p-1 rounded hover:bg-muted/30"
      onClick={() => { navigator.clipboard.writeText(text); setCopied(true); setTimeout(() => setCopied(false), 1500); }}
    >
      {copied ? <CheckCircle className="h-3.5 w-3.5 text-verified" /> : <Copy className="h-3.5 w-3.5" />}
    </button>
  );
}

export function FindingDetailScreen({ findingId }: FindingDetailScreenProps) {
  // Strategy 1: Try React Query cache first (pre-seeded by FindingDetailPanel)
  const { data: apiFinding, isLoading: apiLoading, error: apiError } = useFinding(findingId);

  // Strategy 2: Fallback — search the global findings list (already in memory)
  const { data: allFindings = [], isLoading: globalLoading } = useGlobalFindings();
  const cachedFinding = useMemo(
    () => allFindings.find(f => f.id === findingId),
    [allFindings, findingId]
  );

  const finding = apiFinding ?? cachedFinding;
  const isLoading = !finding && (apiLoading || globalLoading);

  const verifyMutation = useAutoExploitFinding();
  const updateStatusMutation = useUpdateFindingStatus();
  const [isVerifyDialogOpen, setIsVerifyDialogOpen] = useState(false);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full bg-[#030303]">
        <div className="flex items-center gap-2 text-primary animate-pulse">
          <Loader2 className="h-4 w-4 animate-spin" />
          <span className="text-[12px] font-mono font-bold tracking-wider">Loading finding…</span>
        </div>
      </div>
    );
  }

  if (!finding) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground bg-[#030303]">
        <AlertTriangle className="h-8 w-8 mb-3 opacity-40 text-risk" />
        <span className="text-[12px] font-bold">Finding not found</span>
        {apiError && <span className="text-[10px] text-muted-foreground/50 mt-1 font-mono">{apiError.message}</span>}
        <Link href="/active-detection" className="text-[11px] text-primary hover:underline mt-3">
          ← Back to Active Detection
        </Link>
      </div>
    );
  }

  const detectorName = DETECTOR_META[finding.detector]?.name ?? finding.detector;
  const detectorDesc = DETECTOR_META[finding.detector]?.description ?? "";

  return (
    <div className="flex flex-col h-full w-full overflow-hidden">
      {/* Breadcrumb */}
      <div className="shrink-0 flex items-center gap-2 px-4 py-2 border-b border-border/40 bg-[#050505]">
        <Link href="/active-detection" className="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-primary transition-colors">
          <ArrowLeft className="h-3 w-3" /> Active Detection
        </Link>
        <span className="text-[10px] text-muted-foreground/30">/</span>
        <span className="text-[10px] text-foreground font-bold truncate max-w-[400px]">{finding.title}</span>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4 bg-[#030303]">
        {/* Header card */}
        <div className="bg-[#080808] border border-border/30 rounded-md shadow-lg overflow-hidden">
          <div className="p-5 space-y-3">
            <div className="flex items-start justify-between gap-4">
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2 mb-2">
                  <span className={cn("sev-badge", `sev-badge-${finding.severity}`)}>{finding.severity}</span>
                  {finding.cvss != null && (
                    <span className={cn("text-[12px] font-bold font-mono tabular-nums",
                      finding.cvss >= 9 ? "sev-critical" : finding.cvss >= 7 ? "sev-high" : "text-foreground"
                    )}>CVSS {finding.cvss}</span>
                  )}
                  <span className={cn("text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded border",
                    finding.verification === "verified" ? "bg-verified/10 text-verified border-verified/20" :
                    finding.verification === "false_positive" ? "bg-risk/10 text-risk border-risk/20" : "bg-muted/10 text-muted-foreground border-border/20"
                  )}>
                    {finding.verification === "verified" ? "✓ Verified" :
                     finding.verification === "false_positive" ? "✗ False Positive" : "Unverified"}
                  </span>
                </div>
                <h1 className="text-[16px] font-bold text-foreground leading-snug">{finding.title}</h1>
                <p className="text-[12px] text-muted-foreground/80 mt-1.5 leading-relaxed">{finding.description}</p>
              </div>
              {/* Actions */}
              <div className="shrink-0 flex flex-col gap-2">
                <button
                  className={cn(
                    "flex items-center gap-1.5 px-4 py-2 rounded text-[11px] font-bold uppercase tracking-wider transition-all border active:scale-[0.98]",
                    finding.verification === "verified"
                      ? "bg-verified/10 text-verified border-verified/20 cursor-default"
                      : "bg-primary/10 text-primary border-primary/20 hover:bg-primary/20"
                  )}
                  disabled={finding.verification === "verified" || verifyMutation.isPending}
                  onClick={() => setIsVerifyDialogOpen(true)}
                >
                  {verifyMutation.isPending ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <CheckCircle className="h-3.5 w-3.5" />}
                  {finding.verification === "verified" ? "Verified" : "Verify"}
                </button>
                <button
                  className={cn(
                    "flex items-center gap-1.5 px-4 py-2 rounded text-[11px] font-bold uppercase tracking-wider border transition-all active:scale-[0.98]",
                    finding.verification === "false_positive"
                      ? "bg-risk/10 text-risk border-risk/20 cursor-default"
                      : "bg-muted/10 text-muted-foreground border-border/20 hover:bg-risk/10 hover:text-risk hover:border-risk/30"
                  )}
                  disabled={finding.verification === "false_positive" || updateStatusMutation.isPending}
                  onClick={() => updateStatusMutation.mutate({ findingId: finding.id, verified: false, falsePositive: true })}
                >
                  {updateStatusMutation.isPending ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <XCircle className="h-3.5 w-3.5" />}
                  {finding.verification === "false_positive" ? "Marked FP" : "False Positive"}
                </button>
              </div>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-12 gap-3">
          {/* Left: Technical details */}
          <div className="col-span-12 lg:col-span-8 space-y-3">
            {/* Target info */}
            <div className="bg-[#080808] border border-border/30 rounded-md shadow-lg overflow-hidden">
              <div className="px-4 py-3 border-b border-border/20 bg-gradient-to-r from-primary/10 to-transparent flex items-center gap-2">
                <Globe className="h-4 w-4 text-primary" />
                <span className="text-[13px] font-bold tracking-wide text-foreground">Target Information</span>
              </div>
              <div className="p-4 space-y-2.5">
                {[
                  { icon: Globe, label: "URL", value: finding.url, mono: true },
                  { icon: Code, label: "Parameter", value: finding.parameter || "—", mono: true },
                  { icon: Shield, label: "Method", value: finding.method || "GET", mono: true },
                  { icon: FileCode, label: "Detector", value: `${detectorName} — ${detectorDesc}`, mono: false },
                  { icon: AlertTriangle, label: "CWE", value: finding.cwe, mono: true },
                  { icon: Clock, label: "Discovered", value: finding.createdAt ? new Date(finding.createdAt).toLocaleString() : "—", mono: false },
                ].map((item) => (
                  <div key={item.label} className="flex items-center gap-3 text-[11px]">
                    <item.icon className="h-3 w-3 text-primary/60 shrink-0" />
                    <span className="text-muted-foreground/70 w-[80px] shrink-0 uppercase text-[9px] tracking-widest font-bold">{item.label}</span>
                    <span className={cn("text-foreground/90 truncate", item.mono && "font-mono")}>{item.value}</span>
                    {item.mono && item.value !== "—" && <CopyBtn text={item.value} />}
                  </div>
                ))}
              </div>
            </div>

            {/* Payload */}
            {finding.payload && (
              <div className="bg-[#080808] border border-border/30 rounded-md shadow-lg overflow-hidden">
                <div className="px-4 py-3 border-b border-border/20 bg-gradient-to-r from-primary/10 to-transparent flex items-center justify-between">
                  <span className="text-[13px] font-bold tracking-wide text-foreground">Payload</span>
                  <CopyBtn text={finding.payload} />
                </div>
                <pre className="text-[11px] font-mono bg-black/40 px-4 py-3 overflow-x-auto whitespace-pre-wrap break-all text-foreground/90 leading-relaxed max-h-[300px]">
                  {finding.payload}
                </pre>
              </div>
            )}

            {/* Evidence */}
            <div className="bg-[#080808] border border-border/30 rounded-md shadow-lg overflow-hidden">
              <div className="px-4 py-3 border-b border-border/20 bg-gradient-to-r from-primary/10 to-transparent">
                <span className="text-[13px] font-bold tracking-wide text-foreground">Evidence ({finding.evidence.length})</span>
              </div>
              <div className="p-4 space-y-2">
                {finding.evidence.length > 0 ? finding.evidence.map((ev, i) => (
                  <div key={i} className="relative group">
                    <pre className="text-[10px] font-mono bg-[#0c0c0c] border border-border/15 rounded px-3 py-2 overflow-x-auto whitespace-pre-wrap break-all text-muted-foreground/80 max-h-[200px] leading-relaxed">
                      {ev}
                    </pre>
                    <div className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <CopyBtn text={ev} />
                    </div>
                  </div>
                )) : (
                  <span className="text-[11px] text-muted-foreground/50 font-mono">No evidence collected</span>
                )}
              </div>
            </div>

            {/* Response */}
            {finding.response && (
              <div className="bg-[#080808] border border-border/30 rounded-md shadow-lg overflow-hidden">
                <div className="px-4 py-3 border-b border-border/20 bg-gradient-to-r from-primary/10 to-transparent">
                  <span className="text-[13px] font-bold tracking-wide text-foreground">Response</span>
                </div>
                <pre className="text-[10px] font-mono bg-black/40 px-4 py-3 overflow-x-auto whitespace-pre-wrap break-all text-foreground/60 max-h-[300px] leading-relaxed">
                  {finding.response}
                </pre>
              </div>
            )}
          </div>

          {/* Right: Meta */}
          <div className="col-span-12 lg:col-span-4 space-y-3">
            {/* Classification */}
            <div className="bg-[#080808] border border-border/30 rounded-md shadow-lg overflow-hidden">
              <div className="px-4 py-3 border-b border-border/20 bg-gradient-to-r from-primary/10 to-transparent">
                <span className="text-[13px] font-bold tracking-wide text-foreground">Classification</span>
              </div>
              <div className="p-4 space-y-3">
                {[
                  { label: "Severity", value: finding.severity, cls: `sev-${finding.severity}` },
                  { label: "Confidence", value: finding.confidence, cls: finding.confidence === "confirmed" ? "text-verified" : "text-foreground" },
                  { label: "Verification", value: finding.verification, cls: finding.verification === "verified" ? "text-verified" : "text-muted-foreground" },
                  { label: "Scan ID", value: finding.scanId?.slice(0, 12) ?? "—", cls: "text-muted-foreground font-mono" },
                ].map(item => (
                  <div key={item.label} className="flex items-center justify-between text-[11px]">
                    <span className="text-muted-foreground/60 uppercase text-[9px] tracking-widest font-bold">{item.label}</span>
                    <span className={cn("font-bold capitalize", item.cls)}>{item.value}</span>
                  </div>
                ))}
              </div>
            </div>

            {/* References */}
            {finding.references.length > 0 && (
              <div className="bg-[#080808] border border-border/30 rounded-md shadow-lg overflow-hidden">
                <div className="px-4 py-3 border-b border-border/20 bg-gradient-to-r from-primary/10 to-transparent">
                  <span className="text-[13px] font-bold tracking-wide text-foreground">References</span>
                </div>
                <div className="p-4 space-y-2">
                  {finding.references.map((ref, i) => (
                    <a key={i} href={ref} target="_blank" rel="noopener noreferrer"
                      className="flex items-center gap-1.5 text-[10px] text-primary hover:underline truncate">
                      <Link2 className="h-2.5 w-2.5 shrink-0" />{ref}
                    </a>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      <ExecutionLevelDialog 
        isOpen={isVerifyDialogOpen}
        onOpenChange={setIsVerifyDialogOpen}
        isPending={verifyMutation.isPending}
        onConfirm={(level) => {
          verifyMutation.mutate({ findingId: finding.id, execution_level: level });
          setIsVerifyDialogOpen(false);
        }}
      />
    </div>
  );
}
