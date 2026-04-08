"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Search, Loader2, Globe, Server, Code, FileCode2, ShieldAlert, Network, Terminal, Bug, Link, ShieldCheck, Flag } from "lucide-react";
import { reportApi } from "@/utils/api";
import { useLanguage } from "@/context/LanguageContext";
import clsx from "clsx";

export default function MasterReportPage() {
  const [url, setUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [data, setData] = useState<any>(null);
  const [error, setError] = useState("");
  const { t } = useLanguage();

  const handleScan = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url) return;
    
    setIsLoading(true);
    setError("");
    setData(null);

    try {
      const targetUrl = url.startsWith("http") ? url : `https://${url}`;
      const result = await reportApi.masterReport(targetUrl);
      setData(result);
    } catch (err: any) {
      setError(err?.response?.data?.error || "Arayüze dönerken sunucuda asenkron bir hata oluştu.");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      <header className="mb-10">
        <h1 className="text-4xl font-extrabold tracking-tight text-white flex items-center gap-3">
          {t.mrTitle}
          <span className="px-3 py-1 text-xs font-semibold bg-accent-blue/10 text-accent-cyan rounded-full border border-accent-blue/30 backdrop-blur-sm">PRO</span>
        </h1>
        <p className="mt-2 text-gray-400 text-lg">{t.mrDesc}</p>
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
        <button
          type="submit"
          disabled={isLoading || !url}
          className="absolute right-2 top-2 bottom-2 px-6 bg-gradient-to-r from-accent-cyan to-accent-blue hover:from-cyan-400 hover:to-blue-500 rounded-xl font-semibold text-black flex items-center gap-2 transition-all disabled:opacity-50 disabled:cursor-not-allowed transform hover:scale-[1.02] active:scale-95 glow-cyan"
        >
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

      {isLoading && (
        <motion.div 
          initial={{ opacity: 0 }} 
          animate={{ opacity: 1 }} 
          exit={{ opacity: 0 }}
          className="mt-16 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
        >
          {[...Array(6)].map((_, i) => (
            <div key={i} className="h-48 rounded-2xl bg-sidebar-bg/50 border border-sidebar-border relative overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent -translate-x-full animate-[shimmer_1.5s_infinite]"></div>
            </div>
          ))}
        </motion.div>
      )}

      <AnimatePresence>
        {data && !isLoading && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="mt-12 space-y-8"
          >
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              
              {/* Web Scan Result */}
              <ReportCard icon={Globe} title={t.mrWebTech} delay={0.1}>
                {data.analysis ? (
                  <div className="space-y-4">
                    <p className="text-sm text-gray-400"><span className="text-gray-300 font-medium">Architecture:</span> {data.analysis.suspected_architecture || 'Unknown'}</p>
                    <div className="flex flex-wrap gap-2">
                       {data.analysis.detected_technologies?.map((tech: any, i: number) => (
                         <span key={i} className="px-2 py-1 bg-white/5 border border-white/10 rounded-md text-xs text-gray-300">
                           {typeof tech === 'string' ? tech : tech.name}
                         </span>
                       ))}
                    </div>
                  </div>
                ) : (
                  <p className="text-gray-500 italic text-sm">{t.noData}</p>
                )}
              </ReportCard>

              {/* Server Investigation */}
              <ReportCard icon={Server} title={t.mrServer} delay={0.2} accent="blue">
                {data.server_info ? (
                  <div className="space-y-3">
                    <div className="p-3 bg-black/40 rounded-lg border border-white/5">
                      <p className="text-xs text-gray-500 font-mono uppercase tracking-wider mb-1">Location & CMS</p>
                      <p className="text-sm font-semibold truncate">{data.server_info.location_guess}</p>
                      {data.server_info.cms_detected && <p className="text-xs mt-1 text-accent-cyan truncate">{data.server_info.cms_detected}</p>}
                    </div>
                    {data.server_info.headers && (
                      <div className="pt-2 text-[10px] font-mono text-gray-500 max-h-24 overflow-y-auto custom-scrollbar">
                        {Object.entries(data.server_info.headers).map(([k, v]: [string, any], i) => (
                          <div key={i} className="truncate"><span className="text-accent-blue">{k}:</span> {v}</div>
                        ))}
                      </div>
                    )}
                  </div>
                ) : (
                  <p className="text-gray-500 italic text-sm">{t.noData}</p>
                )}
              </ReportCard>

              {/* APIs */}
              <ReportCard icon={Code} title={t.mrApi} delay={0.3} accent="cyan">
                 {data.api_discovery?.detected_endpoints?.length > 0 ? (
                   <ul className="space-y-2 max-h-36 overflow-y-auto custom-scrollbar pr-2">
                      {data.api_discovery.detected_endpoints.map((ep: any, i: number) => (
                        <li key={i} className="text-[11px] font-mono px-2 py-1.5 bg-accent-cyan/10 text-accent-cyan rounded border border-accent-cyan/20 truncate" title={ep.path || ep}>
                          {ep.method_prediction && <span className="font-bold mr-1 opacity-70">[{ep.method_prediction}]</span>}
                          {ep.path || ep}
                        </li>
                      ))}
                   </ul>
                 ) : (
                   <p className="text-gray-500 italic text-sm">{t.noData}</p>
                 )}
              </ReportCard>

              {/* External Services */}
              <ReportCard className="lg:col-span-2" icon={Network} title={t.mrExt} delay={0.4}>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <h4 className="text-xs text-gray-500 uppercase font-semibold mb-3">Integrations</h4>
                    {data.service_communication?.external_integrations?.length > 0 ? (
                      <ul className="space-y-2 max-h-32 overflow-y-auto custom-scrollbar pr-2">
                        {data.service_communication.external_integrations.map((ext: string, i: number) => (
                          <li key={i} className="text-xs font-medium text-gray-300 truncate flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-purple-500" />
                            {ext}
                          </li>
                        ))}
                      </ul>
                    ) : <p className="text-xs text-gray-600">None detected</p>}
                  </div>
                  <div>
                    <h4 className="text-xs text-gray-500 uppercase font-semibold mb-3">Contacted Domains</h4>
                    {data.service_communication?.domains_contacted?.length > 0 ? (
                      <div className="space-y-2 max-h-32 overflow-y-auto custom-scrollbar pr-2">
                         {data.service_communication.domains_contacted.map((dom: string, i: number) => (
                           <div key={i} className="text-xs font-mono text-gray-400 truncate flex items-center gap-2">
                             <div className="w-1 h-1 bg-white/20" />
                             {dom}
                           </div>
                         ))}
                      </div>
                    ) : <p className="text-xs text-gray-600">None</p>}
                  </div>
                </div>
              </ReportCard>

              {/* Security Audit */}
              <ReportCard icon={ShieldAlert} title={t.mrSec} delay={0.5} accent={(data.security_audit?.security_score ?? 100) < 50 ? 'red' : 'green'}>
                  {data.security_audit ? (
                    <div className="flex flex-col items-center justify-center pt-2">
                      <div className="relative flex items-center justify-center w-24 h-24">
                        <svg className="w-full h-full transform -rotate-90">
                          <circle cx="48" cy="48" r="36" stroke="currentColor" strokeWidth="6" fill="transparent" className="text-white/5" />
                          <circle 
                            cx="48" cy="48" r="36" 
                            stroke="currentColor" 
                            strokeWidth="6" 
                            fill="transparent" 
                            strokeDasharray={226} 
                            strokeDashoffset={226 - (226 * data.security_audit.security_score) / 100}
                            className={clsx(
                              "transition-all duration-1000 ease-out",
                              data.security_audit.security_score < 50 ? "text-red-500" : data.security_audit.security_score < 80 ? "text-yellow-500" : "text-green-500"
                            )} 
                          />
                        </svg>
                        <span className="absolute text-xl font-bold">{data.security_audit.security_score}</span>
                      </div>
                      <p className="mt-4 text-sm font-medium text-gray-300">Score</p>
                      
                      {data.security_audit.recommendations?.length > 0 && (
                        <div className="mt-4 w-full h-12 overflow-y-auto custom-scrollbar space-y-1">
                          {data.security_audit.recommendations.slice(0, 3).map((r: string, i: number) => (
                            <div key={i} className="text-[10px] text-red-300 bg-red-900/20 px-2 py-1 rounded truncate" title={r}>
                              {r}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  ) : <p className="text-gray-500 italic text-sm">{t.noData}</p>}
              </ReportCard>
            
            </div>
            
            <div className="w-full p-4 rounded-xl bg-black/50 border border-white/5 font-mono text-xs text-gray-500 flex justify-between">
              <span>Analysis ID: {Math.random().toString(36).substring(7)}</span>
              <span>{t.timestamp} {new Date().toLocaleTimeString()}</span>
            </div>

            {data.normalized_audit && (
              <AuditorPanel auditData={data.normalized_audit} scanStrategy={data.scan_strategy} />
            )}

          </motion.div>
        )}
      </AnimatePresence>

      <style jsx global>{`
        @keyframes shimmer {
          100% {
            transform: translateX(100%);
          }
        }
      `}</style>
    </div>
  );
}

function ReportCard({ children, icon: Icon, title, delay = 0, accent = "cyan", className = "" }: any) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ delay, duration: 0.4 }}
      className={clsx(
        "bg-card-bg border border-card-border p-6 rounded-2xl relative overflow-hidden group hover:border-gray-700 transition-colors",
        className
      )}
    >
      <div className={clsx(
        "absolute top-0 right-0 w-32 h-32 rounded-full blur-3xl opacity-0 group-hover:opacity-10 transition-opacity transform translate-x-1/2 -translate-y-1/2",
        accent === "cyan" && "bg-accent-cyan",
        accent === "blue" && "bg-accent-blue",
        accent === "red" && "bg-red-500",
        accent === "green" && "bg-green-500"
      )} />
      
      <div className="flex items-center gap-3 mb-6 relative z-10">
        <div className={clsx(
          "p-2 rounded-lg bg-black/40 border border-white/5",
          accent === "cyan" && "text-accent-cyan",
          accent === "blue" && "text-accent-blue",
          accent === "red" && "text-red-400",
          accent === "green" && "text-green-400"
        )}>
          <Icon className="h-5 w-5" />
        </div>
        <h3 className="font-semibold text-white tracking-wide">{title}</h3>
      </div>
      
      <div className="relative z-10 min-h-[120px]">
        {children}
      </div>
    </motion.div>
  );
}
import AuditorPanel from "@/components/AuditorPanel";
