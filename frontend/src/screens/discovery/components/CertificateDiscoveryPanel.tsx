"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import type { ApiDiscoverCertificatesResponse, ApiSubdomainAsset } from "@/shared/types/api";
import {
  FileWarning, AlertTriangle, CheckCircle, Activity,
  Clock, Filter, Search, ChevronDown, ChevronRight,
  Wifi, WifiOff, ExternalLink, Server, Tag, Fingerprint,
  BarChart3, Eye,
} from "lucide-react";

interface CertificateDiscoveryPanelProps {
  data: ApiDiscoverCertificatesResponse;
  domain: string;
}

const STATUS_CONFIG: Record<string, { label: string; color: string; icon: React.ElementType }> = {
  validated: { label: "Validated", color: "text-emerald-400 bg-emerald-500/10 border-emerald-500/20", icon: CheckCircle },
  http_alive: { label: "HTTP Alive", color: "text-cyan-400 bg-cyan-500/10 border-cyan-500/20", icon: Wifi },
  http_dead: { label: "HTTP Dead", color: "text-red-400 bg-red-500/10 border-red-500/20", icon: WifiOff },
  unresolved: { label: "Unresolved", color: "text-yellow-400 bg-yellow-500/10 border-yellow-500/20", icon: AlertTriangle },
  wildcard_filtered: { label: "Wildcard", color: "text-orange-400 bg-orange-500/10 border-orange-500/20", icon: Filter },
};

const SOURCE_LABELS: Record<string, string> = {
  crt_sh: "crt.sh",
  dns_wordlist: "DNS Brute",
  crawler_links: "Crawler",
};

