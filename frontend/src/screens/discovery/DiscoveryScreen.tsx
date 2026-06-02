"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import { useDiscoverSubdomains } from "@/entities/intelligence/model/use-intelligence";
import { SubdomainDiscoveryPanel } from "./components/SubdomainDiscoveryPanel";
import { SmartDataViewer } from "@/widgets/smart-data-viewer/SmartDataViewer";
import {
  Compass, Globe, Cloud, Key, FileWarning, MapPin,
  Loader2, Play, CheckCircle, AlertTriangle,
  Code2, LayoutTemplate, Search, Lock,
} from "lucide-react";

type DiscoveryTab = "subdomains" | "cloud" | "leaks" | "certificates" | "exposure" | "expansion";

const TABS: { id: DiscoveryTab; label: string; icon: React.ElementType; ready: boolean }[] = [
  { id: "subdomains",   label: "Subdomains",        icon: Globe,       ready: true },
  { id: "cloud",        label: "Cloud Assets",      icon: Cloud,       ready: false },
  { id: "leaks",        label: "Leak Discovery",    icon: Key,         ready: false },
  { id: "certificates", label: "Certificates",      icon: FileWarning, ready: false },
  { id: "exposure",     label: "Exposure Map",      icon: MapPin,      ready: false },
  { id: "expansion",    label: "Attack Surface",    icon: Compass,     ready: false },
];

const COMING_SOON_INFO: Record<string, { title: string; description: string }> = {
  cloud:        { title: "Cloud Asset Discovery",     description: "Automatically discover and map AWS S3 buckets, GCP storage, Azure blobs, and cloud instances associated with the target domain." },
  leaks:        { title: "Leak Discovery",            description: "Search for exposed source code repositories, sensitive documents, API keys, and credentials across public platforms." },
  certificates: { title: "Certificate Discovery",     description: "Scan Certificate Transparency logs to discover related domains, subdomains, and certificate misconfigurations." },
  exposure:     { title: "External Exposure Map",     description: "Build a comprehensive visual map of the entire internet-facing attack surface including all discovered assets." },
  expansion:    { title: "Attack Surface Expansion",  description: "Perform recursive analysis across discovered assets to identify adjacent targets, shared infrastructure, and lateral paths." },
};

