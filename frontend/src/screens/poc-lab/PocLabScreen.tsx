"use client";

import { useState, useMemo } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { cn } from "@/shared/lib/utils";
import { useGlobalFindings, findingKeys } from "@/entities/finding/model/use-findings";
import { useAutoExploitFinding } from "@/entities/finding/model/use-verify-finding";
import { generatePocForFinding, verifyExploit, downloadPoc } from "@/features/blind-scan/api/blind-scan-api";
import { DETECTOR_META } from "@/entities/finding/model/types";
import type { Finding } from "@/entities/finding/model/types";
import { compareBySeverity } from "@/shared/config/priority";
import {
  FlaskConical, Loader2, Play, AlertTriangle, Code, Download, Cpu,
  CheckCircle, XCircle, ChevronRight, Copy, Shield,
} from "lucide-react";
import { ExecutionLevelDialog } from "@/features/verify-finding/ui/ExecutionLevelDialog";

function CopyBtn({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      className="text-muted-foreground/40 hover:text-primary transition-colors"
      onClick={(e) => { e.stopPropagation(); navigator.clipboard.writeText(text); setCopied(true); setTimeout(() => setCopied(false), 1200); }}
    >
      {copied ? <CheckCircle className="h-3.5 w-3.5 text-emerald-400" /> : <Copy className="h-3.5 w-3.5" />}
    </button>
  );
}

