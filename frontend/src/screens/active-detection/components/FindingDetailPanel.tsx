"use client";

import { cn } from "@/shared/lib/utils";
import type { Finding } from "@/entities/finding/model/types";
import { DETECTOR_META } from "@/entities/finding/model/types";
import { useVerifyFinding, useUpdateFindingStatus } from "@/entities/finding/model/use-verify-finding";
import { findingKeys } from "@/entities/finding/model/use-findings";
import { useQueryClient } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import {
  CheckCircle, XCircle, ShieldAlert, ExternalLink, Copy, Loader2, X,
  Globe, Code, AlertTriangle, FileCode, Link2
} from "lucide-react";
import { useState } from "react";

interface FindingDetailPanelProps {
  finding: Finding | null;
  onClose: () => void;
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      className="text-muted-foreground/40 hover:text-primary transition-colors p-1"
      onClick={() => { navigator.clipboard.writeText(text); setCopied(true); setTimeout(() => setCopied(false), 1500); }}
      title="Copy"
    >
      {copied ? <CheckCircle className="h-4 w-4 text-emerald-400" /> : <Copy className="h-4 w-4" />}
    </button>
  );
}

export function FindingDetailPanel({ finding, onClose }: FindingDetailPanelProps) {
  const verifyMutation = useVerifyFinding();
  const updateStatusMutation = useUpdateFindingStatus();
  const queryClient = useQueryClient();
  const router = useRouter();

  if (!finding) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground/30 py-12 px-6 text-center">
        <div className="h-16 w-16 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-4">
          <ShieldAlert className="h-7 w-7 opacity-50" />
        </div>
        <span className="text-[14px] font-medium text-muted-foreground/50">Select a finding</span>
        <span className="text-[12px] mt-1 text-muted-foreground/30">Choose a finding from the list to inspect</span>
      </div>
    );
  }

  const detectorName = DETECTOR_META[finding.detector]?.name ?? finding.detector;

  function handleFullDetail() {
    if (!finding) return;
    queryClient.setQueryData(findingKeys.detail(finding.id), finding);
    router.push(`/findings/${finding.id}`);
  }

  return (
    <div className="flex flex-col h-full overflow-hidden bg-transparent">
      {/* Header */}
      <div className="flex items-start justify-between gap-3 px-4 py-3 border-b border-white/[0.06] shrink-0">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2 mb-1.5">
            <span className={cn("sev-badge text-[9px] font-semibold", `sev-badge-${finding.severity}`)}>{finding.severity}</span>
            {finding.cvss != null && (
              <span className={cn("text-[11px] font-semibold font-mono tabular-nums",
                finding.cvss >= 9 ? "text-red-400" : finding.cvss >= 7 ? "text-orange-400" : "text-foreground/60"
              )}>CVSS {finding.cvss}</span>
            )}
          </div>
          <h3 className="text-[13px] font-semibold text-foreground leading-snug">{finding.title}</h3>
        </div>
        <button onClick={onClose} className="text-muted-foreground/40 hover:text-foreground hover:bg-white/[0.04] p-1 rounded transition-colors shrink-0">
          <X className="h-3.5 w-3.5" />
        </button>
      </div>

      {/* Scrollable content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Meta Info */}
        <div className="rounded-xl bg-white/[0.03] border border-white/[0.06] p-3 space-y-2">
          <div className="flex items-start gap-2 text-[11px]">
            <Globe className="h-3.5 w-3.5 text-primary/60 shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0 pr-2">
              <span className="font-mono text-foreground/80 break-all leading-relaxed">{finding.url}</span>
            </div>
            <CopyButton text={finding.url} />
          </div>
          <div className="flex items-start gap-2 text-[11px]">
            <Code className="h-3.5 w-3.5 text-primary/60 shrink-0 mt-0.5" />
            <span className="text-muted-foreground/60 font-medium">Parameter:</span>
            <span className="font-mono text-foreground/80 break-all">{finding.parameter || "—"}</span>
          </div>
          <div className="flex items-center gap-2 text-[11px]">
            <FileCode className="h-3.5 w-3.5 text-primary/60 shrink-0" />
            <span className="text-muted-foreground/60 font-medium">Detector:</span>
            <span className="text-foreground/80">{detectorName}</span>
          </div>
          <div className="flex items-center gap-2 text-[11px]">
            <AlertTriangle className="h-3.5 w-3.5 text-primary/60 shrink-0" />
            <span className="text-muted-foreground/60 font-medium">CWE:</span>
            <span className="font-mono text-foreground/80">{finding.cwe}</span>
          </div>
        </div>

        {/* Description */}
        <div>
          <span className="text-[11px] font-semibold text-foreground/80 mb-1.5 block">Description</span>
          <p className="text-[11px] text-muted-foreground/70 leading-relaxed bg-white/[0.02] border border-white/[0.04] rounded-xl p-3">{finding.description}</p>
        </div>

        {/* Payload */}
        {finding.payload && (
          <div className="rounded-xl bg-white/[0.03] border border-white/[0.06] overflow-hidden">
            <div className="flex items-center justify-between px-3 py-2 border-b border-white/[0.04]">
              <div className="flex items-center gap-1.5">
                <Code className="h-3.5 w-3.5 text-primary/70" />
                <span className="text-[11px] font-semibold text-foreground/80">Payload</span>
              </div>
              <CopyButton text={finding.payload} />
            </div>
            <pre className="text-[10px] font-mono bg-black/20 px-3 py-2.5 overflow-x-auto text-foreground/70 whitespace-pre-wrap break-all leading-relaxed">
              {finding.payload}
            </pre>
          </div>
        )}

        {/* Evidence */}
        {finding.evidence.length > 0 && (
          <div>
            <span className="text-[11px] font-semibold text-foreground/80 mb-1.5 block flex items-center gap-1.5">
              Evidence <span className="text-[9px] text-muted-foreground/40 font-mono">{finding.evidence.length}</span>
            </span>
            <div className="space-y-1.5">
              {finding.evidence.map((ev, i) => (
                <pre key={i} className="text-[9px] font-mono bg-white/[0.03] border border-white/[0.06] rounded-xl px-3 py-2 overflow-x-auto text-foreground/60 whitespace-pre-wrap break-all max-h-[140px]">
                  {ev}
                </pre>
              ))}
            </div>
          </div>
        )}

        {/* References */}
        {finding.references.length > 0 && (
          <div>
            <span className="text-[11px] font-semibold text-foreground/80 mb-1.5 block">References</span>
            <div className="space-y-1">
              {finding.references.map((ref, i) => (
                <a key={i} href={ref} target="_blank" rel="noopener noreferrer"
                  className="flex items-center gap-1.5 text-[10px] text-primary/80 hover:text-primary transition-colors truncate font-mono py-0.5">
                  <Link2 className="h-3 w-3 shrink-0 opacity-70" />{ref}
                </a>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="shrink-0 border-t border-white/[0.06] px-4 py-3 flex items-center gap-2">
        <button
          className={cn(
            "flex items-center gap-1.5 px-3 py-2 rounded-lg text-[11px] font-semibold transition-all duration-200 active:scale-[0.97]",
            finding.verification === "verified"
              ? "bg-emerald-500/[0.08] text-emerald-400 border border-emerald-500/20 cursor-default"
              : "bg-primary/10 text-primary border border-primary/20 hover:bg-primary/20"
          )}
          disabled={finding.verification === "verified" || verifyMutation.isPending}
          onClick={() => verifyMutation.mutate(finding.id)}
        >
          {verifyMutation.isPending ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <CheckCircle className="h-3.5 w-3.5" />}
          {finding.verification === "verified" ? "Verified" : "Verify Finding"}
        </button>

        <button
          className={cn(
            "flex items-center gap-1.5 px-3 py-2 rounded-lg text-[11px] font-semibold transition-all duration-200 active:scale-[0.97]",
            finding.verification === "false_positive"
              ? "bg-amber-500/[0.08] text-amber-400 border border-amber-500/20 cursor-default"
              : "bg-white/[0.04] text-muted-foreground/70 border border-white/[0.08] hover:bg-amber-500/10 hover:text-amber-400 hover:border-amber-500/20"
          )}
          disabled={finding.verification === "false_positive" || updateStatusMutation.isPending}
          onClick={() => updateStatusMutation.mutate({ findingId: finding.id, verified: false, falsePositive: true })}
        >
          {updateStatusMutation.isPending ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <XCircle className="h-3.5 w-3.5" />}
          {finding.verification === "false_positive" ? "Marked FP" : "False Pos"}
        </button>

        <button
          onClick={handleFullDetail}
          className="ml-auto flex items-center gap-1.5 text-[10px] font-medium text-muted-foreground/60 hover:text-primary transition-colors cursor-pointer px-1 py-1"
        >
          Full Detail <ExternalLink className="h-3 w-3" />
        </button>
      </div>
    </div>
  );
}
