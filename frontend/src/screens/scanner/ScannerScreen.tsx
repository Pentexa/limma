"use client";

import { useState, useEffect } from "react";
import { cn } from "@/shared/lib/utils";
import { useSearchParams } from "next/navigation";
import { useScans } from "@/entities/scan/model/use-scans";
import { useScanHistory } from "@/entities/scan/model/use-scan-history";
import {
  useAnalyzeWebsite,
  useInvestigateServer,
  useDiscoverApis,
  useCollectServices,
  useAuditSecurity,
  useMapForms,
  useGenerateMasterReport
} from "@/entities/intelligence/model/use-intelligence";
import { IntelModuleTab } from "./components/IntelModuleTab";
import { startScanStream, getCurrentStreamTarget } from "@/features/stream-scan-events/model/scan-stream-manager";
import { ServerInfoPanel } from "./components/ServerInfoPanel";
import { EndpointTable } from "./components/EndpointTable";
import { ServiceGrid } from "./components/ServiceGrid";
import { TechStackPanel } from "./components/TechStackPanel";
import { AnalyzePanel } from "./components/AnalyzePanel";
import { SecurityAuditPanel } from "./components/SecurityAuditPanel";
import { FormMappingPanel } from "./components/FormMappingPanel";
import { AttackChainsPanel } from "./components/AttackChainsPanel";
import {
  Radar, Search, Fingerprint, Network, Server, ShieldCheck, FileInput,
  Loader2, BrainCircuit
} from "lucide-react";
import { EMPTY_SCAN } from "@/entities/scan/model/constants";

type Tab = "analyze" | "investigate" | "discover" | "services" | "audit" | "forms" | "intelligence";

const TABS: { id: Tab; label: string; icon: React.ElementType }[] = [
  { id: "analyze", label: "Analyze", icon: Search },
  { id: "investigate", label: "Investigate", icon: Fingerprint },
  { id: "discover", label: "API Discovery", icon: Network },
  { id: "services", label: "Services", icon: Server },
  { id: "audit", label: "Security Audit", icon: ShieldCheck },
  { id: "forms", label: "Form Map", icon: FileInput },
  { id: "intelligence", label: "Attack Intelligence", icon: BrainCircuit },
];