export function PocLabScreen() {
  const queryClient = useQueryClient();
  const { data: findings = [], isLoading } = useGlobalFindings();
  const autoExploitMutation = useAutoExploitFinding();
  const [selectedFinding, setSelectedFinding] = useState<Finding | null>(null);
  const [pocLoading, setPocLoading] = useState(false);
  const [pocResult, setPocResult] = useState<Record<string, unknown> | null>(null);
  const [pocError, setPocError] = useState<string | null>(null);
  const [exploitLoading, setExploitLoading] = useState(false);
  const [exploitResult, setExploitResult] = useState<Record<string, unknown> | null>(null);
  const [downloadLoading, setDownloadLoading] = useState(false);
  const [isVerifyDialogOpen, setIsVerifyDialogOpen] = useState(false);

  async function handleGeneratePoc(findingId: import("@/shared/types/common").FindingId) {
    setPocLoading(true);
    setPocError(null);
    setPocResult(null);
    try {
      const result = await generatePocForFinding(findingId);
      setPocResult(result);
      queryClient.invalidateQueries({ queryKey: findingKeys.all });
    } catch {
      try {
        const { generatePoc } = await import("@/features/blind-scan/api/blind-scan-api");
        const result = await generatePoc({ finding_id: findingId });
        setPocResult(result);
      } catch (err2) {
        setPocError(err2 instanceof Error ? err2.message : "PoC generation failed — finding may not be in the blind detection database.");
      }
    } finally {
      setPocLoading(false);
    }
  }

  async function handleVerifyExploit(pocId: string) {
    setExploitLoading(true);
    try {
      if (!selectedFinding?.url) throw new Error("No target URL for finding");
      const result = await verifyExploit({ poc_id: pocId, target_url: selectedFinding.url });
      setExploitResult(result);
    } catch (err) {
      setExploitResult({ error: err instanceof Error ? err.message : "Exploit verification failed" } as Record<string, unknown>);
    } finally { setExploitLoading(false); }
  }

  async function handleDownloadPoc(pocId: string) {
    setDownloadLoading(true);
    try {
      const result = await downloadPoc(pocId);
      const code = String(result.code ?? "");
      const blob = new Blob([code], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `poc_${pocId.slice(0, 8)}.txt`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      alert(err instanceof Error ? err.message : "Download failed");
    } finally { setDownloadLoading(false); }
  }

  const pocCandidates = useMemo(() =>
    findings
      .filter(f => f.verification === "verified" || f.severity === "critical" || f.severity === "high")
      .sort(compareBySeverity),
    [findings]
  );

  const allFindings = useMemo(() =>
    [...findings].sort(compareBySeverity),
    [findings]
  );

  const displayList = pocCandidates.length > 0 ? pocCandidates : allFindings;

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden bg-[#0a0a0c]">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-5 py-3 border-b border-white/[0.06]">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center h-7 w-7 rounded-lg bg-primary/10 border border-primary/20">
            <FlaskConical className="h-3.5 w-3.5 text-primary" />
          </div>
          <div>
            <h2 className="text-sm font-semibold text-foreground">PoC Lab</h2>
            <p className="text-[11px] text-muted-foreground/60">
              {isLoading ? "Loading…" : `${pocCandidates.length} candidates · ${displayList.length} total`}
            </p>
          </div>
        </div>
        {isLoading && (
          <Loader2 className="h-4 w-4 animate-spin text-primary" />
        )}
      </div>

      <div className="flex-1 flex min-h-0 overflow-hidden">
        {/* Left: Finding list */}
        <div className="shrink-0 w-[320px] border-r border-white/[0.06] overflow-y-auto">
          {isLoading ? (
            <div className="flex items-center justify-center h-full">
              <div className="flex flex-col items-center gap-2 text-muted-foreground/50">
                <Loader2 className="h-5 w-5 animate-spin text-primary" />
                <span className="text-xs">Loading findings…</span>
              </div>
            </div>
          ) : displayList.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground/40 px-6 text-center">
              <FlaskConical className="h-10 w-10 mb-3 opacity-30" />
              <span className="text-[13px] font-medium">No findings available</span>
              <span className="text-[11px] mt-1">Run an active scan to generate findings for PoC generation.</span>
            </div>
          ) : (
            <div className="divide-y divide-white/[0.04]">
              {displayList.map((f) => {
                const isSelected = selectedFinding?.id === f.id;
                return (
                  <div
                    key={f.id}
                    className={cn(
                      "px-4 py-3.5 cursor-pointer transition-all duration-150 group",
                      isSelected
                        ? "bg-primary/[0.06] border-l-2 border-l-primary"
                        : "hover:bg-white/[0.02] border-l-2 border-l-transparent"
                    )}
                    onClick={() => setSelectedFinding(f)}
                  >
                    <div className="flex items-center gap-2 mb-1.5">
                      <span className={cn("sev-badge text-[9px] font-semibold", `sev-badge-${f.severity}`)}>{f.severity}</span>
                      {f.verification === "verified" && (
                        <span className="flex items-center gap-1 text-[10px] text-emerald-400 font-medium">
                          <CheckCircle className="h-3 w-3" /> Verified
                        </span>
                      )}
                    </div>
                    <div className="flex items-center justify-between gap-2">
                      <div className="min-w-0 flex-1">
                        <p className={cn("text-[13px] font-medium truncate transition-colors",
                          isSelected ? "text-primary" : "text-foreground/90 group-hover:text-foreground"
                        )}>{f.title}</p>
                        <p className="text-[10px] text-muted-foreground/40 font-mono truncate mt-0.5">{f.url}</p>
                      </div>
                      <ChevronRight className={cn("h-4 w-4 shrink-0 transition-colors",
                        isSelected ? "text-primary" : "text-muted-foreground/15 group-hover:text-muted-foreground/40"
                      )} />
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Right: PoC viewer */}
        <div className="flex-1 min-w-0 overflow-y-auto">
          {!selectedFinding ? (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground/30 px-6 text-center">
              <div className="h-16 w-16 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-4">
                <FlaskConical className="h-7 w-7 opacity-50" />
              </div>
              <span className="text-[14px] font-medium text-muted-foreground/50">Select a finding</span>
              <span className="text-[12px] mt-1 text-muted-foreground/30">Choose a finding from the list to generate PoC</span>
            </div>
          ) : (
            <div className="p-5 space-y-5 max-w-3xl">
              {/* Finding header card */}
              <div className="rounded-xl bg-white/[0.03] border border-white/[0.06] p-5">
                <div className="flex items-start justify-between gap-4 mb-3">
                  <div className="flex items-center gap-2.5">
                    <span className={cn("sev-badge text-[9px] font-semibold", `sev-badge-${selectedFinding.severity}`)}>{selectedFinding.severity}</span>
                    {selectedFinding.cvss != null && (
                      <span className={cn("text-[12px] font-semibold font-mono tabular-nums",
                        selectedFinding.cvss >= 9 ? "text-red-400" : selectedFinding.cvss >= 7 ? "text-orange-400" : "text-foreground/60"
                      )}>CVSS {selectedFinding.cvss}</span>
                    )}
                    <span className="text-[11px] text-muted-foreground/50 px-2 py-0.5 rounded-md bg-white/[0.04]">
                      {DETECTOR_META[selectedFinding.detector]?.name ?? selectedFinding.detector}
                    </span>
                  </div>
                </div>
                <h3 className="text-[16px] font-semibold text-foreground leading-snug mb-2">{selectedFinding.title}</h3>
                <p className="text-[12px] text-muted-foreground/60 leading-relaxed">{selectedFinding.description}</p>
              </div>

              {/* Payload section */}
              <div className="rounded-xl bg-white/[0.03] border border-white/[0.06] overflow-hidden">
                <div className="flex items-center justify-between px-4 py-3 border-b border-white/[0.04]">
                  <div className="flex items-center gap-2">
                    <Code className="h-4 w-4 text-primary/70" />
                    <span className="text-[12px] font-semibold text-foreground/80">Payload</span>
                  </div>
                  {selectedFinding.payload && <CopyBtn text={selectedFinding.payload} />}
                </div>
                {selectedFinding.payload ? (
                  <pre className="text-[11px] font-mono px-4 py-3.5 overflow-x-auto whitespace-pre-wrap break-all text-foreground/70 leading-relaxed bg-black/20">
                    {selectedFinding.payload}
                  </pre>
                ) : (
                  <div className="px-4 py-8 text-center text-[12px] text-muted-foreground/30">No payload data available</div>
                )}
              </div>

              {/* Evidence section */}
              {selectedFinding.evidence.length > 0 && (
                <div className="rounded-xl bg-white/[0.03] border border-white/[0.06] overflow-hidden">
                  <div className="flex items-center gap-2 px-4 py-3 border-b border-white/[0.04]">
                    <Shield className="h-4 w-4 text-primary/70" />
                    <span className="text-[12px] font-semibold text-foreground/80">Evidence</span>
                    <span className="text-[10px] text-muted-foreground/40 font-mono ml-1">{selectedFinding.evidence.length}</span>
                  </div>
                  <div className="p-3 space-y-2">
                    {selectedFinding.evidence.map((ev, i) => (
                      <pre key={i} className="text-[10px] font-mono bg-black/20 border border-white/[0.04] rounded-lg px-3.5 py-2.5 overflow-x-auto whitespace-pre-wrap break-all text-foreground/50 max-h-[140px]">
                        {ev}
                      </pre>
                    ))}
                  </div>
                </div>
              )}

              {/* Action buttons */}
              <div className="flex flex-wrap items-center gap-2.5">
                <button
                  className={cn(
                    "flex items-center gap-2 px-4 py-2.5 rounded-lg text-[12px] font-semibold transition-all duration-200 active:scale-[0.97]",
                    "bg-primary/10 text-primary hover:bg-primary/20 border border-primary/20"
                  )}
                  disabled={autoExploitMutation.isPending}
                  onClick={() => setIsVerifyDialogOpen(true)}
                >
                  {autoExploitMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Play className="h-4 w-4" />}
                  Auto-Exploit
                </button>

                <button
                  className="flex items-center gap-2 px-4 py-2.5 rounded-lg text-[12px] font-semibold bg-amber-500/10 text-amber-400 hover:bg-amber-500/20 border border-amber-500/20 transition-all duration-200 active:scale-[0.97] disabled:opacity-50"
                  disabled={pocLoading}
                  onClick={() => handleGeneratePoc(selectedFinding.id)}
                >
                  {pocLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Code className="h-4 w-4" />}
                  Generate PoC
                </button>

                {pocResult?.id != null && (
                  <>
                    <button
                      className="flex items-center gap-2 px-4 py-2.5 rounded-lg text-[12px] font-semibold bg-red-500/10 text-red-400 hover:bg-red-500/20 border border-red-500/20 transition-all duration-200 active:scale-[0.97] disabled:opacity-50"
                      disabled={exploitLoading}
                      onClick={() => handleVerifyExploit(String(pocResult.id))}
                    >
                      {exploitLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Cpu className="h-4 w-4" />}
                      Sandbox Verify
                    </button>
                    <button
                      className="flex items-center gap-2 px-4 py-2.5 rounded-lg text-[12px] font-semibold bg-white/[0.04] text-muted-foreground/70 hover:bg-white/[0.08] hover:text-foreground border border-white/[0.08] transition-all duration-200 active:scale-[0.97] disabled:opacity-50"
                      disabled={downloadLoading}
                      onClick={() => handleDownloadPoc(String(pocResult.id))}
                    >
                      {downloadLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Download className="h-4 w-4" />}
                      Download
                    </button>
                  </>
                )}
              </div>

              {/* Auto-Exploit Result */}
              {autoExploitMutation.data && (
                <div className={cn("rounded-xl border bg-white/[0.03] overflow-hidden",
                  autoExploitMutation.data?.success
                    ? "border-emerald-500/30 shadow-[0_0_15px_var(--emerald-500)/0.05]"
                    : "border-red-500/30 shadow-[0_0_15px_var(--red-500)/0.05]"
                )}>
                  <div className="px-4 py-3 flex items-center justify-between">
                    <div className="flex items-center gap-2.5">
                      {autoExploitMutation.data?.success
                        ? <CheckCircle className="h-4 w-4 text-emerald-400" />
                        : <XCircle className="h-4 w-4 text-red-400" />}
                      <span className={cn("text-[13px] font-semibold",
                        autoExploitMutation.data?.success ? "text-emerald-400" : "text-red-400"
                      )}>
                        {autoExploitMutation.data?.success ? "Exploit Successful" : "Exploit Failed"}
                      </span>
                    </div>
                    <span className="text-[11px] font-mono text-muted-foreground/40">
                      {autoExploitMutation.data?.execution_time_ms ?? 0}ms
                    </span>
                  </div>
                  {autoExploitMutation.data?.output && (
                    <pre className="px-4 py-3 border-t border-white/[0.04] text-[11px] font-mono text-foreground/60 whitespace-pre-wrap break-all bg-black/10">
                      {autoExploitMutation.data.output}
                    </pre>
                  )}
                </div>
              )}

              {/* PoC Error */}
              {pocError && (
                <div className="flex items-start gap-2.5 text-[12px] text-red-400 bg-white/[0.03] border border-red-500/30 rounded-xl px-4 py-3">
                  <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
                  <span className="leading-relaxed">{pocError}</span>
                </div>
              )}

              {/* PoC Result */}
              {pocResult && (
                <div className="rounded-xl bg-white/[0.03] border border-white/[0.06] overflow-hidden">
                  <div className="flex items-center justify-between px-4 py-3 border-b border-white/[0.04]">
                    <div className="flex items-center gap-2">
                      <Code className="h-4 w-4 text-amber-400" />
                      <span className="text-[13px] font-semibold text-amber-400">Generated PoC</span>
                    </div>
                    <CopyBtn text={pocResult.code ? String(pocResult.code) : JSON.stringify(pocResult, null, 2)} />
                  </div>
                  <pre className="text-[11px] font-mono px-4 py-3.5 overflow-x-auto whitespace-pre-wrap text-foreground/70 leading-relaxed bg-black/10">
                    {pocResult.code ? String(pocResult.code) : JSON.stringify(pocResult, null, 2)}
                  </pre>
                </div>
              )}

              {/* Exploit Verification Result */}
              {exploitResult && (
                <div className="rounded-xl bg-white/[0.03] border border-white/[0.06] overflow-hidden">
                  <div className="flex items-center gap-2 px-4 py-3 border-b border-white/[0.04]">
                    <Cpu className="h-4 w-4 text-red-400" />
                    <span className="text-[13px] font-semibold text-red-400">Sandbox Verification</span>
                  </div>
                  <pre className="text-[11px] font-mono px-4 py-3.5 overflow-x-auto whitespace-pre-wrap text-foreground/70 leading-relaxed bg-black/10">
                    {JSON.stringify(exploitResult, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
      
      {selectedFinding && (
        <ExecutionLevelDialog 
          isOpen={isVerifyDialogOpen}
          onOpenChange={setIsVerifyDialogOpen}
          isPending={autoExploitMutation.isPending}
          onConfirm={(level) => {
            autoExploitMutation.mutate({ findingId: selectedFinding.id, execution_level: level });
            setIsVerifyDialogOpen(false);
          }}
        />
      )}
    </div>
  );
}
