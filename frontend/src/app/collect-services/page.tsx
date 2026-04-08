"use client";

import { useState, Fragment } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Database, Loader2, Globe, ShieldAlert, Cpu, Activity, Clock,
  ServerCrash, CheckCircle2, Server, Lock, ChevronDown, ChevronRight,
  Shield, Zap, RotateCcw, ArrowDownRight, AlertTriangle, Eye, Brain,
} from "lucide-react";
import { extractApiError, reportApi } from "@/utils/api";
import { useLanguage } from "@/context/LanguageContext";
import { CertaintyBadge } from "@/components/CertaintyBadge";

/* ───── TypeScript Interfaces (Phase 3) ───── */

interface TargetInput {
  original_input: string;
  normalized_url: string;
  host: string;
  scheme?: string;
  default_port: number;
}
interface ResolvedTarget {
  ip_addresses: string[];
  primary_ip?: string;
  hostname?: string;
}
interface TlsSummary {
  has_tls: boolean;
  protocol_version?: string;
  cipher_suite?: string;
  subject?: string;
  issuer?: string;
  alpn?: string;
  sni_used: boolean;
}
interface HttpSummary {
  status_code?: number;
  server_header?: string;
  content_type?: string;
  redirect_target?: string;
  headers?: Record<string, string>;
  response_length?: number;
}
interface EvidenceItem {
  kind: string;       // protocol_greeting | tls_handshake | http_response | banner_text | port_assumption
  strength: string;   // strong | medium | weak
  source: string;     // banner | http | tls | greeting | port_default
  raw_signal: string;
  interpretation: string;
  suggests_service?: string;
  is_negative: boolean;
}
interface AmbiguityReason {
  description: string;
  conflicting_evidence: string[];
}
interface RuleEvaluation {
  category: string;
  expected: string;
  actual?: string;
  matched: boolean;
  weight: number;
  rule_weight: string;  // critical | strong | medium | weak | contextual
  contribution: number;
  skipped_contextual: boolean;
}
interface ExplanationItem {
  category: string;  // boost | decay | penalty | info | contextual
  description: string;
  impact: number;
}
interface MatchPenalty {
  reason: string;
  amount: number;
}
interface FingerprintMatch {
  fingerprint_id: string;
  service_name: string;
  tier: string;           // specific | generic | fallback
  strength: string;       // full | strong | partial | weak | no_match
  confidence: number;
  confidence_level: string;  // confirmed | high | medium | low | tentative
  coverage: string;          // full | high | partial | minimal
  matched_rules: RuleEvaluation[];
  missing_rules: RuleEvaluation[];
  conflicting_rules: RuleEvaluation[];
  explanation_items: ExplanationItem[];
  penalties: MatchPenalty[];
  reasoning: string;
}
interface ConfidenceBreakdown {
  port_evidence: number;
  protocol_validation: number;
  fingerprint_strength: number;
  header_reliability: number;
  redirect_penalty: number;
  cdn_penalty: number;
  response_quality: number;
  final_score: number;
}
interface DecisionTreeStep {
  step: string;
  detail: string;
}
interface ServiceCandidate {
  service_name: string;
  confidence_breakdown: ConfidenceBreakdown;
  decision: string;   // verified | suspected | cdn_edge | routing_behavior | filtered
  probe_method: string;
  supporting_evidence: EvidenceItem[];
  conflicting_evidence: EvidenceItem[];
  reasoning: string;
  tls_summary?: TlsSummary;
  http_summary?: HttpSummary;
  ambiguity?: AmbiguityReason;
  fingerprint_match?: FingerprintMatch;
  verification_trail: DecisionTreeStep[];
}
interface PortProbeResult {
  port: number;
  state: string;
  latency_ms?: number;
  service_candidates: ServiceCandidate[];
  all_evidence: EvidenceItem[];
  fingerprint_evaluations: FingerprintMatch[];
  fallback_used: boolean;
  retry_count: number;
  probe_duration_ms: number;
}
interface ActivityEvent {
  timestamp: string;
  severity: string;
  event_type: string;
  message: string;
  metadata?: any;
}
interface ChangeEvent {
  change_type: string;  // added | removed | changed | unchanged
  resource: string;
  before?: string;
  after?: string;
  description: string;
}
interface SnapshotDiff {
  previous_timestamp: string;
  current_timestamp: string;
  changes: ChangeEvent[];
  summaries: string[];
}
interface CollectorSnapshot {
  target_input: TargetInput;
  resolved_target: ResolvedTarget;
  timestamp: string;
  port_results: PortProbeResult[];
  activity_timeline: ActivityEvent[];
  errors: string[];
  overall_status: string;
  diff?: SnapshotDiff;
}

