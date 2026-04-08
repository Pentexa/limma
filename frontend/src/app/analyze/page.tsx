"use client";

import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Search, Loader2, Globe, ShieldAlert, Activity, ArrowRight, CornerDownRight, Server, Clock, Database, CheckCircle2, Cpu, FileBadge, AlertTriangle } from "lucide-react";
import { reportApi } from "@/utils/api";
import { useLanguage } from "@/context/LanguageContext";
import { CertaintyBadge } from "@/components/CertaintyBadge";

interface RedirectChainEntry {
  url: string;
  status_code: number;
}

interface TechEvidence {
  source: string;
  snippet: string;
}

interface DetectedTechnology {
  name: string;
  category: string;
  confidence_score: number;
  evidences: TechEvidence[];
}

interface SecurityHeaderResult {
  name: string;
  status: 'present' | 'missing' | 'weak' | 'misconfigured';
  value?: string;
  explanation: string;
}

interface RiskInsight {
  title: string;
  severity: 'low' | 'medium' | 'high';
  explanation: string;
  evidence: string;
}

interface ScannedPage {
  url: string;
  status_code: number;
  latency_ms: number;
  headers: Record<string, string>;
  content_type?: string;
  detected_technologies: DetectedTechnology[];
  security_headers: SecurityHeaderResult[];
  risk_insights: RiskInsight[];
}

interface ScanEvent {
  timestamp: string;
  level: string;
  message: string;
}

interface ScanSummary {
  total_pages: number;
  average_latency_ms: number;
  common_technologies: string[];
}

interface CorrelatedRisk {
  title: string;
  severity: 'low' | 'medium' | 'high';
  explanation: string;
  evidences: string[];
}

interface CorrelationReport {
  overall_risk_score: number;
  correlated_risks: CorrelatedRisk[];
}

interface WebScanResult {
  original_target_url: string;
  final_url: string;
  scan_start_time: string;
  scan_end_time: string;
  total_duration_ms: number;
  final_status_code: number;
  latency_ms: number;
  redirect_count: number;
  redirect_chain: RedirectChainEntry[];
  headers: Record<string, string>;
  content_type?: string;
  content_length?: number;
  server?: string;
  cache_control?: string;
  detected_technologies: DetectedTechnology[];
  security_headers: SecurityHeaderResult[];
  risk_insights: RiskInsight[];
  security_score: number;
  pages: ScannedPage[];
  timeline: ScanEvent[];
  summary?: ScanSummary;
  correlation?: CorrelationReport;
  scan_certainty?: { level: string; reason: string } | null;
}