export function ScannerScreen() {
  const [activeTab, setActiveTab] = useState<Tab>("analyze");
  const searchParams = useSearchParams();
  const scanId = searchParams.get("scanId");

  const { data: activeScans = [], isLoading: isActiveLoading } = useScans();
  const { data: historyScans = [], isLoading: isHistoryLoading } = useScanHistory();
  
  const allScans = [...activeScans, ...historyScans];
  
  const activeScan = scanId 
    ? allScans.find((s) => String(s.id) === scanId) ?? EMPTY_SCAN
    : activeScans.find((s) => s.status === "running") ?? activeScans[0] ?? EMPTY_SCAN;

  const isLoading = isActiveLoading || isHistoryLoading;
  const targetUrl = activeScan?.targetUrl !== "—" ? activeScan?.targetUrl : null;

  // SSE catch-up: ensure stream is connected when scanner is viewed during active scan
  useEffect(() => {
    if (activeScan.status === "running" && targetUrl && !getCurrentStreamTarget()) {
      startScanStream(targetUrl);
    }
  }, [activeScan.status, targetUrl]);

  const analyzeMut = useAnalyzeWebsite();
  const investigateMut = useInvestigateServer();
  const discoverMut = useDiscoverApis();
  const servicesMut = useCollectServices();
  const auditMut = useAuditSecurity();
  const formsMut = useMapForms();
  const masterReportMut = useGenerateMasterReport();

  /* Compute counts for tab badges */
  const counts: Record<Tab, number> = {
    analyze: 0,
    investigate: 0,
    discover: 0,
    services: 0,
    audit: 0,
    forms: 0,
    intelligence: 0,
  };

  // Update counts based on mutation results
  if (analyzeMut.data) counts.analyze = Object.keys(analyzeMut.data).length;
  if (investigateMut.data) counts.investigate = Object.keys(investigateMut.data).length;
  if (discoverMut.data) counts.discover = Object.keys(discoverMut.data).length;
  if (servicesMut.data) counts.services = Object.keys(servicesMut.data).length;
  if (auditMut.data) counts.audit = Object.keys(auditMut.data).length;
  if (formsMut.data) counts.forms = Object.keys(formsMut.data).length;
  if (masterReportMut.data?.normalized_audit?.attack_paths) counts.intelligence = masterReportMut.data.normalized_audit.attack_paths.length;

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden bg-[#0a0a0c]">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-5 py-3 border-b border-white/[0.06]">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center h-7 w-7 rounded-lg bg-primary/10 border border-primary/20">
            <Radar className="h-3.5 w-3.5 text-primary" />
          </div>
          <div>
            <h2 className="text-sm font-semibold text-foreground">Scanner</h2>
            {targetUrl && (
              <p className="text-[11px] font-mono text-muted-foreground/60 truncate max-w-[300px]">
                {targetUrl}
              </p>
            )}
          </div>
        </div>
        {isLoading && (
          <div className="flex items-center gap-1.5 text-[11px] font-medium text-primary animate-pulse">
            <Loader2 className="h-4 w-4 animate-spin" /> Loading data…
          </div>
        )}
      </div>

      {/* Tab bar */}
      <div className="shrink-0 flex items-center gap-2 border-b border-white/[0.06] px-5 py-2.5 overflow-x-auto">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            className={cn(
              "flex items-center gap-2 px-3 py-1.5 text-[12px] font-semibold rounded-lg transition-all duration-200 whitespace-nowrap",
              activeTab === tab.id
                ? "bg-primary/[0.08] text-primary"
                : "bg-transparent text-muted-foreground/60 hover:text-foreground hover:bg-white/[0.02]"
            )}
            onClick={() => setActiveTab(tab.id)}
          >
            <tab.icon className={cn("h-4 w-4 shrink-0 opacity-80")} />
            {tab.label}
            {counts[tab.id] > 0 && (
              <span className={cn("text-[10px] font-mono tabular-nums px-1.5 py-0.5 rounded-md",
                activeTab === tab.id ? "bg-primary/20 text-primary" : "bg-white/[0.04] text-muted-foreground/50"
              )}>{counts[tab.id]}</span>
            )}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        {!targetUrl ? (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground/30 px-6 text-center">
            <div className="h-16 w-16 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-4">
              <Radar className="h-7 w-7 opacity-50" />
            </div>
            <span className="text-[14px] font-medium text-muted-foreground/50">No active scan target</span>
            <span className="text-[12px] mt-1 text-muted-foreground/30">Start a scan from the top bar to begin.</span>
          </div>
        ) : isLoading && !allScans.length ? (
          <div className="flex items-center justify-center h-full">
            <div className="flex items-center gap-2 text-primary animate-pulse">
              <Loader2 className="h-5 w-5 animate-spin" />
              <span className="text-[13px] font-medium">Loading scanner modules…</span>
            </div>
          </div>
        ) : (
          <div className="max-w-6xl mx-auto">

            {/* Analyze */}
            {activeTab === "analyze" && (
              <IntelModuleTab
                title="Analyze"
                targetUrl={targetUrl}
                onExecute={(url) => analyzeMut.mutateAsync({ url })}
                isPending={analyzeMut.isPending}
                result={analyzeMut.data}
                error={analyzeMut.error}
                renderResult={(data) => <AnalyzePanel data={data} />}
              />
            )}

            {/* Investigate */}
            {activeTab === "investigate" && (
              <IntelModuleTab
                title="Investigate"
                targetUrl={targetUrl}
                onExecute={(url) => investigateMut.mutateAsync({ url })}
                isPending={investigateMut.isPending}
                result={investigateMut.data}
                error={investigateMut.error}
                renderResult={(data) => <ServerInfoPanel serverInfo={data} />}
              />
            )}

            {/* API Discovery */}
            {activeTab === "discover" && (
              <IntelModuleTab
                title="API Discovery"
                targetUrl={targetUrl}
                onExecute={(url) => discoverMut.mutateAsync({ url })}
                isPending={discoverMut.isPending}
                result={discoverMut.data}
                error={discoverMut.error}
                renderResult={(data) => <EndpointTable discovery={data} />}
              />
            )}

            {/* Services */}
            {activeTab === "services" && (
              <IntelModuleTab
                title="Service Collection"
                targetUrl={targetUrl}
                onExecute={(url) => servicesMut.mutateAsync({ url })}
                isPending={servicesMut.isPending}
                result={servicesMut.data}
                error={servicesMut.error}
                renderResult={(data) => <ServiceGrid collector={data} />}
              />
            )}

            {/* Security Audit */}
            {activeTab === "audit" && (
              <IntelModuleTab
                title="Security Audit"
                targetUrl={targetUrl}
                onExecute={(url) => auditMut.mutateAsync({ url })}
                isPending={auditMut.isPending}
                result={auditMut.data}
                error={auditMut.error}
                renderResult={(data) => <SecurityAuditPanel data={data} />}
              />
            )}

            {/* Form Mapping */}
            {activeTab === "forms" && (
              <IntelModuleTab
                title="Form Mapping"
                targetUrl={targetUrl}
                onExecute={(url) => formsMut.mutateAsync({ url })}
                isPending={formsMut.isPending}
                result={formsMut.data}
                error={formsMut.error}
                renderResult={(data) => <FormMappingPanel data={data} />}
              />
            )}

            {/* Attack Intelligence */}
            {activeTab === "intelligence" && (
              <IntelModuleTab
                title="Attack Intelligence & Correlation"
                targetUrl={targetUrl}
                onExecute={(url) => masterReportMut.mutateAsync({ url })}
                isPending={masterReportMut.isPending}
                result={masterReportMut.data}
                error={masterReportMut.error}
                renderResult={(data) => {
                  if (!data.normalized_audit) {
                    return (
                      <div className="p-8 text-center text-muted-foreground/60">
                        No attack intelligence generated. Correlation modules might not have produced data.
                      </div>
                    );
                  }
                  return <AttackChainsPanel data={data.normalized_audit} />;
                }}
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
}
