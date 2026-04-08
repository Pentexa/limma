"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Activity, Loader2, Globe, ShieldAlert, CheckCircle, XCircle } from "lucide-react";
import { reportApi } from "@/utils/api";
import { useLanguage } from "@/context/LanguageContext";
import AuditorPanel from "@/components/AuditorPanel";
import { CertaintyBadge } from "@/components/CertaintyBadge";
import clsx from "clsx";

export default function AuditSecurityPage() {
  const [url, setUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [data, setData] = useState<any>(null);
  const [error, setError] = useState("");
  const { t } = useLanguage();

  const handleAudit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url) return;
    
    setIsLoading(true);
    setError("");
    setData(null);

    try {
      const targetUrl = url.startsWith("http") ? url : `https://${url}`;
      // Call masterReport instead of auditSecurity so we get the full normalized_audit pipeline output
      const result = await reportApi.masterReport(targetUrl);
      setData(result);
    } catch (err: any) {
      setError(err?.response?.data?.error || "An error occurred.");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      <header className="mb-10">
        <h1 className="text-4xl font-extrabold tracking-tight text-white flex items-center gap-3">
          {t.audTitle}
          <Activity className="h-8 w-8 text-accent-cyan" />
        </h1>
        <p className="mt-2 text-gray-400 text-lg">{t.audDesc}</p>
      </header>

      <form onSubmit={handleAudit} className="relative group max-w-2xl">
        <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
          <Globe className="h-5 w-5 text-gray-400 group-focus-within:text-accent-cyan transition-colors" />
        </div>
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder={t.enterUrl}
          className="block w-full pl-12 pr-32 py-4 bg-sidebar-bg/50 border border-sidebar-border rounded-2xl text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-accent-cyan/50 focus:border-accent-cyan transition-all shadow-xl shadow-black/20"
        />
        <button
          type="submit"
          disabled={isLoading || !url}
          className="absolute right-2 top-2 bottom-2 px-6 bg-gradient-to-r from-accent-cyan to-accent-blue hover:from-cyan-400 hover:to-blue-500 rounded-xl font-semibold text-black flex items-center gap-2 transition-all disabled:opacity-50 transform hover:scale-[1.02] active:scale-95 glow-cyan"
        >
          {isLoading ? <Loader2 className="h-4 w-4 animate-spin text-white" /> : <Activity className="h-4 w-4" />}
          {isLoading ? <span className="text-white">{t.auditing}</span> : t.audit}
        </button>
      </form>

      {error && (
        <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="p-4 bg-red-950/30 border border-red-500/50 rounded-xl text-red-200 flex items-center gap-3 max-w-2xl">
          <ShieldAlert className="h-5 w-5 text-red-400" />
          {error}
        </motion.div>
      )}

      {isLoading && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="mt-16 grid grid-cols-1 md:grid-cols-2 gap-6">
          {[...Array(2)].map((_, i) => (
            <div key={i} className="h-64 rounded-2xl bg-sidebar-bg/50 border border-sidebar-border relative overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent -translate-x-full animate-[shimmer_1.5s_infinite]"></div>
            </div>
          ))}
        </motion.div>
      )}

      <AnimatePresence>
        {data && !isLoading && (
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="mt-12 space-y-8">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              
              <div className="bg-card-bg border border-card-border p-6 rounded-2xl relative overflow-hidden group">
                <div className="absolute top-0 right-0 w-32 h-32 rounded-full blur-3xl opacity-10 bg-accent-cyan transform translate-x-1/2 -translate-y-1/2" />
                <div className="flex flex-col items-center justify-center py-6">
                  <div className="relative flex items-center justify-center w-32 h-32">
                    <svg className="w-full h-full transform -rotate-90">
                      <circle cx="64" cy="64" r="50" stroke="currentColor" strokeWidth="8" fill="transparent" className="text-white/5" />
                      <circle 
                        cx="64" cy="64" r="50" 
                        stroke="currentColor" 
                        strokeWidth="8" 
                        fill="transparent" 
                        strokeDasharray={314} 
                        strokeDashoffset={314 - (314 * ((data.security_audit?.security_score ?? data.security_score) || 0)) / 100}
                        className={clsx(
                          "transition-all duration-1000 ease-out",
                          ((data.security_audit?.security_score ?? data.security_score) || 0) < 50 ? "text-red-500" : ((data.security_audit?.security_score ?? data.security_score) || 0) < 80 ? "text-yellow-500" : "text-green-500"
                        )} 
                      />
                    </svg>
                    <span className="absolute text-3xl font-bold">{(data.security_audit?.security_score ?? data.security_score) || 0}</span>
                  </div>
                  <p className="mt-4 text-lg font-medium text-gray-300">{t.audScore}</p>
                </div>
              </div>

              <div className="bg-card-bg border border-card-border p-6 rounded-2xl relative overflow-hidden group lg:col-span-2">
                <div className="flex items-center gap-3 mb-6 relative z-10">
                  <div className="p-2 rounded-lg bg-black/40 border border-white/5 text-red-400">
                    <ShieldAlert className="h-5 w-5" />
                  </div>
                  <h3 className="font-semibold text-white tracking-wide">{t.audMissing}</h3>
                </div>
                {(data.security_audit?.missing_headers ?? data.missing_headers) && (data.security_audit?.missing_headers ?? data.missing_headers).length > 0 ? (
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    {(data.security_audit?.missing_headers ?? data.missing_headers).map((header: string, i: number) => (
                      <div key={i} className="flex items-start gap-3 p-3 bg-red-500/5 border border-red-500/20 rounded-lg">
                        <XCircle className="h-5 w-5 text-red-400 shrink-0 mt-0.5" />
                        <div>
                          <p className="text-sm font-medium text-gray-200">{header}</p>
                          <p className="text-xs text-gray-500 mt-1">{t.audHelps}</p>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="flex items-center gap-3 p-4 bg-green-500/5 border border-green-500/20 rounded-lg">
                    <CheckCircle className="h-5 w-5 text-green-400" />
                    <p className="text-sm font-medium text-green-100">{t.audAllGood}</p>
                  </div>
                )}
              </div>
            </div>
            
            {data.normalized_audit && (
              <>
                <CertaintyBadge certainty={data.normalized_audit.audit_certainty} />
                <AuditorPanel auditData={data.normalized_audit} />
              </>
            )}
            
            <div className="w-full p-4 rounded-xl bg-black/50 border border-white/5 font-mono text-xs text-gray-500 flex justify-between">
              <span>Auditor Module</span>
              <span>{t.timestamp} {new Date().toLocaleTimeString()}</span>
            </div>

          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