export default function AnalyzePage() {
  const [url, setUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [data, setData] = useState<WebScanResult | null>(null);
  const [error, setError] = useState("");
  const [logs, setLogs] = useState<string[]>([]);
  const { t } = useLanguage();

  const handleScan = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url) return;
    setIsLoading(true);
    setError("");
    setData(null);
    setLogs([]);

    const targetUrl = url.startsWith("http") ? url : `https://${url}`;
    
    const eventSource = new EventSource(reportApi.analyzeWebsiteStreamUrl(targetUrl));
    let streamData: WebScanResult | null = null;

    const handleStreamEvent = (parsed: any) => {
       if (!streamData) {
          streamData = {
              original_target_url: targetUrl, final_url: targetUrl,
              scan_start_time: new Date().toISOString(), scan_end_time: "",
              total_duration_ms: 0, final_status_code: 0, latency_ms: 0,
              redirect_count: 0, redirect_chain: [], headers: {},
              detected_technologies: [], security_headers: [], risk_insights: [],
              security_score: 100, pages: [], timeline: []
          };
       }
       
       streamData.timeline.unshift({ timestamp: parsed.timestamp, level: parsed.level, message: parsed.message });

       if (parsed.event_type === "PAGE_CRAWLED" && parsed.payload) {
           streamData.pages.push(parsed.payload);
       } else if (parsed.event_type === "TECH_DETECTED" && Array.isArray(parsed.payload)) {
           parsed.payload.forEach((tech: any) => {
               if (!streamData!.detected_technologies.find(t => t.name === tech.name)) {
                   streamData!.detected_technologies.push(tech);
               }
           });
       } else if (parsed.event_type === "RISK_GENERATED" && parsed.payload) {
           if (!streamData.correlation) streamData.correlation = { overall_risk_score: 100, correlated_risks: [] };
           streamData.correlation.correlated_risks.push(parsed.payload);
       } else if (parsed.event_type === "FINAL_RESULT" && parsed.payload) {
           streamData = parsed.payload; 
           eventSource.close();
           setIsLoading(false);
       }

       setData(streamData ? { ...streamData } as WebScanResult : null);
    };

    [
        "SCAN_STARTED", "CRAWLING_PAGE", "PAGE_CRAWLED", "TECH_DETECTED", 
        "TECH_CONFIDENCE_BOOST", "TECH_ISOLATED", "HEADER_DISCREPANCY", 
        "CORRELATION_STARTED", "RISK_GENERATED", "CORRELATION_COMPLETE", 
        "FINAL_RESULT", "CRAWL_LIMIT_REACHED", "CRAWL_COMPLETE", "FETCH_ERROR"
    ].forEach(eventType => {
        eventSource.addEventListener(eventType, (e: MessageEvent) => {
            handleStreamEvent(JSON.parse(e.data));
        });
    });

    eventSource.onerror = () => {
        if (eventSource.readyState === EventSource.CLOSED) {
            setIsLoading(false);
        } else {
            setError("Connection dropped. Stream ended abruptly.");
            eventSource.close();
            setIsLoading(false);
        }
    };
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-700 max-w-5xl">
      <header className="mb-10">
        <h1 className="text-4xl font-extrabold tracking-tight text-white flex items-center gap-3">
          Web Scanner
          <Search className="h-8 w-8 text-accent-cyan" />
        </h1>
        <p className="mt-2 text-gray-400 text-lg">Phase 3 Engine: Advanced network request tracing, tech fingerprinting, and surface-level risk auditing.</p>
      </header>

      <form onSubmit={handleScan} className="relative group max-w-2xl">
        <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
          <Globe className="h-5 w-5 text-gray-400 group-focus-within:text-accent-cyan transition-colors" />
        </div>
        <input
          id="urlInput"
          name="url"
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder={t.enterUrl}
          className="block w-full pl-12 pr-32 py-4 bg-sidebar-bg/50 border border-sidebar-border rounded-2xl text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-accent-cyan/50 focus:border-accent-cyan transition-all shadow-xl shadow-black/20"
        />
        <button type="submit" disabled={isLoading || !url} className="absolute right-2 top-2 bottom-2 px-6 bg-gradient-to-r from-accent-cyan to-accent-blue hover:from-cyan-400 hover:to-blue-500 rounded-xl font-semibold text-black flex items-center gap-2 transition-all disabled:opacity-50 transform hover:scale-[1.02] active:scale-95 glow-cyan">
          {isLoading ? <Loader2 className="h-4 w-4 animate-spin text-white" /> : <Search className="h-4 w-4" />}
          {isLoading ? <span className="text-white">{t.scanning}</span> : t.scan}
        </button>
      </form>

      {error && (
        <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="p-4 bg-red-950/30 border border-red-500/50 rounded-xl text-red-200 flex items-center gap-3 max-w-2xl">
          <ShieldAlert className="h-5 w-5 text-red-400" />
          {error}
        </motion.div>
      )}

      {(isLoading || data) && (
        <div className="bg-black/60 border border-white/5 rounded-xl p-4 font-mono text-sm max-w-2xl">
          <div className="flex items-center gap-2 mb-3 text-emerald-400 border-b border-white/5 pb-2">
            <Activity className="h-4 w-4" /> Activity Pipeline
          </div>
          <div className="space-y-1 h-32 overflow-y-auto pl-2">
            {data && data.timeline ? (
              data.timeline.map((event, i) => (
                <motion.div key={i} initial={{ opacity: 0, x: -10 }} animate={{ opacity: 1, x: 0 }} className="flex gap-2 text-gray-400">
                  <span className={`w-16 shrink-0 ${event.level==='ERROR'?'text-red-400':(event.level==='WARN'?'text-yellow-400':'text-emerald-500/50')}`}>[{event.level}]</span> 
                  <span className={event.level==='ERROR'?'text-red-300':(event.level==='WARN'?'text-yellow-300':'')}>{event.message}</span>
                </motion.div>
              ))
            ) : (
              logs.map((log, i) => (
                <motion.div key={i} initial={{ opacity: 0, x: -10 }} animate={{ opacity: 1, x: 0 }} className="flex gap-2 items-center text-gray-400">
                  <span className="text-emerald-500/50">{'>'}</span> 
                  <span className={i === logs.length - 1 && isLoading ? "animate-pulse string-emerald-400" : ""}>{log}</span>
                </motion.div>
              ))
            )}
          </div>
        </div>
      )}

      <AnimatePresence>
        {data && !isLoading && (
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="mt-8 space-y-6">
            
            {/* Overview Metrics */}
            <div className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-6 gap-4`}>
               <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl relative overflow-hidden group hover:border-accent-cyan/50 transition-colors">
                  <div className="flex justify-between items-center mb-3">
                     <span className="text-xs uppercase tracking-widest text-gray-500 font-bold">Target Risk</span>
                     <ShieldAlert className={`w-4 h-4 ${(data.correlation?.overall_risk_score ?? data.security_score) >= 80 ? 'text-emerald-400' : ((data.correlation?.overall_risk_score ?? data.security_score) >= 50 ? 'text-yellow-400' : 'text-red-400')}`} />
                  </div>
                  <div className={`text-4xl font-extrabold ${(data.correlation?.overall_risk_score ?? data.security_score) >= 80 ? 'text-emerald-400' : ((data.correlation?.overall_risk_score ?? data.security_score) >= 50 ? 'text-yellow-400' : 'text-red-400')}`}>
                    {data.correlation?.overall_risk_score ?? data.security_score}
                  </div>
                  <div className="text-[10px] text-gray-500 mt-1">Holistic Risk Score</div>
               </div>

               <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl relative overflow-hidden group hover:border-accent-cyan/50 transition-colors">
                  <div className="flex justify-between items-center mb-3">
                     <span className="text-xs uppercase tracking-widest text-gray-500 font-bold">Status Code</span>
                     <CheckCircle2 className={`w-4 h-4 ${data.final_status_code >= 200 && data.final_status_code < 400 ? 'text-emerald-400' : 'text-red-400'}`} />
                  </div>
                  <div className={`text-4xl font-extrabold ${data.final_status_code >= 200 && data.final_status_code < 400 ? 'text-emerald-400' : 'text-red-400'}`}>
                    {data.final_status_code}
                  </div>
               </div>

               <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl relative overflow-hidden group hover:border-accent-cyan/50 transition-colors">
                  <div className="flex justify-between items-center mb-3">
                     <span className="text-xs uppercase tracking-widest text-gray-500 font-bold">Avg Latency</span>
                     <Clock className="w-4 h-4 text-purple-400" />
                  </div>
                  <div className="text-4xl font-extrabold text-purple-400">
                    {data.summary?.average_latency_ms || data.latency_ms}<span className="text-base text-gray-500 font-normal">ms</span>
                  </div>
                  <div className="text-[10px] text-gray-500 mt-1">Total run: {data.total_duration_ms}ms</div>
               </div>

               <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl relative overflow-hidden group hover:border-accent-cyan/50 transition-colors">
                 <div className="flex justify-between items-center mb-3">
                     <span className="text-xs uppercase tracking-widest text-gray-500 font-bold">Server Identity</span>
                     <Server className="w-4 h-4 text-blue-400" />
                  </div>
                  <div className="text-xl font-bold text-blue-400 truncate" title={data.server || "HIDDEN"}>
                    {data.server || "HIDDEN"}
                  </div>
                  <div className="text-[10px] text-gray-500 mt-1 truncate">{data.content_type || "No content-type"}</div>
               </div>

               <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl relative overflow-hidden group hover:border-accent-cyan/50 transition-colors">
                 <div className="flex justify-between items-center mb-3">
                     <span className="text-xs uppercase tracking-widest text-accent-cyan font-bold">Tech Stack</span>
                     <Cpu className="w-4 h-4 text-accent-cyan" />
                  </div>
                  <div className="text-4xl font-extrabold text-accent-cyan">
                    {data.detected_technologies.length}
                  </div>
                  <div className="text-[10px] text-gray-400 mt-1 truncate">
                    Top Certainty: {data.detected_technologies.length > 0 ? `${data.detected_technologies[0].name} (${(data.detected_technologies[0].confidence_score*100).toFixed(0)}%)` : "None"}
                  </div>
               </div>
               
               <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl relative overflow-hidden group hover:border-accent-cyan/50 transition-colors">
                  <div className="flex justify-between items-center mb-3">
                     <span className="text-xs uppercase tracking-widest text-gray-500 font-bold">Pages Scanned</span>
                     <Globe className="w-4 h-4 text-pink-400" />
                  </div>
                  <div className="text-4xl font-extrabold text-pink-400">
                    {data.summary?.total_pages || 1}
                  </div>
               </div>
            </div>

            {/* Scan Certainty */}
            <CertaintyBadge certainty={data.scan_certainty as any} />

            {/* Phase 5: Correlated Risks */}
            {data.correlation && data.correlation.correlated_risks.length > 0 && (
              <div className="bg-red-950/20 border border-red-500/20 p-5 rounded-xl">
                 <div className="flex justify-between items-center mb-6">
                   <h3 className="text-sm uppercase tracking-widest text-gray-300 font-bold flex items-center gap-2">
                     <AlertTriangle className="w-4 h-4 text-red-400" /> Correlated Engine Insights
                   </h3>
                   <span className="text-[10px] bg-red-500/20 text-red-300 px-2 py-0.5 rounded border border-red-500/30 flex items-center gap-1">
                     Phase 5 Analytics
                   </span>
                 </div>
                 
                 <div className="space-y-4">
                    {data.correlation.correlated_risks.map((risk, idx) => (
                      <div key={idx} className="bg-black/50 border border-white/5 rounded-xl p-5 relative overflow-hidden">
                         <div className={`absolute top-0 left-0 w-1 h-full ${risk.severity === 'high' ? 'bg-red-500' : (risk.severity === 'medium' ? 'bg-orange-500' : 'bg-yellow-500')}`}></div>
                         <div className="pl-3">
                           <div className="flex justify-between items-start mb-2">
                              <span className="font-bold text-gray-200 text-sm">{risk.title}</span>
                              <span className={`text-[9px] font-bold px-1.5 py-0.5 rounded uppercase tracking-wider ${risk.severity === 'high' ? 'bg-red-500/20 text-red-400 border border-red-500/30' : (risk.severity === 'medium' ? 'bg-orange-500/20 text-orange-400 border border-orange-500/30' : 'bg-yellow-500/10 text-yellow-500/70 border border-yellow-500/20')}`}>
                                {risk.severity} Risk
                              </span>
                           </div>
                           <p className="text-xs text-gray-400 mb-4">{risk.explanation}</p>
                           <div className="mt-auto pt-3 border-t border-white/5">
                              <span className="text-[10px] uppercase text-gray-500 font-bold block mb-2">Cross-Reference Evidence</span>
                              <div className="space-y-1">
                                {risk.evidences.map((ev, i) => (
                                  <code key={i} className="text-[10px] text-gray-300 bg-gray-900/50 px-2 py-1 rounded block truncate">{ev}</code>
                                ))}
                              </div>
                           </div>
                         </div>
                      </div>
                    ))}
                 </div>
              </div>
            )}

            {/* Redirect Chain */}
            <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl">
               <h3 className="text-sm uppercase tracking-widest text-gray-400 font-bold mb-4 flex items-center gap-2">
                 <ArrowRight className="w-4 h-4" /> Redirect Chain ({data.redirect_count})
               </h3>
               {data.redirect_chain.length > 0 ? (
                 <div className="space-y-0">
                    {data.redirect_chain.map((rc, idx) => (
                      <div key={idx} className="flex flex-col relative py-2">
                         {idx !== data.redirect_chain.length - 1 && (
                            <div className="absolute left-[11px] top-8 bottom-[-8px] w-px bg-white/10 z-0 border-dashed border-l border-white/20"></div>
                         )}
                         <div className="flex items-start gap-4 z-10">
                           <div className={`mt-0.5 w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold shrink-0 ${rc.status_code >= 300 && rc.status_code < 400 ? 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30' : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'}`}>
                             {rc.status_code}
                           </div>
                           <div className="flex-1 min-w-0">
                              <p className="text-sm font-mono text-gray-300 break-all">{rc.url}</p>
                              {idx === 0 && <p className="text-[10px] text-gray-500 uppercase">Original Target</p>}
                              {idx === data.redirect_chain.length - 1 && <p className="text-[10px] text-accent-cyan uppercase mt-1">Final Destination</p>}
                           </div>
                         </div>
                      </div>
                    ))}
                 </div>
               ) : (
                 <p className="text-sm text-gray-500 italic">No redirects. Reached destination directly.</p>
               )}
            </div>

            {/* Phase 2: Tech Fingerprinting */}
            <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl">
               <div className="flex justify-between items-center mb-6">
                 <h3 className="text-sm uppercase tracking-widest text-gray-400 font-bold flex items-center gap-2">
                   <Cpu className="w-4 h-4" /> Detected Technologies ({data.detected_technologies.length})
                 </h3>
                 <span className="text-[10px] bg-accent-cyan/10 text-accent-cyan px-2 py-0.5 rounded border border-accent-cyan/20 flex items-center gap-1">
                   <FileBadge className="w-3 h-3" /> Phase 2 Engine
                 </span>
               </div>
               
               {data.detected_technologies.length > 0 ? (
                 <div className="space-y-6">
                   {Array.from(new Set(data.detected_technologies.map(t => t.category))).map(category => (
                     <div key={category}>
                       <h4 className="text-xs uppercase tracking-widest text-gray-500 font-bold border-b border-white/5 pb-2 mb-3">{category}</h4>
                       <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
                         {data.detected_technologies.filter(t => t.category === category).map((tech, idx) => (
                           <div key={idx} className="bg-black/40 border border-white/5 rounded-lg p-3 group relative hover:border-accent-cyan/30 transition-colors">
                             <div className="flex justify-between items-start mb-2">
                               <span className="font-bold text-gray-200 text-sm">{tech.name}</span>
                               <span className={`text-[9px] font-bold px-1.5 py-0.5 rounded ${tech.confidence_score > 0.8 ? 'bg-emerald-500/20 text-emerald-400' : (tech.confidence_score > 0.5 ? 'bg-yellow-500/20 text-yellow-400' : 'bg-red-500/20 text-red-400')}`}>
                                 {(tech.confidence_score * 100).toFixed(0)}% Certainty
                               </span>
                             </div>
                             
                             <div className="space-y-1">
                               {tech.evidences.map((ev, ei) => (
                                 <div key={ei} className="text-[10px] text-gray-400 flex items-center gap-1.5 truncate" title={ev.snippet}>
                                   <span className="shrink-0 w-1.5 h-1.5 rounded-full bg-accent-cyan/50"></span>
                                   <span className="uppercase text-[8px] tracking-wider text-gray-500">[{ev.source}]</span> 
                                   <span className="truncate">{ev.snippet}</span>
                                 </div>
                               ))}
                             </div>
                           </div>
                         ))}
                       </div>
                     </div>
                   ))}
                 </div>
               ) : (
                 <p className="text-sm text-gray-500 italic">No recognizable technologies found via fingerprinting.</p>
               )}
            </div>

            {/* Phase 3: Risk Insights */}
            <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl">
               <div className="flex justify-between items-center mb-6">
                 <h3 className="text-sm uppercase tracking-widest text-gray-400 font-bold flex items-center gap-2">
                   <AlertTriangle className="w-4 h-4 text-orange-400" /> Security Risk Insights ({data.risk_insights.length})
                 </h3>
                 <span className="text-[10px] bg-orange-500/10 text-orange-400 px-2 py-0.5 rounded border border-orange-500/20 flex items-center gap-1">
                   Phase 3 Engine
                 </span>
               </div>
               
               {data.risk_insights.length > 0 ? (
                 <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {data.risk_insights.map((risk, idx) => (
                      <div key={idx} className="bg-black/40 border border-white/5 rounded-lg p-4 relative overflow-hidden group">
                         <div className={`absolute top-0 left-0 w-1 h-full ${risk.severity === 'high' ? 'bg-red-500' : (risk.severity === 'medium' ? 'bg-orange-500' : 'bg-yellow-500/50')}`}></div>
                         <div className="pl-3 flex flex-col h-full">
                           <div className="flex justify-between items-start mb-2">
                              <span className="font-bold text-gray-200 text-sm">{risk.title}</span>
                              <span className={`text-[9px] font-bold px-1.5 py-0.5 rounded uppercase tracking-wider ${risk.severity === 'high' ? 'bg-red-500/20 text-red-400 border border-red-500/30' : (risk.severity === 'medium' ? 'bg-orange-500/20 text-orange-400 border border-orange-500/30' : 'bg-yellow-500/10 text-yellow-500/70 border border-yellow-500/20')}`}>
                                {risk.severity} Risk
                              </span>
                           </div>
                           <p className="text-xs text-gray-400 mb-4">{risk.explanation}</p>
                           <div className="mt-auto pt-3 border-t border-white/5">
                              <span className="text-[10px] uppercase text-gray-500 font-bold block mb-1">Evidence</span>
                              <code className="text-[10px] text-gray-300 bg-gray-900/50 px-2 py-1 rounded block truncate" title={risk.evidence}>{risk.evidence}</code>
                           </div>
                         </div>
                      </div>
                    ))}
                 </div>
               ) : (
                 <p className="text-sm text-emerald-400/80 italic flex items-center gap-2"><CheckCircle2 className="w-4 h-4" /> No immediate surface-level risks detected.</p>
               )}
            </div>

            {/* Phase 3: Security Headers Auditing */}
            <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl">
               <div className="flex justify-between items-center mb-6">
                 <h3 className="text-sm uppercase tracking-widest text-gray-400 font-bold flex items-center gap-2">
                   <ShieldAlert className="w-4 h-4 text-emerald-400" /> Security Headers
                 </h3>
                 <span className="text-[10px] bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/20 flex items-center gap-1">
                   Phase 3 Engine
                 </span>
               </div>
               
               <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                  {data.security_headers.map((hdr, idx) => (
                    <div key={idx} className="bg-black/40 border border-white/5 rounded-lg p-3 group">
                       <div className="flex justify-between items-start mb-1.5">
                         <span className="text-xs font-mono font-bold text-gray-300">{hdr.name}</span>
                         <span className={`text-[9px] font-bold px-1.5 py-0.5 rounded uppercase ${hdr.status === 'present' ? 'bg-emerald-500/20 text-emerald-400' : (hdr.status === 'weak' ? 'bg-yellow-500/20 text-yellow-400' : (hdr.status === 'misconfigured' ? 'bg-orange-500/20 text-orange-400' : 'bg-red-500/20 text-red-400'))}`}>
                           {hdr.status}
                         </span>
                       </div>
                       <div className="text-[10px] text-gray-500 mb-2">{hdr.explanation}</div>
                       {hdr.value && (
                          <div className="text-[10px] text-gray-400 font-mono bg-white/5 px-2 py-1.5 rounded break-all max-h-16 overflow-y-auto custom-scrollbar">
                            {hdr.value}
                          </div>
                       )}
                    </div>
                  ))}
               </div>
            </div>

            {/* Raw Headers */}
            <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl">
               <h3 className="text-sm uppercase tracking-widest text-gray-400 font-bold mb-4 flex items-center gap-2">
                 <Database className="w-4 h-4" /> Response Headers
               </h3>
               <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                  {Object.entries(data.headers).map(([k, v], i) => (
                     <div key={i} className="bg-black/40 border border-white/5 p-3 rounded flex flex-col gap-1 overflow-hidden">
                       <span className="text-[10px] text-gray-500 uppercase tracking-widest font-bold">{k}</span>
                       <span className="text-xs text-emerald-200/80 font-mono break-all">{v}</span>
                     </div>
                  ))}
               </div>
            </div>
            
            {/* Phase 4: Crawled Pages Overview */}
            {data.pages && data.pages.length > 0 && (
              <div className="bg-card-bg/50 border border-card-border p-5 rounded-xl">
                 <div className="flex justify-between items-center mb-6">
                   <h3 className="text-sm uppercase tracking-widest text-gray-400 font-bold flex items-center gap-2">
                     <Globe className="w-4 h-4 text-pink-400" /> Crawled Pages Overview ({data.pages.length})
                   </h3>
                   <span className="text-[10px] bg-pink-500/10 text-pink-400 px-2 py-0.5 rounded border border-pink-500/20 flex items-center gap-1">
                     Phase 4 Crawler
                   </span>
                 </div>
                 
                 <div className="space-y-3">
                    {data.pages.map((page, idx) => (
                      <div key={idx} className="bg-black/40 border border-white/5 rounded-lg p-4 group hover:border-pink-500/30 transition-colors">
                         <div className="flex items-center justify-between mb-2">
                           <span className="text-sm font-mono text-emerald-300 break-all">{page.url}</span>
                           <span className={`text-[10px] font-bold px-2 py-0.5 rounded ${page.status_code >= 200 && page.status_code < 400 ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400'}`}>
                             {page.status_code}
                           </span>
                         </div>
                         <div className="flex flex-wrap gap-4 text-xs text-gray-500">
                           <span className="flex items-center gap-1"><Clock className="w-3 h-3" /> {page.latency_ms}ms</span>
                           <span className="flex items-center gap-1"><Cpu className="w-3 h-3" /> {page.detected_technologies.length} Techs</span>
                           <span className="flex items-center gap-1"><AlertTriangle className="w-3 h-3 text-yellow-400" /> {page.risk_insights.length} Risks</span>
                         </div>
                      </div>
                    ))}
                 </div>
              </div>
            )}

            <div className="w-full p-4 rounded-xl bg-black/50 border border-white/5 font-mono text-xs text-gray-500 flex justify-between">
              <span>Foundation Scanner Module (Phase 4 Upgrade)</span>
              <span>Ran at: {new Date(data.scan_end_time).toLocaleString()}</span>
            </div>

          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