export default function CollectServicesPage() {
  const [url, setUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [data, setData] = useState<CollectorSnapshot | null>(null);
  const [error, setError] = useState("");
  const [expandedPort, setExpandedPort] = useState<number | null>(null);
  const [filterVerified, setFilterVerified] = useState(true);
  const [filterOthers, setFilterOthers] = useState(false);
  const [verifyingPorts, setVerifyingPorts] = useState<Record<number, boolean>>({});
  const { t } = useLanguage();

  const handleCollect = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url) return;
    setIsLoading(true);
    setError("");
    setData(null);
    setExpandedPort(null);

    try {
      const targetUrl = url.startsWith("http") ? url : `https://${url}`;
      const result = await reportApi.collectServices(targetUrl);
      setData(result as CollectorSnapshot);
    } catch (err: any) {
      setError(extractApiError(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleVerifyPort = async (p: any, e: React.MouseEvent) => {
    e.stopPropagation();
    setVerifyingPorts(prev => ({ ...prev, [p.port]: true }));
    
    try {
      const targetIp = data?.target_input?.host || "";
      const result = await reportApi.verifyPort(targetIp, p.port);
      
      setData(prev => {
          if (!prev) return prev;
          const newResults = prev.port_results.map(portRes => {
             if (portRes.port === p.port) {
                 const newCandidate = portRes.service_candidates?.[0] || {} as any;
                 
                 if (result.is_active) {
                     return {
                         ...portRes,
                         state: "open",
                         latency_ms: result.latency_ms || portRes.latency_ms,
                         service_candidates: [
                           { 
                              ...newCandidate, 
                              decision: "verified",
                              confidence_breakdown: {
                                  ...newCandidate.confidence_breakdown,
                                  final_score: 1.0,
                                  protocol_validation: 1.0
                              }
                           },
                           ...(portRes.service_candidates?.slice(1) || [])
                         ]
                     };
                 } else {
                     return {
                         ...portRes,
                         state: "filtered",
                         service_candidates: [
                           {
                              ...newCandidate,
                              decision: "filtered",
                              confidence_breakdown: {
                                  ...newCandidate.confidence_breakdown,
                                  final_score: 0.1,
                                  protocol_validation: 0.0
                              }
                           },
                           ...(portRes.service_candidates?.slice(1) || [])
                         ]
                     };
                 }
             }
             return portRes;
          });
          return { ...prev, port_results: newResults };
      });
    } catch (err) {
      console.error("Port verification failed:", err);
    } finally {
      setVerifyingPorts(prev => ({ ...prev, [p.port]: false }));
    }
  };

  /* ── Helper Functions ── */

  const getStatusColor = (state: string) => {
    switch (state.toLowerCase()) {
      case "open": return "text-emerald-400 bg-emerald-500/10 border-emerald-500/20";
      case "closed": return "text-red-400 bg-red-500/10 border-red-500/20";
      case "filtered": return "text-orange-400 bg-orange-500/10 border-orange-500/20";
      case "timeout": return "text-yellow-400 bg-yellow-500/10 border-yellow-500/20";
      case "ambiguous": return "text-violet-400 bg-violet-500/10 border-violet-500/20";
      default: return "text-gray-400 bg-gray-500/10 border-gray-500/20";
    }
  };

  const getStatusIcon = (state: string) => {
    switch (state.toLowerCase()) {
      case "open": return <CheckCircle2 className="h-3.5 w-3.5" />;
      case "closed": return <ServerCrash className="h-3.5 w-3.5" />;
      case "filtered": return <Lock className="h-3.5 w-3.5" />;
      case "ambiguous": return <AlertTriangle className="h-3.5 w-3.5" />;
      default: return <Clock className="h-3.5 w-3.5" />;
    }
  };

  const getConfidenceColor = (score: number) => {
    if (score >= 0.6) return "bg-emerald-500";
    if (score >= 0.3) return "bg-yellow-500";
    return "bg-red-500";
  };

  const getDecisionBadge = (decision: string) => {
    switch (decision) {
      case "verified": return "bg-emerald-500/20 text-emerald-400 border-emerald-500/30";
      case "suspected": return "bg-yellow-500/20 text-yellow-400 border-yellow-500/30";
      case "routing_behavior": return "bg-blue-500/20 text-blue-400 border-blue-500/30";
      case "cdn_edge": return "bg-orange-500/20 text-orange-400 border-orange-500/30";
      case "filtered": return "bg-gray-500/20 text-gray-400 border-gray-500/30";
      default: return "bg-gray-500/20 text-gray-400 border-gray-500/30";
    }
  };

  const getDecisionIcon = (decision: string) => {
    switch (decision) {
      case "verified": return <CheckCircle2 className="h-3 w-3" />;
      case "suspected": return <Eye className="h-3 w-3" />;
      case "routing_behavior": return <ArrowDownRight className="h-3 w-3" />;
      case "cdn_edge": return <Globe className="h-3 w-3" />;
      case "filtered": return <Lock className="h-3 w-3" />;
      default: return null;
    }
  };

  const getProbeMethodBadge = (method: string) => {
    const colors: Record<string, string> = {
      tls: "bg-blue-500/20 text-blue-400 border-blue-500/30",
      http: "bg-green-500/20 text-green-400 border-green-500/30",
      banner: "bg-purple-500/20 text-purple-400 border-purple-500/30",
      greeting: "bg-orange-500/20 text-orange-400 border-orange-500/30",
      port_default: "bg-gray-500/20 text-gray-400 border-gray-500/30",
    };
    return colors[method] || colors.port_default;
  };

  const getStrengthDot = (strength: string) => {
    switch (strength) {
      case "strong": return "bg-emerald-400";
      case "medium": return "bg-yellow-400";
      case "weak": return "bg-gray-500";
      default: return "bg-gray-600";
    }
  };

  const getEvidenceKindLabel = (kind: string) => {
    switch (kind) {
      case "protocol_greeting": return "Protocol";
      case "tls_handshake": return "TLS";
      case "http_response": return "HTTP";
      case "banner_text": return "Banner";
      case "port_assumption": return "Port";
      default: return kind;
    }
  };

  const getEventTypeColor = (eventType: string) => {
    if (eventType.includes("DECIDED")) return "text-emerald-400 bg-emerald-500/10";
    if (eventType.includes("FALLBACK")) return "text-orange-400 bg-orange-500/10";
    if (eventType.includes("RETRY")) return "text-yellow-400 bg-yellow-500/10";
    if (eventType.includes("TLS")) return "text-blue-400 bg-blue-500/10";
    if (eventType.includes("HTTP")) return "text-green-400 bg-green-500/10";
    if (eventType.includes("SERVICE")) return "text-purple-400 bg-purple-500/10";
    if (eventType.includes("FAILED") || eventType.includes("ERROR")) return "text-red-400 bg-red-500/10";
    return "text-gray-400 bg-gray-500/10";
  };

  const openPorts = data?.port_results?.filter((p) => p.state.toLowerCase() === "open") || [];
  const suspectedCount = openPorts.filter(p => p.service_candidates?.[0]?.decision !== "verified").length;

  const filteredPorts = data?.port_results?.filter(p => {
    const dec = p.service_candidates?.[0]?.decision;
    if (dec === "verified") return filterVerified;
    if (dec) return filterOthers;
    return filterOthers; // For ports with no candidate or not verified
  }) || [];

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      <header className="mb-10">
        <h1 className="text-4xl font-extrabold tracking-tight text-white flex items-center gap-3">
          {t.colTitle}
          <Database className="h-8 w-8 text-purple-400" />
        </h1>
        <p className="mt-2 text-gray-400 text-lg">{t.colDesc}</p>
      </header>

      <form onSubmit={handleCollect} className="relative group max-w-3xl">
        <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
          <Globe className="h-5 w-5 text-gray-400 group-focus-within:text-purple-400 transition-colors" />
        </div>
        <input
          id="urlInput"
          name="url"
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder={t.enterUrl}
          className="block w-full pl-12 pr-44 py-4 bg-sidebar-bg/50 border border-sidebar-border rounded-2xl text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-400/50 focus:border-purple-400 transition-all shadow-xl shadow-black/20"
        />
        <button
          type="submit"
          disabled={isLoading || !url}
          className="absolute right-2 top-2 bottom-2 px-8 bg-gradient-to-r from-purple-500 to-indigo-500 hover:from-purple-400 hover:to-indigo-400 rounded-xl font-semibold text-white flex items-center gap-2 transition-all disabled:opacity-50 transform hover:scale-[1.02] active:scale-95 shadow-[0_0_20px_rgba(168,85,247,0.4)]"
        >
          {isLoading ? <Loader2 className="h-4 w-4 animate-spin text-white" /> : <Activity className="h-4 w-4" />}
          {isLoading ? <span>{t.collecting}</span> : "Scan Target"}
        </button>
      </form>

      {error && (
        <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="p-4 bg-red-950/30 border border-red-500/50 rounded-xl text-red-200 flex items-center gap-3 max-w-3xl">
          <ShieldAlert className="h-5 w-5 text-red-400" />
          {error}
        </motion.div>
      )}

      {isLoading && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="mt-16 h-[400px] rounded-2xl bg-sidebar-bg/50 border border-sidebar-border relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent -translate-x-full animate-[shimmer_1.5s_infinite]" />
        </motion.div>
      )}

      <AnimatePresence>
        {data && !isLoading && (
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="mt-10 space-y-6 max-w-6xl">

            {/* ── 3-Column Summary ── */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {/* Target Resolution */}
              <div className="bg-card-bg border border-card-border p-5 rounded-2xl relative overflow-hidden">
                <div className="absolute top-0 right-0 w-28 h-28 rounded-full blur-3xl opacity-10 bg-indigo-400 translate-x-1/2 -translate-y-1/2" />
                <div className="flex items-center gap-3 mb-4">
                  <div className="p-2 rounded-lg bg-black/40 border border-white/5 text-indigo-400"><Globe className="h-4 w-4" /></div>
                  <h3 className="font-semibold text-white text-sm tracking-wide">Target Resolution</h3>
                </div>
                <div className="space-y-2.5">
                  <div className="flex justify-between items-center text-xs">
                    <span className="text-gray-500">Host</span>
                    <span className="text-gray-200 font-mono">{data.target_input?.host}</span>
                  </div>
                  <div className="flex justify-between items-center text-xs">
                    <span className="text-gray-500">Primary IP</span>
                    <span className="text-emerald-400 font-mono font-bold bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">
                      {data.resolved_target?.primary_ip || "Unknown"}
                    </span>
                  </div>
                  <div className="flex justify-between items-start text-xs">
                    <span className="text-gray-500">All IPs</span>
                    <div className="flex flex-col items-end gap-0.5 font-mono text-gray-400">
                      {data.resolved_target?.ip_addresses?.map((ip: string) => <span key={ip}>{ip}</span>)}
                    </div>
                  </div>
                </div>
              </div>

              {/* Scan Summary */}
              <div className="bg-card-bg border border-card-border p-5 rounded-2xl relative overflow-hidden">
                <div className="absolute top-0 right-0 w-28 h-28 rounded-full blur-3xl opacity-10 bg-purple-400 translate-x-1/2 -translate-y-1/2" />
                <div className="flex items-center gap-3 mb-4">
                  <div className="p-2 rounded-lg bg-black/40 border border-white/5 text-purple-400"><Cpu className="h-4 w-4" /></div>
                  <h3 className="font-semibold text-white text-sm tracking-wide">Scan Summary</h3>
                </div>
                <div className="space-y-2.5">
                  <div className="flex justify-between items-center text-xs">
                    <span className="text-gray-500">Total Ports</span>
                    <span className="text-gray-200 font-bold">{data.port_results?.length}</span>
                  </div>
                  <div className="flex justify-between items-center text-xs">
                    <span className="text-gray-500">Open Ports</span>
                    <span className="text-emerald-400 font-bold">{openPorts.length}</span>
                  </div>
                  <div className="flex justify-between items-center text-xs">
                    <span className="text-gray-500">Suspected/Edge</span>
                    <span className={`font-bold ${suspectedCount > 0 ? "text-orange-400" : "text-gray-600"}`}>{suspectedCount}</span>
                  </div>
                  <div className="flex justify-between items-center text-xs">
                    <span className="text-gray-500">Fallbacks</span>
                    <span className="text-orange-400 font-bold">{data.port_results?.filter(p => p.fallback_used).length}</span>
                  </div>
                </div>
              </div>

              {/* Detected Services */}
              <div className="bg-card-bg border border-card-border p-5 rounded-2xl relative overflow-hidden">
                <div className="absolute top-0 right-0 w-28 h-28 rounded-full blur-3xl opacity-10 bg-emerald-400 translate-x-1/2 -translate-y-1/2" />
                <div className="flex items-center gap-3 mb-4">
                  <div className="p-2 rounded-lg bg-black/40 border border-white/5 text-emerald-400"><Zap className="h-4 w-4" /></div>
                  <h3 className="font-semibold text-white text-sm tracking-wide">Detected Services</h3>
                </div>
                <div className="flex flex-wrap gap-1.5">
                  {openPorts.map((p) => {
                    const top = p.service_candidates?.[0];
                    if (!top) return null;
                    return (
                      <span
                        key={p.port}
                        className="px-2.5 py-1 bg-purple-500/15 border border-purple-500/20 text-purple-300 rounded-lg text-[11px] font-medium flex items-center gap-1.5"
                      >
                        <span className="font-bold">{top.service_name}</span>
                        <span className="text-gray-500">:{p.port}</span>
                        <span className={`inline-flex items-center gap-0.5 px-1 py-px rounded text-[8px] font-bold border ${getDecisionBadge(top.decision)}`}>
                          {getDecisionIcon(top.decision)}
                        </span>
                      </span>
                    );
                  })}
                  {openPorts.length === 0 && <span className="text-gray-600 text-xs">No open ports detected</span>}
                </div>
              </div>
            </div>

            {/* ── Phase 6: Change Detection Summary ── */}
            {data.diff && data.diff.changes.length > 0 && (
              <div className="bg-card-bg border border-card-border rounded-2xl overflow-hidden">
                <div className="p-5 border-b border-white/5 bg-black/20 flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className="p-2 rounded-lg bg-black/40 border border-white/5 text-amber-400"><Zap className="h-4 w-4" /></div>
                    <h3 className="font-semibold text-white text-sm tracking-wide">Change Detection</h3>
                    <span className="text-[10px] text-gray-500 font-mono">vs previous scan</span>
                  </div>
                  <span className="text-[10px] text-gray-600 font-mono">
                    {new Date(data.diff.previous_timestamp).toLocaleTimeString()} → {new Date(data.diff.current_timestamp).toLocaleTimeString()}
                  </span>
                </div>
                <div className="p-5 space-y-4">
                  {/* Summary badges */}
                  <div className="flex flex-wrap gap-2">
                    {data.diff.summaries.map((s, si) => (
                      <span key={si} className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[11px] font-medium bg-amber-500/10 text-amber-400 border border-amber-500/20">
                        <Zap className="h-3 w-3" />{s}
                      </span>
                    ))}
                  </div>

                  {/* Change event list */}
                  <div className="space-y-2">
                    {data.diff.changes.filter(c => c.change_type !== "unchanged").map((ch, ci) => (
                      <div key={ci} className={`flex items-center gap-3 px-4 py-2.5 rounded-xl text-xs border ${
                        ch.change_type === "added" ? "bg-emerald-500/5 border-emerald-500/15 text-emerald-400" :
                        ch.change_type === "removed" ? "bg-red-500/5 border-red-500/15 text-red-400" :
                        "bg-amber-500/5 border-amber-500/15 text-amber-400"
                      }`}>
                        <span className={`shrink-0 w-5 h-5 rounded-md flex items-center justify-center text-[10px] font-black ${
                          ch.change_type === "added" ? "bg-emerald-500/20 text-emerald-300" :
                          ch.change_type === "removed" ? "bg-red-500/20 text-red-300" :
                          "bg-amber-500/20 text-amber-300"
                        }`}>
                          {ch.change_type === "added" ? "+" : ch.change_type === "removed" ? "−" : "Δ"}
                        </span>
                        <span className="font-semibold">{ch.resource}</span>
                        <span className="text-gray-500">{ch.description}</span>
                        {ch.before && (
                          <span className="ml-auto flex items-center gap-1 font-mono text-[10px]">
                            <span className="text-red-400/70 line-through">{ch.before}</span>
                            {ch.after && <span className="text-gray-600">→</span>}
                            {ch.after && <span className="text-emerald-400">{ch.after}</span>}
                          </span>
                        )}
                        {!ch.before && ch.after && (
                          <span className="ml-auto font-mono text-[10px] text-emerald-400">{ch.after}</span>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {/* ── Port Scanner Table ── */}
            <div className="bg-card-bg border border-card-border rounded-2xl overflow-hidden">
              <div className="p-5 border-b border-white/5 flex items-center justify-between bg-black/20">
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-lg bg-black/40 border border-white/5 text-emerald-400"><Server className="h-4 w-4" /></div>
                  <h3 className="font-semibold text-white text-sm tracking-wide">Network Port Analysis</h3>
                </div>
                <div className="flex items-center gap-3 bg-black/40 p-1 rounded-xl border border-white/5">
                  <button
                    type="button"
                    onClick={() => setFilterVerified(!filterVerified)}
                    className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors ${filterVerified ? "bg-emerald-500/20 text-emerald-400" : "text-gray-500 hover:text-gray-300"}`}
                  >
                    Verified Only
                  </button>
                  <button
                    type="button"
                    onClick={() => setFilterOthers(!filterOthers)}
                    className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors ${filterOthers ? "bg-orange-500/20 text-orange-400" : "text-gray-500 hover:text-gray-300"}`}
                  >
                    Suspected / Edge
                  </button>
                </div>
              </div>
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm whitespace-nowrap">
                  <thead className="bg-black/40 text-gray-500 uppercase text-[10px] tracking-wider">
                    <tr>
                      <th className="px-5 py-3 font-medium border-b border-white/5 w-8"></th>
                      <th className="px-5 py-3 font-medium border-b border-white/5">Port</th>
                      <th className="px-5 py-3 font-medium border-b border-white/5">State</th>
                      <th className="px-5 py-3 font-medium border-b border-white/5">Service</th>
                      <th className="px-5 py-3 font-medium border-b border-white/5">Decision</th>
                      <th className="px-5 py-3 font-medium border-b border-white/5">Confidence</th>
                      <th className="px-5 py-3 font-medium border-b border-white/5">Probe</th>
                      <th className="px-5 py-3 font-medium border-b border-white/5">Latency</th>
                      <th className="px-5 py-3 font-medium border-b border-white/5">Flags</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-white/5">
                    {filteredPorts.map((p) => {
                      const top = p.service_candidates?.[0];
                      const isOpen = p.state.toLowerCase() === "open";
                      const isExpanded = expandedPort === p.port;

                      return (
                        <Fragment key={`port-${p.port}`}>
                          <tr
                            onClick={() => isOpen && setExpandedPort(isExpanded ? null : p.port)}
                            className={`transition-colors ${isOpen ? "cursor-pointer hover:bg-white/[0.03]" : "opacity-50"}`}
                          >
                            <td className="px-5 py-3 text-gray-600">
                              {isOpen ? (isExpanded ? <ChevronDown className="h-3.5 w-3.5" /> : <ChevronRight className="h-3.5 w-3.5" />) : null}
                            </td>
                            <td className="px-5 py-3 font-mono text-gray-300 font-bold text-xs">{p.port}</td>
                            <td className="px-5 py-3">
                              <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold border ${getStatusColor(p.state)}`}>
                                {getStatusIcon(p.state)}
                                {p.state.toUpperCase()}
                              </span>
                            </td>
                            <td className="px-5 py-3 text-xs">
                              {top ? (
                                <span className="text-purple-300 font-medium">{top.service_name}</span>
                              ) : (
                                <span className="text-gray-600">—</span>
                              )}
                            </td>
                            <td className="px-5 py-3">
                              {top ? (
                                <div className="flex items-center gap-2">
                                  <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold border ${getDecisionBadge(top.decision)}`}>
                                    {getDecisionIcon(top.decision)}
                                    {top.decision.toUpperCase()}
                                  </span>
                                  {top.decision !== "verified" && (
                                    <button
                                      type="button"
                                      disabled={verifyingPorts[p.port]}
                                      onClick={(e) => handleVerifyPort(p, e)}
                                      className="flex items-center gap-1 px-1.5 py-0.5 bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 rounded text-[9px] font-bold uppercase transition-colors disabled:opacity-50"
                                      title="Perform Active Manual Verification"
                                    >
                                      {verifyingPorts[p.port] ? (
                                        <Loader2 key="loading" className="w-2.5 h-2.5 animate-spin" />
                                      ) : (
                                        <Activity key="static" className="w-2.5 h-2.5" />
                                      )}
                                      <span>Test</span>
                                    </button>
                                  )}
                                </div>
                              ) : (
                                <span className="text-gray-600 text-xs">—</span>
                              )}
                            </td>
                            <td className="px-5 py-3">
                              {top ? (
                                <div className="flex items-center gap-2 group relative">
                                  <div className="w-16 h-1.5 bg-white/5 rounded-full overflow-hidden">
                                    <div
                                      className={`h-full rounded-full transition-all ${getConfidenceColor(top.confidence_breakdown?.final_score ?? (top as any).confidence_score ?? 0)}`}
                                      style={{ width: `${Math.round((top.confidence_breakdown?.final_score ?? (top as any).confidence_score ?? 0) * 100)}%` }}
                                    />
                                  </div>
                                  <span className="text-[10px] text-gray-400 font-mono">{Math.round((top.confidence_breakdown?.final_score ?? (top as any).confidence_score ?? 0) * 100)}%</span>
                                </div>
                              ) : (
                                <span className="text-gray-600 text-xs">—</span>
                              )}
                            </td>
                            <td className="px-5 py-3">
                              {top ? (
                                <span className={`px-2 py-0.5 rounded text-[10px] font-semibold border ${getProbeMethodBadge(top.probe_method)}`}>
                                  {top.probe_method.toUpperCase()}
                                </span>
                              ) : (
                                <span className="text-gray-600 text-xs">—</span>
                              )}
                            </td>
                            <td className="px-5 py-3 text-xs">
                              {p.latency_ms != null ? (
                                <span className="text-gray-400 font-mono">{p.latency_ms}ms</span>
                              ) : (
                                <span className="text-gray-600">—</span>
                              )}
                            </td>
                            <td className="px-5 py-3">
                              <div className="flex items-center gap-1.5">
                                {p.fallback_used && (
                                  <span className="text-orange-400" title="Fallback used"><ArrowDownRight className="h-3.5 w-3.5" /></span>
                                )}
                                {p.retry_count > 0 && (
                                  <span className="text-yellow-400" title={`${p.retry_count} retry`}><RotateCcw className="h-3.5 w-3.5" /></span>
                                )}
                                {top?.tls_summary?.has_tls && (
                                  <span className="text-blue-400" title="TLS"><Shield className="h-3.5 w-3.5" /></span>
                                )}
                                {top?.ambiguity && (
                                  <span className="text-red-400" title="Ambiguous"><AlertTriangle className="h-3.5 w-3.5" /></span>
                                )}
                              </div>
                            </td>
                          </tr>

                          {/* ── Expanded Detail Row ── */}
                          {isOpen && isExpanded && (
                            <tr>
                              <td colSpan={9} className="px-5 py-4 bg-black/30">
                                <div className="space-y-4">

                                  {/* Reasoning banner */}
                                  {top?.reasoning && (
                                    <div className="flex items-start gap-2 bg-indigo-500/5 border border-indigo-500/10 rounded-xl p-3">
                                      <Brain className="h-4 w-4 text-indigo-400 mt-0.5 shrink-0" />
                                      <p className="text-xs text-indigo-300 leading-relaxed">{top.reasoning}</p>
                                    </div>
                                  )}

                                  {/* Edge CDN Warning */}
                                  {top?.decision === "cdn_edge" && (
                                    <div className="flex items-start gap-2 bg-orange-500/10 border border-orange-500/20 rounded-xl p-3">
                                      <Globe className="h-4 w-4 text-orange-400 mt-0.5 shrink-0" />
                                      <div>
                                        <p className="text-xs text-orange-400 font-bold uppercase tracking-wide">Edge Detected — Origin Unknown</p>
                                        <p className="text-xs text-orange-300/80 mt-1">This service appears to be hosted behind a CDN or proxy layer. Verified direct connection to the origin cannot be established.</p>
                                      </div>
                                    </div>
                                  )}

                                  {/* Verification Trail */}
                                  {top?.verification_trail && top.verification_trail.length > 0 && (
                                    <div className="bg-black/40 rounded-xl border border-white/5 p-4">
                                      <h4 className="text-gray-300 font-semibold text-[11px] uppercase tracking-wider mb-3 flex items-center gap-1.5">
                                        <Activity className="h-3.5 w-3.5" /> Verification Trail
                                      </h4>
                                      <div className="space-y-3 relative before:absolute before:inset-0 before:ml-2 before:-translate-x-px md:before:mx-auto md:before:translate-x-0 before:h-full before:w-0.5 before:bg-gradient-to-b before:from-transparent before:via-white/5 before:to-transparent">
                                        {top.verification_trail.map((step, idx) => (
                                          <div key={idx} className="relative flex items-start justify-between md:justify-normal md:odd:flex-row-reverse group is-active">
                                            <div className="flex items-center justify-center w-4 h-4 rounded-full border border-white/10 bg-black text-gray-500 shadow shrink-0 md:order-1 md:group-odd:-translate-x-1/2 md:group-even:translate-x-1/2 mt-1">
                                              <span className="w-1.5 h-1.5 bg-gray-500 rounded-full"></span>
                                            </div>
                                            <div className="w-[calc(100%-2rem)] md:w-[calc(50%-1.5rem)] bg-white/5 p-2 rounded border border-white/5">
                                              <p className="text-[10px] font-bold text-gray-300 uppercase">{step.step}</p>
                                              <p className="text-[10px] text-gray-500 mt-1">{step.detail}</p>
                                            </div>
                                          </div>
                                        ))}
                                      </div>
                                    </div>
                                  )}

                                  {/* Ambiguity warning */}
                                  {top?.ambiguity && (
                                    <div className="flex items-start gap-2 bg-red-500/5 border border-red-500/10 rounded-xl p-3">
                                      <AlertTriangle className="h-4 w-4 text-red-400 mt-0.5 shrink-0" />
                                      <div>
                                        <p className="text-xs text-red-300 font-medium">{top.ambiguity.description}</p>
                                        {top.ambiguity.conflicting_evidence.length > 0 && (
                                          <ul className="mt-1.5 space-y-0.5">
                                            {top.ambiguity.conflicting_evidence.map((c, ci) => (
                                              <li key={ci} className="text-[10px] text-red-400/70">• {c}</li>
                                            ))}
                                          </ul>
                                        )}
                                      </div>
                                    </div>
                                  )}

                                  <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 text-xs">
                                    {/* TLS Summary */}
                                    {top?.tls_summary?.has_tls && (
                                      <div className="bg-blue-500/5 border border-blue-500/10 rounded-xl p-4">
                                        <h4 className="text-blue-400 font-semibold text-[11px] uppercase tracking-wider mb-3 flex items-center gap-1.5">
                                          <Shield className="h-3.5 w-3.5" /> TLS Details
                                        </h4>
                                        <div className="space-y-1.5 text-gray-400">
                                          {top.tls_summary.protocol_version && <div>Protocol: <span className="text-gray-200">{top.tls_summary.protocol_version}</span></div>}
                                          {top.tls_summary.cipher_suite && <div>Cipher: <span className="text-gray-200 break-all">{top.tls_summary.cipher_suite}</span></div>}
                                          {top.tls_summary.subject && <div>Subject: <span className="text-gray-200">{top.tls_summary.subject}</span></div>}
                                          {top.tls_summary.issuer && <div>Issuer: <span className="text-gray-200">{top.tls_summary.issuer}</span></div>}
                                          {top.tls_summary.alpn && <div>ALPN: <span className="text-gray-200">{top.tls_summary.alpn}</span></div>}
                                        </div>
                                      </div>
                                    )}

                                    {/* HTTP Summary */}
                                    {top?.http_summary && (
                                      <div className="bg-green-500/5 border border-green-500/10 rounded-xl p-4">
                                        <h4 className="text-green-400 font-semibold text-[11px] uppercase tracking-wider mb-3 flex items-center gap-1.5">
                                          <Globe className="h-3.5 w-3.5" /> HTTP Details
                                        </h4>
                                        <div className="space-y-1.5 text-gray-400">
                                          {top.http_summary.status_code && <div>Status: <span className="text-gray-200">{top.http_summary.status_code}</span></div>}
                                          {top.http_summary.server_header && <div>Server: <span className="text-gray-200">{top.http_summary.server_header}</span></div>}
                                          {top.http_summary.content_type && <div>Content-Type: <span className="text-gray-200">{top.http_summary.content_type}</span></div>}
                                          {top.http_summary.redirect_target && <div>Redirect: <span className="text-gray-200">{top.http_summary.redirect_target}</span></div>}
                                          {top.http_summary.response_length !== undefined && <div>Length: <span className="text-gray-200">{top.http_summary.response_length} bytes</span></div>}
                                          {top.http_summary.headers && Object.keys(top.http_summary.headers).length > 0 && (
                                            <div className="mt-2 text-[10px] text-gray-500 font-mono border-t border-white/5 pt-2">
                                              {Object.entries(top.http_summary.headers).map(([k, v]) => (
                                                <div key={k} className="flex"><span className="text-gray-400 w-24 truncate">{k}:</span><span className="text-gray-300 w-full truncate">{v}</span></div>
                                              ))}
                                            </div>
                                          )}
                                        </div>
                                      </div>
                                    )}

                                    {/* Supporting Evidence */}
                                    <div className="bg-emerald-500/5 border border-emerald-500/10 rounded-xl p-4">
                                      <h4 className="text-emerald-400 font-semibold text-[11px] uppercase tracking-wider mb-3 flex items-center gap-1.5">
                                        <CheckCircle2 className="h-3.5 w-3.5" /> Supporting Evidence
                                      </h4>
                                      <div className="space-y-2 max-h-40 overflow-y-auto custom-scrollbar">
                                        {top?.supporting_evidence?.map((ev, eidx) => (
                                          <div key={eidx} className="bg-black/30 rounded-lg p-2 border border-white/5">
                                            <div className="flex items-center gap-1.5 mb-1">
                                              <span className={`w-1.5 h-1.5 rounded-full ${getStrengthDot(ev.strength)}`} />
                                              <span className={`px-1.5 py-0.5 rounded text-[9px] font-bold border ${getProbeMethodBadge(ev.source)}`}>
                                                {getEvidenceKindLabel(ev.kind)}
                                              </span>
                                              <span className="text-[9px] text-gray-600 uppercase">{ev.strength}</span>
                                              {ev.suggests_service && (
                                                <span className="text-[9px] text-purple-400 ml-auto">→ {ev.suggests_service}</span>
                                              )}
                                            </div>
                                            <p className="text-gray-400 text-[10px] leading-relaxed">{ev.interpretation}</p>
                                            <p className="text-gray-600 mt-0.5 font-mono text-[9px] break-all">{ev.raw_signal}</p>
                                          </div>
                                        ))}
                                        {(!top?.supporting_evidence || top.supporting_evidence.length === 0) && (
                                          <span className="text-gray-600 text-[10px]">No supporting evidence</span>
                                        )}
                                      </div>
                                    </div>
                                  </div>

                                  {/* Conflicting Evidence (if any) */}
                                  {top?.conflicting_evidence && top.conflicting_evidence.length > 0 && (
                                    <div className="bg-red-500/5 border border-red-500/10 rounded-xl p-4">
                                      <h4 className="text-red-400 font-semibold text-[11px] uppercase tracking-wider mb-3 flex items-center gap-1.5">
                                        <AlertTriangle className="h-3.5 w-3.5" /> Conflicting Evidence
                                      </h4>
                                      <div className="space-y-2">
                                        {top.conflicting_evidence.map((ev, eidx) => (
                                          <div key={eidx} className="bg-black/30 rounded-lg p-2 border border-red-500/10">
                                            <div className="flex items-center gap-1.5 mb-1">
                                              <span className="w-1.5 h-1.5 rounded-full bg-red-400" />
                                              <span className="text-[9px] text-red-400 font-bold">{getEvidenceKindLabel(ev.kind)}</span>
                                            </div>
                                            <p className="text-red-300/70 text-[10px]">{ev.interpretation}</p>
                                          </div>
                                        ))}
                                      </div>
                                    </div>
                                  )}

                                  {/* Fingerprint Match Details — Phase 5 */}
                                  {top?.fingerprint_match && (
                                    <div className="bg-cyan-500/5 border border-cyan-500/10 rounded-xl p-4 space-y-3">
                                      {/* Header with badges */}
                                      <div className="flex items-center gap-2 flex-wrap">
                                        <Database className="h-3.5 w-3.5 text-cyan-400" />
                                        <span className="text-cyan-400 font-semibold text-[11px] uppercase tracking-wider">Fingerprint Match</span>
                                        {/* Tier badge */}
                                        <span className={`px-1.5 py-0.5 rounded text-[9px] font-bold border ${
                                          top.fingerprint_match.tier === "specific" ? "bg-violet-500/20 text-violet-400 border-violet-500/30" :
                                          top.fingerprint_match.tier === "generic" ? "bg-amber-500/20 text-amber-400 border-amber-500/30" :
                                          "bg-gray-500/20 text-gray-400 border-gray-500/30"
                                        }`}>
                                          {top.fingerprint_match.tier.toUpperCase()}
                                        </span>
                                        {/* Strength badge */}
                                        <span className={`px-1.5 py-0.5 rounded text-[9px] font-bold border ${
                                          top.fingerprint_match.strength === "full" ? "bg-emerald-500/20 text-emerald-400 border-emerald-500/30" :
                                          top.fingerprint_match.strength === "strong" ? "bg-blue-500/20 text-blue-400 border-blue-500/30" :
                                          top.fingerprint_match.strength === "partial" ? "bg-yellow-500/20 text-yellow-400 border-yellow-500/30" :
                                          "bg-gray-500/20 text-gray-400 border-gray-500/30"
                                        }`}>
                                          {top.fingerprint_match.strength.toUpperCase()}
                                        </span>
                                        {/* Confidence level badge */}
                                        <span className={`px-1.5 py-0.5 rounded text-[9px] font-bold border ${
                                          top.fingerprint_match.confidence_level === "confirmed" ? "bg-emerald-500/20 text-emerald-300 border-emerald-500/30" :
                                          top.fingerprint_match.confidence_level === "high" ? "bg-blue-500/20 text-blue-300 border-blue-500/30" :
                                          top.fingerprint_match.confidence_level === "medium" ? "bg-yellow-500/20 text-yellow-300 border-yellow-500/30" :
                                          top.fingerprint_match.confidence_level === "low" ? "bg-orange-500/20 text-orange-300 border-orange-500/30" :
                                          "bg-red-500/20 text-red-300 border-red-500/30"
                                        }`}>
                                          {top.fingerprint_match.confidence_level.toUpperCase()}
                                        </span>
                                        {/* Coverage badge */}
                                        <span className="px-1.5 py-0.5 rounded text-[9px] font-mono text-gray-500 border border-white/5 bg-white/5">
                                          COV: {top.fingerprint_match.coverage}
                                        </span>
                                        <span className="ml-auto text-[10px] text-gray-500 font-mono">{top.fingerprint_match.fingerprint_id}</span>
                                      </div>

                                      {/* Explanation Items */}
                                      {top.fingerprint_match.explanation_items.length > 0 && (
                                        <div className="bg-black/20 rounded-lg p-3 border border-white/5">
                                          <div className="text-[10px] text-gray-500 font-semibold uppercase tracking-wider mb-2">Scoring Explanation</div>
                                          <div className="space-y-1">
                                            {top.fingerprint_match.explanation_items.map((ex, ei) => (
                                              <div key={ei} className="flex items-start gap-2 text-[10px]">
                                                <span className={`shrink-0 mt-0.5 w-1.5 h-1.5 rounded-full ${
                                                  ex.category === "boost" ? "bg-emerald-400" :
                                                  ex.category === "decay" ? "bg-orange-400" :
                                                  ex.category === "penalty" ? "bg-red-400" :
                                                  ex.category === "contextual" ? "bg-violet-400" :
                                                  "bg-gray-500"
                                                }`} />
                                                <span className={`${
                                                  ex.category === "boost" ? "text-emerald-400" :
                                                  ex.category === "decay" ? "text-orange-400" :
                                                  ex.category === "penalty" ? "text-red-400" :
                                                  ex.category === "contextual" ? "text-violet-400" :
                                                  "text-gray-500"
                                                }`}>{ex.description}</span>
                                                {ex.impact !== 0 && (
                                                  <span className={`ml-auto font-mono shrink-0 ${ex.impact > 0 ? "text-emerald-400" : "text-red-400"}`}>
                                                    {ex.impact > 0 ? "+" : ""}{ex.impact.toFixed(2)}
                                                  </span>
                                                )}
                                              </div>
                                            ))}
                                          </div>
                                        </div>
                                      )}

                                      {/* Penalties */}
                                      {top.fingerprint_match.penalties.length > 0 && (
                                        <div className="bg-red-500/5 rounded-lg p-3 border border-red-500/10">
                                          <div className="text-[10px] text-red-400 font-semibold uppercase tracking-wider mb-2">Penalties Applied</div>
                                          <div className="space-y-1">
                                            {top.fingerprint_match.penalties.map((pen, pi) => (
                                              <div key={pi} className="flex items-center gap-2 text-[10px]">
                                                <span className="w-1.5 h-1.5 rounded-full bg-red-400 shrink-0" />
                                                <span className="text-red-300/70">{pen.reason}</span>
                                                <span className="text-red-400 font-mono ml-auto">-{pen.amount.toFixed(2)}</span>
                                              </div>
                                            ))}
                                          </div>
                                        </div>
                                      )}

                                      {/* Rules breakdown */}
                                      <div className="space-y-1">
                                        {top.fingerprint_match.matched_rules.map((r, ri) => (
                                          <div key={ri} className="flex items-center gap-2 text-[10px]">
                                            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 shrink-0" />
                                            <span className={`px-1 py-0.5 rounded text-[8px] font-bold uppercase border ${
                                              r.rule_weight === "critical" ? "text-amber-400 border-amber-500/30 bg-amber-500/10" :
                                              r.rule_weight === "strong" ? "text-blue-400 border-blue-500/30 bg-blue-500/10" :
                                              r.rule_weight === "medium" ? "text-gray-400 border-gray-500/30 bg-gray-500/10" :
                                              "text-gray-600 border-gray-700/30 bg-gray-700/10"
                                            }`}>{r.rule_weight}</span>
                                            <span className="text-gray-400">{r.category}</span>
                                            <span className="text-gray-600">→</span>
                                            <span className="text-gray-300 font-mono">{r.expected || "any"}</span>
                                            {r.actual && <span className="text-emerald-400 font-mono ml-auto">✓ {r.actual}</span>}
                                          </div>
                                        ))}
                                        {top.fingerprint_match.missing_rules.map((r, ri) => (
                                          <div key={`m-${ri}`} className="flex items-center gap-2 text-[10px]">
                                            <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${r.skipped_contextual ? "bg-violet-500" : "bg-gray-600"}`} />
                                            <span className={`px-1 py-0.5 rounded text-[8px] font-bold uppercase border text-gray-600 border-gray-700/30 bg-gray-700/10`}>{r.rule_weight}</span>
                                            <span className="text-gray-600">{r.category}</span>
                                            <span className="text-gray-700">→</span>
                                            <span className="text-gray-600 font-mono">{r.expected}</span>
                                            <span className={`ml-auto ${r.skipped_contextual ? "text-violet-400" : "text-gray-600"}`}>
                                              {r.skipped_contextual ? "⊘ skipped" : "✗ missing"}
                                            </span>
                                          </div>
                                        ))}
                                        {top.fingerprint_match.conflicting_rules.map((r, ri) => (
                                          <div key={`c-${ri}`} className="flex items-center gap-2 text-[10px]">
                                            <span className="w-1.5 h-1.5 rounded-full bg-red-400 shrink-0" />
                                            <span className="text-red-400">{r.category}</span>
                                            {r.actual && <span className="text-red-400/70 font-mono ml-auto">{r.actual}</span>}
                                          </div>
                                        ))}
                                      </div>
                                    </div>
                                  )}

                                  {/* All Evidence (port-level) */}
                                  {p.all_evidence && p.all_evidence.length > 0 && (
                                    <div className="bg-purple-500/5 border border-purple-500/10 rounded-xl p-4">
                                      <h4 className="text-purple-400 font-semibold text-[11px] uppercase tracking-wider mb-3 flex items-center gap-1.5">
                                        <Eye className="h-3.5 w-3.5" /> All Evidence ({p.all_evidence.length} items)
                                      </h4>
                                      <div className="flex flex-wrap gap-1.5">
                                        {p.all_evidence.map((ev, eidx) => (
                                          <span
                                            key={eidx}
                                            className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-lg text-[10px] border ${ev.is_negative ? "bg-red-500/10 text-red-400 border-red-500/20" : "bg-white/5 text-gray-400 border-white/10"}`}
                                            title={ev.raw_signal}
                                          >
                                            <span className={`w-1.5 h-1.5 rounded-full ${getStrengthDot(ev.strength)}`} />
                                            {getEvidenceKindLabel(ev.kind)}
                                            {ev.suggests_service && <span className="text-purple-400">→ {ev.suggests_service}</span>}
                                          </span>
                                        ))}
                                      </div>
                                    </div>
                                  )}

                                  {/* Alternative Candidates */}
                                  {p.service_candidates.length > 1 && (
                                    <div className="bg-black/20 border border-white/5 rounded-xl p-4">
                                      <h4 className="text-gray-400 font-semibold text-[11px] uppercase tracking-wider mb-3">Alternative Candidates</h4>
                                      <div className="space-y-2">
                                        {p.service_candidates.slice(1).map((c, cidx) => (
                                          <div key={cidx} className="flex items-center gap-3 bg-black/20 rounded-lg p-2.5 border border-white/5">
                                            <span className="text-gray-300 font-medium text-xs">{c.service_name}</span>
                                            <span className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[9px] font-bold border ${getDecisionBadge(c.decision)}`}>
                                              {getDecisionIcon(c.decision)}
                                              {c.decision.toUpperCase()}
                                            </span>
                                            <div className="flex items-center gap-1.5">
                                              <div className="w-12 h-1 bg-white/5 rounded-full overflow-hidden">
                                                <div
                                                  className={`h-full rounded-full ${getConfidenceColor(c.confidence_breakdown?.final_score ?? (c as any).confidence_score ?? 0)}`}
                                                  style={{ width: `${Math.round((c.confidence_breakdown?.final_score ?? (c as any).confidence_score ?? 0) * 100)}%` }}
                                                />
                                              </div>
                                              <span className="text-[9px] text-gray-500 font-mono">{Math.round((c.confidence_breakdown?.final_score ?? (c as any).confidence_score ?? 0) * 100)}%</span>
                                            </div>
                                            <p className="text-[10px] text-gray-500 ml-auto max-w-xs truncate" title={c.reasoning}>{c.reasoning}</p>
                                          </div>
                                        ))}
                                      </div>
                                    </div>
                                  )}
                                </div>
                              </td>
                            </tr>
                          )}
                        </Fragment>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>

            {/* ── Activity Timeline ── */}
            <div className="bg-card-bg border border-card-border p-5 rounded-2xl">
              <h3 className="font-semibold text-white text-sm tracking-wide mb-5 flex items-center gap-2">
                <Activity className="h-4 w-4 text-purple-400" />
                Execution Timeline
                <span className="ml-auto text-[10px] text-gray-600 font-mono">{data.activity_timeline?.length} events</span>
              </h3>
              <div className="space-y-2 max-h-[350px] overflow-y-auto pr-2 custom-scrollbar">
                {data.activity_timeline?.map((act, i) => (
                  <div key={i} className="flex gap-3 items-start group">
                    <div className="mt-0.5 text-gray-600 font-mono text-[10px] whitespace-nowrap w-16 shrink-0">
                      {new Date(act.timestamp).toLocaleTimeString([], { hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit" })}
                    </div>
                    <span className={`mt-0.5 px-1.5 py-0.5 rounded text-[9px] font-bold shrink-0 ${getEventTypeColor(act.event_type)}`}>
                      {act.event_type}
                    </span>
                    <div className="flex-1 pb-2 border-b border-white/5">
                      <p className={`text-xs ${
                        act.severity === "error" ? "text-red-400" :
                        act.severity === "warning" ? "text-yellow-400" : "text-gray-400"
                      }`}>
                        {act.message}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            </div>

          </motion.div>
        )}
      </AnimatePresence>

      <style jsx global>{`
        @keyframes shimmer { 100% { transform: translateX(100%); } }
        .custom-scrollbar::-webkit-scrollbar { width: 3px; }
        .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
        .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); border-radius: 10px; }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: rgba(255,255,255,0.15); }
      `}</style>
    </div>
  );
}
