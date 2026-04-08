"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { TerminalSquare, Loader2, Globe, ShieldAlert, Server, Activity, CornerDownRight, Box, Fingerprint, Info, Network, Workflow, ShieldCheck, AlertTriangle, Shield } from "lucide-react";
import { reportApi } from "@/utils/api";
import { useLanguage } from "@/context/LanguageContext";
import { CertaintyBadge } from "@/components/CertaintyBadge";

interface InvestigatorFingerprint {
  name: string;
  category: string;
  confidence_score: number;
  evidences: string[];
  explanation: string;
}

interface DeliveryInsight {
  name: string;
  category: string;
  confidence_score: number;
  evidence: string;
  explanation: string;
}

interface SecurityPostureInsight {
  name: string;
  category: string;
  status: string; // "Secure", "Warning", "Critical", "Informational"
  confidence_score: number;
  evidence: string;
  explanation: string;
}

interface ConsistencyInsight {
  name: string;
  severity: string;
  category: string;
  evidences: string[];
  explanation: string;
}

interface InfrastructureSignal {
  signal_type: string;
  value: string;
  evidence: string;
}

interface ServerInfo {
  original_target: string;
  resolved_url: string;
  status_code: number;
  latency_ms: number;
  raw_headers: Record<string, string[]>;
  categorized_headers: Record<string, Record<string, string[]>>;
  infrastructure_signals: InfrastructureSignal[];
  fingerprints: InvestigatorFingerprint[];
  delivery_insights: DeliveryInsight[];
  security_insights: SecurityPostureInsight[];
  routes_checked: string[];
  consistency_insights: ConsistencyInsight[];
  activity_log: string[];
  investigation_certainty?: { level: string; reason: string } | null;
}

