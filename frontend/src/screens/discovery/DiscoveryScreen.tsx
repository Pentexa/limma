"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import { useActiveMasterReport } from "@/entities/discovery/model/use-master-report";
import { EndpointTable } from "./components/EndpointTable";
import { ServiceGrid } from "./components/ServiceGrid";
import { ServerInfoPanel } from "./components/ServerInfoPanel";
import { TechStackPanel } from "./components/TechStackPanel";
import { Search, Loader2, AlertTriangle, Globe, Server, Cpu, Network } from "lucide-react";

type Tab = "endpoints" | "services" | "server" | "technologies";

const TABS: { id: Tab; label: string; icon: React.ElementType }[] = [
  { id: "endpoints", label: "Endpoints", icon: Network },
  { id: "services", label: "Services", icon: Server },
  { id: "server", label: "Server Info", icon: Globe },
  { id: "technologies", label: "Technologies", icon: Cpu },
];

export function DiscoveryScreen() {
  const [activeTab, setActiveTab] = useState<Tab>("endpoints");
  const { data: report, isLoading, error, targetUrl, isScanning } = useActiveMasterReport();

  /* Compute counts for tab badges */
  const counts: Record<Tab, number> = {
    endpoints: report?.api_discovery?.detected_endpoints?.length ?? 0,
    services: report?.service_collector?.port_results?.length ?? 0,
    server: (report?.server_info?.fingerprints?.length ?? 0) + (report?.server_info?.security_insights?.length ?? 0),
    technologies: report?.analysis?.detected_technologies?.length ?? 0,
  };

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden bg-[#0a0a0c]">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-5 py-3 border-b border-white/[0.06]">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center h-7 w-7 rounded-lg bg-primary/10 border border-primary/20">
            <Search className="h-3.5 w-3.5 text-primary" />
          </div>
          <div>
            <h2 className="text-sm font-semibold text-foreground">Discovery</h2>
            {targetUrl && (
              <p className="text-[11px] font-mono text-muted-foreground/60 truncate max-w-[300px]">
                {targetUrl}
              </p>
            )}
          </div>
        </div>
        {(isLoading || isScanning) && (
          <div className="flex items-center gap-1.5 text-[11px] font-medium text-primary animate-pulse">
            <Loader2 className="h-4 w-4 animate-spin" /> {isScanning ? "Scanning target…" : "Loading data…"}
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
        {error && !report ? (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground/30 px-6 text-center">
            <div className="h-16 w-16 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-4 text-red-400">
              <AlertTriangle className="h-7 w-7 opacity-70" />
            </div>
            <span className="text-[14px] font-medium text-muted-foreground/50">Failed to load discovery data</span>
            <span className="text-[12px] mt-1 text-muted-foreground/30 font-mono">{error.message}</span>
          </div>
        ) : !targetUrl ? (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground/30 px-6 text-center">
            <div className="h-16 w-16 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-4">
              <Search className="h-7 w-7 opacity-50" />
            </div>
            <span className="text-[14px] font-medium text-muted-foreground/50">No active scan target</span>
            <span className="text-[12px] mt-1 text-muted-foreground/30">Start a scan from the top bar to begin.</span>
          </div>
        ) : isLoading && !report ? (
          <div className="flex items-center justify-center h-full">
            <div className="flex items-center gap-2 text-primary animate-pulse">
              <Loader2 className="h-5 w-5 animate-spin" />
              <span className="text-[13px] font-medium">Running discovery analysis…</span>
            </div>
          </div>
        ) : (
          <div className="max-w-6xl mx-auto h-full">
            {activeTab === "endpoints" && <EndpointTable discovery={report?.api_discovery} />}
            {activeTab === "services" && <ServiceGrid collector={report?.service_collector} />}
            {activeTab === "server" && <ServerInfoPanel serverInfo={report?.server_info} />}
            {activeTab === "technologies" && <TechStackPanel technologies={report?.analysis?.detected_technologies} />}
          </div>
        )}
      </div>
    </div>
  );
}