export function CertificateDiscoveryPanel({ data, domain }: CertificateDiscoveryPanelProps) {
  const [expandedAsset, setExpandedAsset] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  const filteredAssets = data.assets.filter((asset) => {
    if (statusFilter && asset.status !== statusFilter) return false;
    if (searchQuery && !asset.asset.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  const durationSec = (data.scan_duration_ms / 1000).toFixed(1);
  const precisionPct = data.unique_candidates > 0 ? ((data.validated_assets / data.unique_candidates) * 100).toFixed(1) : "0.0";
  const httpAliveCount = data.assets.filter(a => a.status === 'http_alive').length;

  return (
    <div className="space-y-4 w-full">
      {/* ── Metrics HUD ── */}
      <div className="flex flex-wrap items-center gap-3 p-3 bg-[#080808] rounded-md border border-border/30 shadow-inner">
        <div className="flex items-center gap-2">
          <FileWarning className="h-4 w-4 text-primary" />
          <span className="text-[11px] font-bold text-foreground uppercase tracking-widest">
            Certificate Discovery
          </span>
        </div>
        <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded ml-auto">
          <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">Domain</span>
          <span className="font-mono text-[11px] text-foreground font-bold">{domain}</span>
        </div>
      </div>

      {/* ── Stats Grid ── */}
      <div className="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-8 gap-2">
        {[
          { label: "Total Cert Names", value: data.total_cert_names, icon: Search, color: "text-blue-400" },
          { label: "Unique Candidates", value: data.unique_candidates, icon: Eye, color: "text-violet-400" },
          { label: "Validated", value: data.validated_assets, icon: CheckCircle, color: "text-emerald-400" },
          { label: "HTTP Alive", value: httpAliveCount, icon: Wifi, color: "text-cyan-400" },
          { label: "Wildcard Removed", value: data.wildcard_removed, icon: Filter, color: "text-orange-400" },
          { label: "Out of Scope", value: data.out_of_scope_removed, icon: Filter, color: "text-yellow-400" },
          { label: "Precision", value: `${precisionPct}%`, icon: BarChart3, color: "text-emerald-400" },
          { label: "Duration", value: `${durationSec}s`, icon: Clock, color: "text-muted-foreground" },
        ].map((stat) => (
          <div
            key={stat.label}
            className="bg-[#080808] border border-border/20 rounded-md p-3 flex flex-col items-center gap-1.5 hover:border-white/[0.08] transition-colors"
          >
            <stat.icon className={cn("h-3.5 w-3.5", stat.color)} />
            <span className="text-[14px] font-mono font-bold text-foreground tabular-nums">
              {stat.value}
            </span>
            <span className="text-[9px] uppercase tracking-wider text-muted-foreground/50 font-bold text-center leading-tight">
              {stat.label}
            </span>
          </div>
        ))}
      </div>

      {/* ── Filter Bar ── */}
      <div className="flex items-center gap-3 flex-wrap">
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground/40" />
          <input
            type="text"
            placeholder="Filter domains…"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full h-8 pl-9 pr-3 text-[11px] font-mono bg-white/[0.03] border border-white/[0.06] rounded-lg focus:border-primary/50 focus:ring-1 focus:ring-primary/20 focus:outline-none transition-all text-foreground placeholder:text-muted-foreground/40"
          />
        </div>
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => setStatusFilter(null)}
            className={cn(
              "px-2.5 py-1.5 rounded-md text-[10px] font-bold uppercase tracking-wider transition-colors border",
              !statusFilter
                ? "bg-primary/10 text-primary border-primary/20"
                : "bg-white/[0.02] text-muted-foreground/50 border-white/[0.04] hover:text-foreground"
            )}
          >
            All ({data.assets.length})
          </button>
          {Object.entries(STATUS_CONFIG).map(([key, cfg]) => {
            const count = data.assets.filter(a => a.status === key).length;
            if (count === 0) return null;
            return (
              <button
                key={key}
                onClick={() => setStatusFilter(statusFilter === key ? null : key)}
                className={cn(
                  "px-2.5 py-1.5 rounded-md text-[10px] font-bold uppercase tracking-wider transition-colors border",
                  statusFilter === key
                    ? cfg.color
                    : "bg-white/[0.02] text-muted-foreground/50 border-white/[0.04] hover:text-foreground"
                )}
              >
                {cfg.label} ({count})
              </button>
            );
          })}
        </div>
      </div>

      {/* ── Assets Table ── */}
      <div className="bg-[#080808] border border-border/20 rounded-md shadow-lg overflow-hidden">
        {/* Table Header */}
        <div className="grid grid-cols-[1fr_120px_100px_100px_80px] gap-3 items-center px-4 py-2.5 bg-white/[0.01] border-b border-border/10 text-[9px] font-bold uppercase tracking-widest text-muted-foreground/50">
          <span>Domain</span>
          <span>IPs</span>
          <span>Status</span>
          <span>Sources</span>
          <span className="text-right">Confidence</span>
        </div>

        {/* Table Body */}
        <div className="divide-y divide-border/10 max-h-[500px] overflow-y-auto">
          {filteredAssets.length === 0 ? (
            <div className="p-8 text-center text-muted-foreground/40 text-[12px] font-mono">
              No domains match the current filter.
            </div>
          ) : (
            filteredAssets.map((asset) => (
              <AssetRow
                key={asset.asset}
                asset={asset}
                expanded={expandedAsset === asset.asset}
                onToggle={() =>
                  setExpandedAsset(expandedAsset === asset.asset ? null : asset.asset)
                }
              />
            ))
          )}
        </div>
      </div>
    </div>
  );
}

/* ── Individual Asset Row ── */
function AssetRow({
  asset,
  expanded,
  onToggle,
}: {
  asset: ApiSubdomainAsset;
  expanded: boolean;
  onToggle: () => void;
}) {
  const status = STATUS_CONFIG[asset.status] ?? STATUS_CONFIG.unresolved;
  const StatusIcon = status.icon;
  const confidencePct = Math.round(asset.confidence * 100);

  return (
    <div className="group">
      {/* Main Row */}
      <button
        className="w-full grid grid-cols-[1fr_120px_100px_100px_80px] gap-3 items-center px-4 py-3 hover:bg-white/[0.015] transition-colors text-left"
        onClick={onToggle}
      >
        {/* Subdomain */}
        <div className="flex items-center gap-2 min-w-0">
          {expanded ? (
            <ChevronDown className="h-3 w-3 text-muted-foreground/40 shrink-0" />
          ) : (
            <ChevronRight className="h-3 w-3 text-muted-foreground/40 shrink-0" />
          )}
          <span className="text-[12px] font-mono font-semibold text-foreground truncate">
            {asset.asset}
          </span>
          {asset.http_probe && (
            <ExternalLink className="h-3 w-3 text-primary/40 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity" />
          )}
        </div>

        {/* IPs */}
        <div className="flex flex-col gap-0.5 min-w-0">
          {asset.resolved_ips.slice(0, 2).map((ip) => (
            <span key={ip} className="text-[10px] font-mono text-muted-foreground/70 truncate">
              {ip}
            </span>
          ))}
          {asset.resolved_ips.length > 2 && (
            <span className="text-[9px] text-muted-foreground/40">
              +{asset.resolved_ips.length - 2} more
            </span>
          )}
        </div>

        {/* Status */}
        <div className={cn("flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded border w-fit", status.color)}>
          <StatusIcon className="h-2.5 w-2.5" />
          {status.label}
        </div>

        {/* Sources */}
        <div className="flex flex-wrap gap-1">
          {asset.sources.map((src) => (
            <span
              key={src}
              className="text-[9px] font-mono bg-white/[0.04] border border-white/[0.06] px-1.5 py-0.5 rounded text-muted-foreground/60"
            >
              {SOURCE_LABELS[src] ?? src}
            </span>
          ))}
        </div>

        {/* Confidence */}
        <div className="flex items-center gap-2 justify-end">
          <div className="w-12 h-1.5 bg-white/[0.06] rounded-full overflow-hidden">
            <div
              className={cn(
                "h-full rounded-full transition-all",
                confidencePct >= 80
                  ? "bg-emerald-500 shadow-[0_0_6px_rgba(52,211,153,0.4)]"
                  : confidencePct >= 50
                  ? "bg-yellow-500 shadow-[0_0_6px_rgba(250,204,21,0.3)]"
                  : "bg-red-500 shadow-[0_0_6px_rgba(248,113,113,0.3)]"
              )}
              style={{ width: `${confidencePct}%` }}
            />
          </div>
          <span
            className={cn(
              "text-[11px] font-mono font-bold tabular-nums",
              confidencePct >= 80
                ? "text-emerald-400"
                : confidencePct >= 50
                ? "text-yellow-400"
                : "text-red-400"
            )}
          >
            {confidencePct}%
          </span>
        </div>
      </button>

      {/* Expanded Detail */}
      {expanded && (
        <div className="px-4 pb-4 pt-1 animate-in fade-in slide-in-from-top-1 duration-200">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 ml-5">
            {/* DNS Records */}
            {asset.dns_records.length > 0 && (
              <div className="bg-black/40 border border-white/[0.04] rounded-md p-3">
                <div className="flex items-center gap-2 mb-2">
                  <Fingerprint className="h-3 w-3 text-primary/60" />
                  <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">
                    DNS Records
                  </span>
                </div>
                <div className="space-y-1.5">
                  {asset.dns_records.map((rec, i) => (
                    <div key={i} className="flex items-start gap-2">
                      <span className="text-[9px] font-mono font-bold text-primary/70 bg-primary/10 px-1.5 py-0.5 rounded shrink-0">
                        {rec.record_type}
                      </span>
                      <span className="text-[10px] font-mono text-muted-foreground/70 break-all">
                        {rec.values.join(", ")}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* HTTP Probe */}
            {asset.http_probe && (
              <div className="bg-black/40 border border-white/[0.04] rounded-md p-3">
                <div className="flex items-center gap-2 mb-2">
                  <Activity className="h-3 w-3 text-cyan-400/60" />
                  <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">
                    HTTP Probe
                  </span>
                </div>
                <div className="space-y-1.5 text-[10px] font-mono">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground/50">Status</span>
                    <span className={cn(
                      "font-bold",
                      asset.http_probe.status_code < 400 ? "text-emerald-400" : "text-red-400"
                    )}>
                      {asset.http_probe.status_code}
                    </span>
                  </div>
                  {asset.http_probe.title && (
                    <div className="flex justify-between gap-2">
                      <span className="text-muted-foreground/50 shrink-0">Title</span>
                      <span className="text-foreground/70 truncate text-right">{asset.http_probe.title}</span>
                    </div>
                  )}
                  {asset.http_probe.server && (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground/50">Server</span>
                      <span className="text-foreground/70">{asset.http_probe.server}</span>
                    </div>
                  )}
                  <div className="flex justify-between">
                    <span className="text-muted-foreground/50">Response</span>
                    <span className="text-foreground/70">{asset.http_probe.response_time_ms}ms</span>
                  </div>
                  {asset.http_probe.tls_issuer && (
                    <div className="flex justify-between gap-2">
                      <span className="text-muted-foreground/50 shrink-0">TLS</span>
                      <span className="text-foreground/70 truncate text-right">{asset.http_probe.tls_issuer}</span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Technologies & Risk Tags */}
            {(asset.technologies.length > 0 || asset.risk_tags.length > 0) && (
              <div className="bg-black/40 border border-white/[0.04] rounded-md p-3">
                {asset.technologies.length > 0 && (
                  <>
                    <div className="flex items-center gap-2 mb-2">
                      <Server className="h-3 w-3 text-violet-400/60" />
                      <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">
                        Technologies
                      </span>
                    </div>
                    <div className="flex flex-wrap gap-1.5 mb-3">
                      {asset.technologies.map((tech) => (
                        <span
                          key={tech}
                          className="text-[9px] font-mono bg-violet-500/10 text-violet-400 border border-violet-500/20 px-1.5 py-0.5 rounded"
                        >
                          {tech}
                        </span>
                      ))}
                    </div>
                  </>
                )}
                {asset.risk_tags.length > 0 && (
                  <>
                    <div className="flex items-center gap-2 mb-2">
                      <Tag className="h-3 w-3 text-red-400/60" />
                      <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">
                        Risk Tags
                      </span>
                    </div>
                    <div className="flex flex-wrap gap-1.5">
                      {asset.risk_tags.map((tag) => (
                        <span
                          key={tag}
                          className="text-[9px] font-mono bg-red-500/10 text-red-400 border border-red-500/20 px-1.5 py-0.5 rounded"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                  </>
                )}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