export default function InvestigatePage() {
  const [url, setUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [data, setData] = useState<ServerInfo | null>(null);
  const [error, setError] = useState("");
  const { t } = useLanguage();

  const handleInvestigate = (e: React.FormEvent) => {
    e.preventDefault();
    if (!url) return;
    setIsLoading(true);
    setError("");
    setData({
      original_target: url,
      resolved_url: "Resolving stream...",
      status_code: 0,
      latency_ms: 0,
      raw_headers: {},
      categorized_headers: {},
      infrastructure_signals: [],
      fingerprints: [],
      delivery_insights: [],
      security_insights: [],
      routes_checked: [],
      consistency_insights: [],
      activity_log: []
    } as any);

    const targetUrl = url.startsWith("http") ? url : `https://${url}`;
    const encodedUrl = encodeURIComponent(targetUrl);
    const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8900';
    const es = new EventSource(`${apiUrl}/investigate/stream?url=${encodedUrl}`);

    const events = [
      "INVESTIGATION_STARTED", "HEADERS_NORMALIZED", "INFRA_SIGNAL_DETECTED",
      "CMS_FINGERPRINT_MATCHED", "CACHE_BEHAVIOR_ANALYZED", "SECURITY_SIGNAL_EVALUATED",
      "ROUTE_COMPARED", "INVESTIGATION_COMPLETED"
    ];

    events.forEach(evt => {
       es.addEventListener(evt, (e: any) => {
           let parsedEvent;
           try {
               parsedEvent = JSON.parse(e.data);
           } catch(err) { return; }

           const payload = parsedEvent.payload;

           setData(prev => {
               if(!prev) return prev;
               let next = { ...prev };
               
               let timestamp = new Date().toLocaleTimeString();
               next.activity_log = [...(next.activity_log || []), `[${timestamp}] ${parsedEvent.message}`];

               switch(evt) {
                   case "HEADERS_NORMALIZED":
                       next.raw_headers = payload || {};
                       break;
                   case "INFRA_SIGNAL_DETECTED":
                       next.infrastructure_signals = payload || [];
                       break;
                   case "CMS_FINGERPRINT_MATCHED":
                       next.fingerprints = payload || [];
                       break;
                   case "CACHE_BEHAVIOR_ANALYZED":
                       next.delivery_insights = payload || [];
                       break;
                   case "SECURITY_SIGNAL_EVALUATED":
                       next.security_insights = payload || [];
                       break;
                   case "ROUTE_COMPARED":
                       next.consistency_insights = payload || [];
                       break;
                   case "INVESTIGATION_COMPLETED":
                       next = payload;
                       setIsLoading(false);
                       es.close();
                       break;
               }

               return next;
           });
       });
    });

    es.onerror = (err) => {
        // usually triggers when stream finishes or backend drops
        setIsLoading(false);
        es.close();
    };
  };

  const fingerprintsByCategory = (data?.fingerprints || []).reduce((acc, fp) => {
    if (!acc[fp.category]) acc[fp.category] = [];
    acc[fp.category].push(fp);
    return acc;
  }, {} as Record<string, InvestigatorFingerprint[]>) || {};

  const deliveryInsightsByCategory = (data?.delivery_insights || []).reduce((acc, di) => {
    if (!acc[di.category]) acc[di.category] = [];
    acc[di.category].push(di);
    return acc;
  }, {} as Record<string, DeliveryInsight[]>) || {};

  const securityInsightsByCategory = (data?.security_insights || []).reduce((acc, si) => {
    if (!acc[si.category]) acc[si.category] = [];
    acc[si.category].push(si);
    return acc;
  }, {} as Record<string, SecurityPostureInsight[]>) || {};

  const getStatusBadge = (status: string) => {
      switch (status) {
          case 'Secure': return <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-[10px] uppercase font-bold tracking-wider"><ShieldCheck className="w-3 h-3" /> {status}</div>;
          case 'Warning': return <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-yellow-500/10 border border-yellow-500/20 text-yellow-400 text-[10px] uppercase font-bold tracking-wider"><AlertTriangle className="w-3 h-3" /> {status}</div>;
          case 'Critical': return <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-red-500/10 border border-red-500/20 text-red-400 text-[10px] uppercase font-bold tracking-wider"><ShieldAlert className="w-3 h-3" /> {status}</div>;
          case 'Informational': return <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-blue-500/10 border border-blue-500/20 text-blue-400 text-[10px] uppercase font-bold tracking-wider"><Info className="w-3 h-3" /> {status}</div>;
          default: return null;
      }
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-700 max-w-6xl">
      <header className="mb-10">
        <h1 className="text-4xl font-extrabold tracking-tight text-white flex items-center gap-3">
          {t.invTitle}
          <TerminalSquare className="h-8 w-8 text-accent-blue" />
        </h1>
        <p className="mt-2 text-gray-400 text-lg">Phase 5: Concurrent Multi-Route Profiling mapping Deep Infrastructure Consistency.</p>
      </header>

      <form onSubmit={handleInvestigate} className="relative group max-w-2xl">
        <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
          <Globe className="h-5 w-5 text-gray-400 group-focus-within:text-accent-blue transition-colors" />
        </div>
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder={t.enterUrl}
          className="block w-full pl-12 pr-32 py-4 bg-sidebar-bg/50 border border-sidebar-border rounded-2xl text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-accent-blue/50 focus:border-accent-blue transition-all shadow-xl shadow-black/20"
        />
        <button type="submit" disabled={isLoading || !url} className="absolute right-2 top-2 bottom-2 px-6 bg-gradient-to-r from-accent-blue to-blue-600 hover:from-blue-500 hover:to-blue-700 rounded-xl font-semibold text-white flex items-center gap-2 transition-all disabled:opacity-50 transform hover:scale-[1.02] active:scale-95 shadow-lg shadow-blue-500/20">
          {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <TerminalSquare className="h-4 w-4" />}
          {isLoading ? <span>{t.investigating}</span> : t.investigate}
        </button>
      </form>

      {error && (
        <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="p-4 bg-red-950/30 border border-red-500/50 rounded-xl text-red-200 flex items-center gap-3 max-w-2xl">
          <ShieldAlert className="h-5 w-5 text-red-400" />
          {error}
        </motion.div>
      )}

      {isLoading && !data?.raw_headers && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="mt-16 min-h-[300px] rounded-2xl bg-sidebar-bg/50 border border-sidebar-border relative overflow-hidden flex items-center justify-center">
            <div className="flex flex-col items-center gap-4 text-accent-blue">
                <Loader2 className="h-10 w-10 animate-spin" />
                <p>Establishing Streaming Connection...</p>
            </div>
          <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent -translate-x-full animate-[shimmer_1.5s_infinite]"></div>
        </motion.div>
      )}

      <AnimatePresence>
        {data && (
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="mt-12 space-y-8 relative">
            
            {/* Live Progress Bar */}
            {isLoading && (
               <div className="sticky top-0 z-50 bg-black/80 backdrop-blur-md border border-white/10 rounded-xl p-4 mb-8 shadow-2xl flex items-center justify-between">
                   <div className="flex items-center gap-3">
                       <Loader2 className="w-5 h-5 animate-spin text-accent-blue" />
                       <span className="font-semibold text-white tracking-wide">Live Investigation In Progress...</span>
                   </div>
                   <div className="text-xs font-mono text-gray-400">
                      Receiving SSE Stream Updates {data.activity_log.length > 0 ? `(${data.activity_log.length} Events)` : ''}
                   </div>
               </div>
            )}
            
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              {/* Overview */}
              <div className="bg-card-bg border border-card-border p-6 rounded-2xl relative overflow-hidden md:col-span-1 shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]">
                <div className="flex items-center gap-3 mb-6 relative z-10">
                  <div className="p-2 rounded-lg bg-black/40 border border-white/5 text-accent-blue"><Globe className="h-5 w-5" /></div>
                  <h3 className="font-semibold text-white tracking-wide">Target Overview</h3>
                </div>
                <div className="space-y-4">
                  <div>
                    <p className="text-xs text-gray-500 font-mono uppercase tracking-wider mb-1">Resolved URL</p>
                    <p className="text-sm font-semibold text-gray-200 truncate" title={data.resolved_url}>{data.resolved_url}</p>
                  </div>
                  <div className="flex justify-between items-center bg-black/30 p-3 rounded-lg border border-white/5">
                    <div>
                        <p className="text-xs text-gray-500 font-mono uppercase tracking-wider mb-1">Status Code</p>
                        <p className={`text-lg font-semibold ${data.status_code < 400 ? 'text-green-400' : 'text-red-400'}`}>{data.status_code}</p>
                    </div>
                    <div className="text-right">
                        <p className="text-xs text-gray-500 font-mono uppercase tracking-wider mb-1">Latency</p>
                        <p className="text-lg font-semibold text-blue-400">{data.latency_ms} <span className="text-sm">ms</span></p>
                    </div>
                  </div>
                </div>
              </div>

               {/* Activity Log */}
               <div className="bg-card-bg border border-card-border p-6 rounded-2xl relative overflow-hidden md:col-span-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.05)] flex flex-col">
                <div className="flex items-center gap-3 mb-6 relative z-10">
                  <div className="p-2 rounded-lg bg-black/40 border border-white/5 text-accent-blue"><Activity className="h-5 w-5" /></div>
                  <h3 className="font-semibold text-white tracking-wide">Activity Timeline</h3>
                </div>
                <div className="bg-black/60 rounded-xl p-4 border border-white/10 font-mono text-xs overflow-y-auto custom-scrollbar flex-1">
                    {data.activity_log.map((log, i) => (
                        <div key={i} className="flex gap-4 p-1.5 text-gray-300">
                            <span className="text-blue-500/50">[{new Date().toLocaleTimeString()}]</span>
                            <span className="text-blue-400 opacity-70"><CornerDownRight className="w-3 h-3 inline" /></span>
                            <span className="opacity-90">{log}</span>
                        </div>
                    ))}
                </div>
              </div>
            </div>

            {/* Investigation Certainty */}
            <CertaintyBadge certainty={data.investigation_certainty as any} />

            {/* Cross-Request Consistency Insights (Phase 5) */}
            <div className="space-y-4">
                <div className="flex items-center gap-3 mb-6">
                    <div className="p-2 rounded-lg bg-orange-500/10 border border-orange-500/20 text-orange-400"><Workflow className="h-5 w-5" /></div>
                    <h2 className="text-2xl font-bold text-white">Cross-Request Consistency</h2>
                    <span className={`py-1 px-3 text-sm rounded-full font-semibold border ${data.consistency_insights?.length === 0 ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30' : 'bg-orange-500/20 text-orange-300 border-orange-500/30'}`}>
                        {data.consistency_insights?.length === 0 ? 'Uniform Profile' : `${data.consistency_insights?.length} Inconsistencies`}
                    </span>
                </div>

                <div className="bg-black/30 border border-white/5 rounded-2xl p-4 mb-4 flex flex-col sm:flex-row gap-4 items-start sm:items-center">
                   <div className="text-sm font-mono text-gray-400 uppercase tracking-wider shrink-0">Routes Evaluated ({data.routes_checked?.length})</div>
                   <div className="flex flex-wrap gap-2">
                       {data.routes_checked?.map((route, i) => (
                           <span key={i} className="bg-white/5 border border-white/10 px-2 py-1 rounded text-xs text-gray-300 truncate max-w-[200px]" title={route}>{route.replace(/^https?:\/\/[^\/]+/, '') || '/'}</span>
                       ))}
                   </div>
                </div>

                {data.consistency_insights?.length > 0 ? (
                    <div className="grid grid-cols-1 gap-6">
                        {data.consistency_insights.map((insight, i) => (
                            <div key={i} className="bg-card-bg border border-card-border rounded-xl p-6 shadow-[inset_0_1px_0_rgba(255,255,255,0.05)] ml-4 border-l-4 border-l-orange-500">
                                <div className="flex justify-between items-start mb-2">
                                    <div>
                                        <h4 className="text-xl font-bold text-gray-100">{insight.name}</h4>
                                        <span className="text-xs text-orange-400 tracking-wider uppercase font-mono mt-1 block">{insight.category}</span>
                                    </div>
                                    {getStatusBadge(insight.severity.replace('Medium', 'Warning').replace('High', 'Critical'))} 
                                </div>
                                <p className="text-sm text-gray-400 mt-3 mb-4">{insight.explanation}</p>
                                <div className="bg-black/50 border border-white/5 rounded-xl p-3 text-xs font-mono">
                                    <span className="text-gray-500 block mb-2 uppercase tracking-widest pl-1">Identified Asymmetries On Routes</span>
                                    <ul className="space-y-1 pl-1">
                                        {insight.evidences.map((ev, eIdx) => (
                                            <li key={eIdx} className="text-gray-300 flex items-start gap-2">
                                                <span className="text-orange-500 mt-0.5">›</span> {ev}
                                            </li>
                                        ))}
                                    </ul>
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="p-8 bg-black/20 border border-dashed border-white/10 rounded-2xl text-center">
                        <ShieldCheck className="h-10 w-10 text-emerald-500/50 mx-auto mb-3" />
                        <p className="text-gray-400 font-medium">Target demonstrates robust infrastructure uniformity across all evaluated endpoints.</p>
                    </div>
                )}
            </div>

            {/* Security Posture Insights (Phase 4) */}
            <div className="space-y-4">
                <div className="flex items-center gap-3 mb-6">
                    <div className="p-2 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-400"><Shield className="h-5 w-5" /></div>
                    <h2 className="text-2xl font-bold text-white">Infrastructure Security Posture</h2>
                    <span className="bg-rose-500/20 text-rose-300 py-1 px-3 text-sm rounded-full font-semibold border border-rose-500/30">
                        {data.security_insights.length} Insight{data.security_insights.length !== 1 ? 's' : ''}
                    </span>
                </div>

                {data.security_insights.length > 0 ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {Object.entries(securityInsightsByCategory).map(([category, insights]) => (
                            <div key={category} className="bg-card-bg border border-card-border rounded-2xl overflow-hidden shadow-[inset_0_1px_0_rgba(255,255,255,0.05)] col-span-1">
                                <div className="bg-sidebar-bg/50 px-6 py-4 border-b border-sidebar-border">
                                    <h3 className="font-semibold text-rose-400 tracking-wide uppercase text-sm">{category}</h3>
                                </div>
                                <div className="p-6 space-y-6 bg-black/20">
                                    {insights.map((insight, i) => (
                                        <div key={i} className="space-y-4 border-b border-white/5 pb-6 last:border-0 last:pb-0">
                                             <div>
                                                <div className="flex justify-between items-start mb-2">
                                                    <h4 className="text-lg font-bold text-gray-100 pr-2">{insight.name}</h4>
                                                    {getStatusBadge(insight.status)}
                                                </div>
                                                <p className="text-xs text-gray-400 flex items-start gap-1"><Info className="w-3.5 h-3.5 min-w-[14px] mt-0.5 opacity-70" /> {insight.explanation}</p>
                                            </div>
                                            <div className="bg-black/50 border border-white/5 rounded-xl p-3 text-xs font-mono">
                                                <span className="text-gray-500 block mb-2 uppercase tracking-widest pl-1">Identified Evidence</span>
                                                <div className="text-gray-300 flex items-start gap-2 pl-1 break-words">
                                                    <span className="text-rose-500 mt-0.5">›</span> {insight.evidence}
                                                </div>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="p-8 bg-black/20 border border-dashed border-white/10 rounded-2xl text-center">
                        <ShieldCheck className="h-10 w-10 text-gray-600 mx-auto mb-3" />
                        <p className="text-gray-400 font-medium">No strict security parameters or blatant leaks discovered on this route.</p>
                    </div>
                )}
            </div>

            {/* Delivery Insights (Phase 3) */}
            <div className="space-y-4">
                <div className="flex items-center gap-3 mb-6">
                    <div className="p-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400"><Network className="h-5 w-5" /></div>
                    <h2 className="text-2xl font-bold text-white">Delivery Behavior & Caching</h2>
                    <span className="bg-emerald-500/20 text-emerald-300 py-1 px-3 text-sm rounded-full font-semibold border border-emerald-500/30">
                        {data.delivery_insights.length} Route Insight{data.delivery_insights.length !== 1 ? 's' : ''}
                    </span>
                </div>

                {data.delivery_insights.length > 0 ? (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {Object.entries(deliveryInsightsByCategory).map(([category, insights]) => (
                            <div key={category} className="bg-card-bg border border-card-border rounded-2xl overflow-hidden shadow-[inset_0_1px_0_rgba(255,255,255,0.05)] col-span-1">
                                <div className="bg-sidebar-bg/50 px-6 py-4 border-b border-sidebar-border">
                                    <h3 className="font-semibold text-emerald-400 tracking-wide uppercase text-sm">{category}</h3>
                                </div>
                                <div className="p-6 space-y-6 bg-black/20">
                                    {insights.map((insight, i) => (
                                        <div key={i} className="space-y-4 border-b border-white/5 pb-6 last:border-0 last:pb-0">
                                             <div>
                                                <div className="flex justify-between items-start mb-1">
                                                    <h4 className="text-lg font-bold text-gray-100">{insight.name}</h4>
                                                    <span className={`text-xs px-2 py-0.5 rounded-full border ${insight.confidence_score >= 90 ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400' : 'bg-yellow-500/10 border-yellow-500/20 text-yellow-400'}`}>{insight.confidence_score}%</span>
                                                </div>
                                                <p className="text-xs text-gray-400 flex items-start gap-1"><Info className="w-3.5 h-3.5 min-w-[14px] mt-0.5 opacity-70" /> {insight.explanation}</p>
                                            </div>
                                            <div className="bg-black/50 border border-white/5 rounded-xl p-3 text-xs font-mono">
                                                <span className="text-gray-500 block mb-2 uppercase tracking-widest pl-1">Network Trace</span>
                                                <div className="text-gray-300 flex items-start gap-2 pl-1 break-words">
                                                    <span className="text-emerald-500 mt-0.5">›</span> {insight.evidence}
                                                </div>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="p-8 bg-black/20 border border-dashed border-white/10 rounded-2xl text-center">
                        <Workflow className="h-10 w-10 text-gray-600 mx-auto mb-3" />
                        <p className="text-gray-400 font-medium">Standard direct routing. No specialized Edge CDN, Transit Cache, or Proxy behaviors isolated dynamically.</p>
                    </div>
                )}
            </div>

            {/* Platform Fingerprints (Phase 2) */}
            <div className="space-y-4">
                <div className="flex items-center gap-3 mb-6">
                    <div className="p-2 rounded-lg bg-purple-500/10 border border-purple-500/20 text-purple-400"><Fingerprint className="h-5 w-5" /></div>
                    <h2 className="text-2xl font-bold text-white">Platform Fingerprints</h2>
                    <span className="bg-purple-500/20 text-purple-300 py-1 px-3 text-sm rounded-full font-semibold border border-purple-500/30">
                        {data.fingerprints.length} Match{data.fingerprints.length !== 1 ? 'es' : ''}
                    </span>
                </div>

                {data.fingerprints.length > 0 ? (
                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                        {Object.entries(fingerprintsByCategory).map(([category, fps]) => (
                            <div key={category} className="bg-card-bg border border-card-border rounded-2xl overflow-hidden shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]">
                                <div className="bg-sidebar-bg/50 px-6 py-4 border-b border-sidebar-border">
                                    <h3 className="font-semibold text-accent-blue tracking-wide uppercase text-sm">{category}</h3>
                                </div>
                                <div className="p-6 space-y-6 bg-black/20">
                                    {fps.map((fp, i) => (
                                        <div key={i} className="space-y-4 border-b border-white/5 pb-6 last:border-0 last:pb-0">
                                            <div className="flex justify-between items-start">
                                                <div>
                                                    <h4 className="text-xl font-bold text-gray-100">{fp.name}</h4>
                                                    <p className="text-sm text-gray-400 mt-1 flex items-start gap-1"><Info className="w-4 h-4 mt-0.5 opacity-70" /> {fp.explanation}</p>
                                                </div>
                                                <div className="bg-black/60 border border-white/5 px-3 py-1.5 rounded-lg flex flex-col items-center">
                                                    <span className="text-[10px] text-gray-500 uppercase tracking-wider font-mono">Confidence</span>
                                                    <span className={`text-lg font-bold ${fp.confidence_score >= 80 ? 'text-green-400' : fp.confidence_score >= 50 ? 'text-yellow-400' : 'text-red-400'}`}>{fp.confidence_score.toFixed(0)}%</span>
                                                </div>
                                            </div>
                                            <div className="bg-black/50 border border-white/5 rounded-xl p-3 text-xs font-mono">
                                                <span className="text-gray-500 block mb-2 uppercase tracking-widest pl-1">Evidences Traversed</span>
                                                <ul className="space-y-1 pl-1">
                                                    {fp.evidences.map((ev, eIdx) => (
                                                        <li key={eIdx} className="text-gray-300 flex items-start gap-2">
                                                            <span className="text-blue-500 mt-0.5">›</span> {ev}
                                                        </li>
                                                    ))}
                                                </ul>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="p-8 bg-black/20 border border-dashed border-white/10 rounded-2xl text-center">
                        <Fingerprint className="h-10 w-10 text-gray-600 mx-auto mb-3" />
                        <p className="text-gray-400 font-medium">No definitive deployment or CMS fingerprints discovered via HTML/Cookie analysis.</p>
                    </div>
                )}
            </div>

            {/* Infrastructure Signals (Phase 1 retained) */}
            <div className="grid grid-cols-1 gap-6">
                <div className="bg-gradient-to-br from-sidebar-bg/80 to-black/60 border border-sidebar-border p-6 rounded-2xl relative overflow-hidden shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]">
                    <div className="flex items-center gap-3 mb-6 relative z-10">
                        <div className="p-2 rounded-lg bg-blue-500/10 border border-blue-500/20 text-accent-blue"><Server className="h-5 w-5" /></div>
                        <h3 className="font-semibold text-white tracking-wide">Infrastructure Header Signals</h3>
                    </div>
                    {data.infrastructure_signals.length > 0 ? (
                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                            {data.infrastructure_signals.map((signal, idx) => (
                                <div key={idx} className="bg-black/40 border border-white/10 p-4 rounded-xl flex flex-col group hover:border-blue-500/30 transition-colors">
                                    <div className="flex items-center gap-2 text-xs text-blue-400 font-mono mb-2 uppercase tracking-wider">
                                        <Box className="w-3 h-3" />
                                        {signal.signal_type}
                                    </div>
                                    <p className="text-xl font-bold text-gray-100 mb-3">{signal.value}</p>
                                    <div className="mt-auto pt-3 border-t border-white/5">
                                        <p className="text-xs text-gray-500 italic flex items-start gap-1.5">
                                            <span className="text-blue-500 font-bold mt-0.5">↳</span>
                                            {signal.evidence}
                                        </p>
                                    </div>
                                </div>
                            ))}
                        </div>
                    ) : (
                        <div className="p-6 bg-black/20 border border-dashed border-white/10 rounded-xl text-center">
                            <p className="text-gray-500 italic">No definitive infrastructure signals extracted from headers.</p>
                        </div>
                    )}
                </div>
            </div>

            {/* Headers Area */}
            <div className="space-y-6">
                <h2 className="text-2xl font-bold text-white mt-10 mb-4 px-2">Normalized HTTP Headers</h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    {/* Categorized */}
                    <div className="bg-card-bg border border-card-border rounded-2xl overflow-hidden flex flex-col">
                        <div className="bg-sidebar-bg/50 px-6 py-4 border-b border-sidebar-border">
                            <h3 className="font-semibold text-white tracking-wide">Categorized Buckets</h3>
                        </div>
                        <div className="p-6 space-y-6 flex-1 bg-black/20">
                            {Object.entries(data.categorized_headers).map(([category, headers]) => (
                                <div key={category} className="space-y-3">
                                    <h4 className="text-sm font-semibold text-accent-blue tracking-wide uppercase border-b border-white/10 pb-2">{category}</h4>
                                    <div className="space-y-2">
                                        {Object.entries(headers).map(([k, vals]) => (
                                            <div key={k} className="bg-black/60 rounded p-2 px-3 border border-white/5 flex flex-col md:flex-row md:gap-4 md:items-baseline">
                                                <span className="text-gray-400 font-mono text-xs min-w-[120px] shrink-0">{k}:</span>
                                                <div className="flex flex-col flex-wrap">
                                                    {vals.map((v, i) => (
                                                        <span key={i} className="text-gray-200 font-mono text-sm break-all">{v}</span>
                                                    ))}
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>

                    {/* Raw Dump */}
                    <div className="bg-card-bg border border-card-border rounded-2xl overflow-hidden flex flex-col">
                        <div className="bg-sidebar-bg/50 px-6 py-4 border-b border-sidebar-border">
                            <h3 className="font-semibold text-white tracking-wide">Raw Representation</h3>
                        </div>
                        <div className="p-6 flex-1 bg-black/40 font-mono text-xs sm:text-sm text-gray-300 overflow-x-auto">
                            {Object.entries(data.raw_headers).map(([k, vals]) => (
                                vals.map((v, i) => (
                                    <div key={`${k}-${i}`} className="py-1">
                                        <span className="text-blue-300 font-semibold">{k}:</span> <span className="opacity-90">{v}</span>
                                    </div>
                                ))
                            ))}
                        </div>
                    </div>
                </div>
            </div>

            <div className="w-full p-4 rounded-xl bg-black/50 border border-white/5 font-mono text-xs text-gray-500 flex justify-between">
              <span>Investigator Module (P4)</span>
              <span>{t.timestamp} {new Date().toLocaleTimeString()}</span>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
      <style jsx global>{`@keyframes shimmer { 100% { transform: translateX(100%); } }`}</style>
    </div>
  );
}