export function DiscoveryScreen() {
  const [activeTab, setActiveTab] = useState<DiscoveryTab>("subdomains");
  const [domain, setDomain] = useState("");
  const [viewMode, setViewMode] = useState<"ui" | "raw">("ui");
  const subdomainMut = useDiscoverSubdomains();

  function handleRun() {
    if (!domain.trim()) return;
    let cleanDomain = domain.trim();
    try {
      const url = new URL(cleanDomain.includes("://") ? cleanDomain : `https://${cleanDomain}`);
      cleanDomain = url.hostname;
    } catch {
      // Use as-is if not a valid URL
    }
    subdomainMut.mutate({ domain: cleanDomain });
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") handleRun();
  }

  const isPending = activeTab === "subdomains" && subdomainMut.isPending;

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
            <p className="text-[11px] font-mono text-muted-foreground/60">
              Asset discovery & attack surface mapping
            </p>
          </div>
        </div>
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
            <tab.icon className="h-4 w-4 shrink-0 opacity-80" />
            {tab.label}
            {!tab.ready && (
              <Lock className="h-3 w-3 text-muted-foreground/30" />
            )}
            {tab.id === "subdomains" && subdomainMut.data && (
              <span className={cn("text-[10px] font-mono tabular-nums px-1.5 py-0.5 rounded-md",
                activeTab === tab.id ? "bg-primary/20 text-primary" : "bg-white/[0.04] text-muted-foreground/50"
              )}>{subdomainMut.data.assets.length}</span>
            )}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-6xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500">

          {/* ── Subdomain Discovery Tab ── */}
          {activeTab === "subdomains" && (
            <div className="space-y-6">
              {/* Input + Run */}
              <div className="flex items-center gap-3">
                <div className="relative flex-1">
                  <Globe className="absolute left-3.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/30" />
                  <input
                    id="subdomain-domain-input"
                    type="text"
                    placeholder="Enter target domain (e.g. example.com)"
                    value={domain}
                    onChange={(e) => setDomain(e.target.value)}
                    onKeyDown={handleKeyDown}
                    className="w-full h-11 pl-10 pr-4 text-[13px] font-mono bg-white/[0.03] border border-white/[0.06] rounded-lg focus:border-primary/50 focus:ring-1 focus:ring-primary/20 focus:outline-none transition-all text-foreground placeholder:text-muted-foreground/40"
                  />
                </div>
                <button
                  id="subdomain-run-button"
                  className={cn(
                    "flex items-center gap-2 px-6 h-11 rounded-lg text-[12px] font-semibold transition-all duration-200 active:scale-[0.98]",
                    subdomainMut.isPending
                      ? "bg-primary/10 text-primary border border-primary/20 cursor-wait"
                      : "bg-primary/10 text-primary border border-primary/20 hover:bg-primary/20",
                    (!domain.trim() || subdomainMut.isPending) && "opacity-50"
                  )}
                  disabled={!domain.trim() || subdomainMut.isPending}
                  onClick={handleRun}
                >
                  {subdomainMut.isPending ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Play className="h-4 w-4 shrink-0" />
                  )}
                  Run Discovery
                </button>
              </div>

              {/* Error */}
              {subdomainMut.error && (
                <div className="flex items-center gap-3 text-[12px] font-medium text-red-400 bg-red-500/[0.06] border border-red-500/10 rounded-xl px-4 py-3">
                  <AlertTriangle className="h-4 w-4 shrink-0" /> {subdomainMut.error.message}
                </div>
              )}

              {/* Results */}
              {subdomainMut.data && (
                <div className={cn(
                  "transition-all duration-300 flex flex-col",
                  viewMode === "raw" ? "bg-white/[0.02] border border-white/[0.06] rounded-xl shadow-lg overflow-hidden" : ""
                )}>
                  <div className={cn(
                    "flex items-center justify-between gap-3",
                    viewMode === "raw" ? "p-4 border-b border-white/[0.04] bg-white/[0.01]" : "mb-4"
                  )}>
                    <div className="flex items-center gap-3">
                      <div className="flex items-center justify-center h-6 w-6 rounded-md bg-emerald-500/10">
                        <CheckCircle className="h-3.5 w-3.5 text-emerald-400" />
                      </div>
                      <span className="text-[13px] font-semibold text-foreground/90">Discovery Results</span>
                      <span className="text-[11px] font-mono font-medium text-muted-foreground/60 bg-white/[0.04] border border-white/[0.06] px-2 py-0.5 rounded-md">
                        {subdomainMut.data.assets.length} subdomains found
                      </span>
                    </div>
                    <div className="flex items-center bg-black/40 border border-white/[0.06] rounded-lg p-0.5">
                      <button
                        onClick={() => setViewMode("ui")}
                        className={cn(
                          "flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[11px] font-semibold transition-colors",
                          viewMode === "ui"
                            ? "bg-white/[0.08] text-foreground shadow-sm"
                            : "text-muted-foreground/60 hover:text-foreground/80 hover:bg-white/[0.02]"
                        )}
                      >
                        <LayoutTemplate className="h-3.5 w-3.5" />
                        UI View
                      </button>
                      <button
                        onClick={() => setViewMode("raw")}
                        className={cn(
                          "flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[11px] font-semibold transition-colors",
                          viewMode === "raw"
                            ? "bg-white/[0.08] text-foreground shadow-sm"
                            : "text-muted-foreground/60 hover:text-foreground/80 hover:bg-white/[0.02]"
                        )}
                      >
                        <Code2 className="h-3.5 w-3.5" />
                        Raw Data
                      </button>
                    </div>
                  </div>
                  <div className="bg-transparent">
                    {viewMode === "ui" ? (
                      <SubdomainDiscoveryPanel data={subdomainMut.data} />
                    ) : (
                      <SmartDataViewer data={subdomainMut.data} />
                    )}
                  </div>
                </div>
              )}

              {/* Empty state */}
              {!subdomainMut.data && !subdomainMut.isPending && !subdomainMut.error && (
                <div className="flex flex-col items-center justify-center py-16 text-muted-foreground/30 px-6 text-center">
                  <div className="h-16 w-16 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-4">
                    <Globe className="h-7 w-7 opacity-50" />
                  </div>
                  <span className="text-[14px] font-medium text-muted-foreground/50">
                    Enter a domain to begin discovery
                  </span>
                  <span className="text-[12px] mt-1 text-muted-foreground/30">
                    Enumerate subdomains via passive collection (crt.sh) and active DNS brute-forcing.
                  </span>
                </div>
              )}
            </div>
          )}

          {/* ── Coming Soon Tabs ── */}
          {activeTab !== "subdomains" && (
            <ComingSoonPanel
              tabId={activeTab}
              icon={TABS.find(t => t.id === activeTab)!.icon}
            />
          )}

        </div>
      </div>
    </div>
  );
}

/* ── Coming Soon Placeholder ── */
function ComingSoonPanel({ tabId, icon: Icon }: { tabId: string; icon: React.ElementType }) {
  const info = COMING_SOON_INFO[tabId];
  if (!info) return null;

  return (
    <div className="flex flex-col items-center justify-center py-24 px-6 text-center animate-in fade-in duration-500">
      <div className="relative mb-6">
        <div className="h-20 w-20 rounded-2xl bg-white/[0.02] border border-white/[0.06] flex items-center justify-center">
          <Icon className="h-9 w-9 text-muted-foreground/30" />
        </div>
        <div className="absolute -top-1.5 -right-1.5 h-6 w-6 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center">
          <Lock className="h-3 w-3 text-primary/60" />
        </div>
      </div>
      <h3 className="text-[18px] font-bold text-foreground/80 mb-2">{info.title}</h3>
      <p className="text-[13px] text-muted-foreground/50 max-w-md leading-relaxed mb-6">
        {info.description}
      </p>
      <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary/[0.04] border border-primary/10">
        <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
        <span className="text-[11px] font-semibold text-primary/70 uppercase tracking-wider">
          In Development
        </span>
      </div>
    </div>
  );
}
