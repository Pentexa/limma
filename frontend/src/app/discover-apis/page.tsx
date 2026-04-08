"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Network, Loader2, Globe, ShieldAlert, Code, Play, Send, Activity, CheckCircle2, XCircle } from "lucide-react";
import { extractApiError, reportApi } from "@/utils/api";
import { useLanguage } from "@/context/LanguageContext";
import { CertaintyBadge } from "@/components/CertaintyBadge";

export default function DiscoverApisPage() {
  const [url, setUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [data, setData] = useState<any>(null);
  const [error, setError] = useState("");
  const { t } = useLanguage();

  const handleDiscover = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url) return;
    setIsLoading(true);
    setError("");
    setData(null);

    try {
      const targetUrl = url.startsWith("http") ? url : `https://${url}`;
      const result = await reportApi.discoverApis(targetUrl);
      setData(result);
    } catch (err: any) {
      setError(extractApiError(err));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      <header className="mb-10">
        <h1 className="text-4xl font-extrabold tracking-tight text-white flex items-center gap-3">
          {t.apiTitle}
          <Network className="h-8 w-8 text-emerald-400" />
        </h1>
        <p className="mt-2 text-gray-400 text-lg">{t.apiDesc}</p>
      </header>

      <form onSubmit={handleDiscover} className="relative group max-w-2xl">
        <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
          <Globe className="h-5 w-5 text-gray-400 group-focus-within:text-emerald-400 transition-colors" />
        </div>
        <input
          id="urlInput"
          name="url"
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder={t.enterUrl}
          className="block w-full pl-12 pr-32 py-4 bg-sidebar-bg/50 border border-sidebar-border rounded-2xl text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-emerald-400/50 focus:border-emerald-400 transition-all shadow-xl shadow-black/20"
        />
        <button type="submit" disabled={isLoading || !url} className="absolute right-2 top-2 bottom-2 px-6 bg-gradient-to-r from-emerald-500 to-teal-500 hover:from-emerald-400 hover:to-teal-400 rounded-xl font-semibold text-black flex items-center gap-2 transition-all disabled:opacity-50 transform hover:scale-[1.02] active:scale-95 glow-emerald">
          {isLoading ? <Loader2 className="h-4 w-4 animate-spin text-black" /> : <Network className="h-4 w-4" />}
          {isLoading ? t.discovering : t.discover}
        </button>
      </form>

      {error && (
        <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="p-4 bg-red-950/30 border border-red-500/50 rounded-xl text-red-200 flex items-center gap-3 max-w-2xl">
          <ShieldAlert className="h-5 w-5 text-red-400" />
          {error}
        </motion.div>
      )}

      {isLoading && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="mt-16 h-64 rounded-2xl bg-sidebar-bg/50 border border-sidebar-border relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent -translate-x-full animate-[shimmer_1.5s_infinite]"></div>
        </motion.div>
      )}

      <AnimatePresence>
        {data && !isLoading && (
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="mt-12 space-y-8">
            <div className="bg-card-bg border border-card-border p-6 rounded-2xl relative overflow-hidden group max-w-4xl">
              <div className="absolute top-0 right-0 w-32 h-32 rounded-full blur-3xl opacity-10 bg-emerald-400 transform translate-x-1/2 -translate-y-1/2" />
              <div className="flex items-center justify-between mb-6 relative z-10">
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-lg bg-black/40 border border-white/5 text-emerald-400"><Code className="h-5 w-5" /></div>
                  <h3 className="font-semibold text-white tracking-wide">{t.apiDiscovered}</h3>
                </div>
                <div className="px-3 py-1 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded-full text-xs font-bold font-mono">
                  {data.detected_endpoints?.length || 0} {t.apiCount}
                </div>
              </div>
              
              <div className="relative z-10">
                {data.metrics && (
                  <div className="bg-emerald-950/20 border border-emerald-500/30 rounded-lg p-4 mb-6 relative overflow-hidden">
                    <div className="absolute top-0 left-0 w-1 h-full bg-emerald-500"></div>
                    <h2 className="text-sm font-bold text-emerald-400 mb-4 flex items-center gap-2">
                      <Activity className="w-4 h-4" /> Runtime Verification & Metrics (Ground Truth)
                    </h2>
                    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
                        <div className="bg-black/40 border border-white/5 p-3 rounded text-center">
                          <div className="text-[10px] text-gray-400 uppercase tracking-widest mb-1">Endpoints (Valid / Total)</div>
                          <div className="text-xl font-bold font-mono text-white">
                              <span className="text-emerald-400">{data.metrics.valid_endpoints}</span> / <span className="text-gray-400">{data.metrics.total_endpoints}</span>
                          </div>
                        </div>
                        <div className="bg-black/40 border border-white/5 p-3 rounded text-center">
                          <div className="text-[10px] text-gray-400 uppercase tracking-widest mb-1">Precision</div>
                          <div className="text-xl font-bold font-mono text-emerald-400">
                              {(data.metrics.precision * 100).toFixed(1)}%
                          </div>
                          <div className="text-[9px] text-gray-500 mt-1">({data.metrics.false_positives} F/P)</div>
                        </div>
                        <div className="bg-black/40 border border-white/5 p-3 rounded text-center flex flex-col justify-center">
                          <div className="text-[10px] text-gray-400 uppercase tracking-widest mb-1">Confidence Calibration</div>
                          <div className={`text-xl font-bold font-mono ${data.metrics.confidence_accuracy_correlation > 0 ? "text-emerald-400" : "text-yellow-400"}`}>
                              {(data.metrics.confidence_accuracy_correlation * 100).toFixed(1)}%
                          </div>
                        </div>
                        <div className="bg-black/40 border border-white/5 p-3 rounded text-left overflow-y-auto max-h-24">
                          <div className="text-[10px] text-gray-400 uppercase tracking-widest mb-1 sticky top-0 bg-transparent flex justify-between">Source Dist</div>
                          <div className="space-y-1 mt-2">
                              {Object.entries(data.metrics.source_distribution || {}).map(([src, pct], i) => (
                                <div key={i} className="flex justify-between items-center text-[10px] font-mono text-gray-300 bg-white/5 px-2 py-1 rounded">
                                    <span className="truncate pr-2 max-w-[80px]" title={src as string}>{src as string}</span>
                                    <span className="text-emerald-400">{(pct as number).toFixed(0)}%</span>
                                </div>
                              ))}
                          </div>
                        </div>
                    </div>
                  </div>
                )}

                {/* Discovery Certainty */}
                <CertaintyBadge certainty={data.discovery_certainty} />

                {data.detected_endpoints && data.detected_endpoints.length > 0 ? (
                  <ul className="space-y-3">
                    {data.detected_endpoints.map((ep: any, i: number) => (
                      <ApiTestRow key={i} endpoint={ep} baseUrl={data.base_url} />
                    ))}
                  </ul>
                ) : (
                  <div className="text-center py-12 px-4 border border-dashed border-white/10 rounded-xl">
                    <p className="text-gray-500">{t.noData}</p>
                  </div>
                )}
              </div>
            </div>
            
            <div className="w-full max-w-4xl p-4 rounded-xl bg-black/50 border border-white/5 font-mono text-xs text-gray-500 flex justify-between">
              <span>Discoverer Module</span>
              <span>{t.timestamp} {new Date().toLocaleTimeString()}</span>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
      <style jsx global>{`
        @keyframes shimmer { 100% { transform: translateX(100%); } }
        .glow-emerald { box-shadow: 0 0 20px rgba(52, 211, 153, 0.4); }
      `}</style>
    </div>
  );
}

interface Evidence {
  source_type: string;
  snippet: string;
  reason: string;
  line_number?: number;
}

interface ParamDetail {
  name: string;
  param_type: string;
  data_type: string;
}

interface DiscoveryMetrics {
  total_endpoints: number;
  valid_endpoints: number;
  false_positives: number;
  precision: number;
  source_distribution: Record<string, number>;
  confidence_accuracy_correlation: number;
}

interface RuntimeVerification {
  is_valid: boolean;
  best_method: string;
  status_code: number;
  response_time_ms: number;
  has_body: boolean;
  content_type?: string;
  server?: string;
}

interface EndpointDetail {
  path: string;
  method_prediction: string;
  parameters: ParamDetail[];
  auth_probability: number;
  auth_likelihood: string;
  confidence_score: number;
  evidences: Evidence[];
  runtime_verification?: RuntimeVerification;
}

function ApiTestRow({ endpoint, baseUrl }: { endpoint: EndpointDetail, baseUrl: string }) {
  const [isOpen, setIsOpen] = useState(false);
  const [showEvidence, setShowEvidence] = useState(false);
  
  const generateDummyValue = (type: string) => {
    switch (type) {
      case 'email': return "test@example.com";
      case 'id': return "12345";
      case 'token': return "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
      case 'number': return 1;
      case 'password': return "P@ssw0rd123";
      default: return "";
    }
  };

  const initialMethod = endpoint.runtime_verification?.best_method 
    ? endpoint.runtime_verification.best_method as any 
    : (endpoint.method_prediction !== "UNKNOWN" ? endpoint.method_prediction : "GET") as any;
  const [method, setMethod] = useState<"GET" | "POST" | "PUT" | "DELETE">(initialMethod);
  
  const initialBody = ["POST", "PUT"].includes(initialMethod) && endpoint.parameters.length > 0
    ? JSON.stringify(endpoint.parameters.reduce((acc, p) => p.param_type === 'body' || p.param_type === 'query' ? ({...acc, [p.name]: generateDummyValue(p.data_type)}) : acc, {} as any), null, 2)
    : "";
  const [body, setBody] = useState(initialBody);
  
  const initialBodyType = endpoint.evidences.some(e => e.source_type.includes("HTML Form")) ? "form" : "json";
  const [bodyType, setBodyType] = useState<"json" | "form">(initialBodyType as "json" | "form");

  const initialQueryParams = endpoint.parameters
    .filter(p => p.param_type === 'query' || p.param_type === 'path')
    .map(p => ({ key: p.name, value: generateDummyValue(p.data_type).toString(), type: p.param_type }));
  
  const [queryParams, setQueryParams] = useState(initialQueryParams);

  const [result, setResult] = useState<any>(null);
  const [isTesting, setIsTesting] = useState(false);

  const handleTest = async () => {
    setIsTesting(true);
    setResult(null);
    try {
      let finalBody = body;
      if (bodyType === "form" && body) {
         try {
             const parsedObj = JSON.parse(body);
             finalBody = new URLSearchParams(parsedObj).toString();
         } catch(e) { /* ignore invalid json while parsing */ }
      }

      let finalPath = endpoint.path;
      if (method === "GET") {
          let qstr = new URLSearchParams();
          queryParams.forEach(q => {
             if (q.type === 'path') {
                finalPath = finalPath.replace(`[VAR]`, q.value).replace(`:${q.key}`, q.value);
             } else if (q.value) {
                qstr.append(q.key, q.value);
             }
          });
          let qs = qstr.toString();
          if (qs) finalPath += (finalPath.includes('?') ? '&' : '?') + qs;
      }

      const fullUrl = finalPath.startsWith('http') 
          ? finalPath 
          : `${baseUrl.replace(/\/$/, '')}${finalPath.startsWith('/') ? finalPath : `/${finalPath}`}`;
          
      const parsedData = await reportApi.proxyRequest(
        fullUrl, 
        method, 
        ["POST", "PUT"].includes(method) && body ? finalBody : undefined
      );
      setResult({ status: 200, ok: true, data: parsedData });
    } catch (err: any) {
      setResult({ 
        status: err.response?.status || 0, 
        ok: false, 
        data: err.response?.data || err.message || "Proxy server connection error" 
      });
    }
    setIsTesting(false);
  };

  return (
    <li className="flex flex-col bg-black/40 border border-white/5 rounded-lg overflow-hidden transition-colors hover:border-emerald-500/30">
      <div 
        className="flex p-3 items-center justify-between cursor-pointer hover:bg-emerald-500/5 group"
        onClick={() => setIsOpen(!isOpen)}
      >
        <div className="flex items-center gap-3 overflow-hidden flex-1">
          <span className="truncate text-sm font-mono text-gray-300" title={endpoint.path}>{endpoint.path}</span>
          <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full border ${
              endpoint.method_prediction === 'POST' ? 'border-blue-500/30 text-blue-400 bg-blue-500/10' :
              endpoint.method_prediction === 'GET' ? 'border-emerald-500/30 text-emerald-400 bg-emerald-500/10' :
              'border-gray-500/30 text-gray-400 bg-gray-500/10'
            }`}>
            {endpoint.method_prediction}
          </span>
          <span className="text-[10px] font-bold px-2 py-0.5 rounded-full border border-purple-500/30 text-purple-400 bg-purple-500/10 hidden sm:block">
             Conf: {(endpoint.confidence_score * 100).toFixed(0)}%
          </span>
          {endpoint.runtime_verification?.is_valid === true && (
             <span className="flex items-center gap-1 text-[10px] bg-emerald-500/20 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/30" title={`HTTP ${endpoint.runtime_verification.status_code} | ${endpoint.runtime_verification.response_time_ms}ms`}>
                <CheckCircle2 className="w-3 h-3" /> <span className="hidden sm:inline">Verified</span>
             </span>
          )}
          {endpoint.runtime_verification?.is_valid === false && (
             <span className="flex items-center gap-1 text-[10px] bg-red-500/20 text-red-400 px-2 py-0.5 rounded border border-red-500/30" title={`HTTP ${endpoint.runtime_verification.status_code} | ${endpoint.runtime_verification.response_time_ms}ms`}>
                <XCircle className="w-3 h-3" /> F/P
             </span>
          )}
          {endpoint.evidences && endpoint.evidences.length > 0 && (
             <span className="text-[10px] uppercase font-bold text-gray-400 border border-gray-600/50 bg-gray-800/30 px-2 py-0.5 rounded-full whitespace-nowrap hidden lg:block" title={Array.from(new Set(endpoint.evidences.map(e => e.source_type))).join(", ")}>
               {endpoint.evidences.length} {endpoint.evidences.length > 1 ? "Sources" : "Source"} ({Array.from(new Set(endpoint.evidences.map(e => e.source_type))).join(", ")})
             </span>
          )}
        </div>
        <button className="px-3 py-1 bg-emerald-500/20 text-emerald-400 rounded text-xs font-bold hover:bg-emerald-500/30 transition-colors shrink-0 flex items-center gap-1 opacity-0 group-hover:opacity-100">
          <Play className="w-3 h-3" /> Test
        </button>
      </div>

      <AnimatePresence>
        {isOpen && (
          <motion.div 
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="border-t border-white/5 bg-black/60 p-4"
          >
            {/* Context Metrics Bar */}
            <div className="flex gap-2 mb-4 flex-wrap">
               {endpoint.auth_likelihood && endpoint.auth_likelihood !== "None" && (
                 <span className={`text-[10px] ${endpoint.auth_likelihood === 'Likely' ? 'text-red-400 bg-red-400/10 border-red-400/20' : 'text-yellow-400 bg-yellow-400/10 border-yellow-400/20'} border px-2 py-1 rounded flex items-center gap-1`}>
                   <ShieldAlert className="w-3 h-3"/> Auth: {endpoint.auth_likelihood} ({(endpoint.auth_probability * 100).toFixed(0)}%)
                 </span>
               )}
               {endpoint.parameters && endpoint.parameters.length > 0 && (
                 <span className="text-[10px] text-purple-400 bg-purple-400/10 border border-purple-400/20 px-2 py-1 rounded flex gap-2">
                   {endpoint.parameters.map((p, idx) => (
                      <span key={idx}><span className="opacity-50 font-bold">[{p.param_type}]</span> {p.name}</span>
                   ))}
                 </span>
               )}
               {endpoint.evidences && endpoint.evidences.length > 0 && (
                 <button 
                    onClick={(e) => { e.stopPropagation(); setShowEvidence(!showEvidence); }}
                    className="text-[10px] text-blue-400 bg-blue-400/10 border border-blue-400/20 hover:bg-blue-400/20 px-2 py-1 rounded transition-colors"
                 >
                   {showEvidence ? "Hide Evidence" : "View Evidence Snippets"}
                 </button>
               )}
            </div>

            {/* Evidence Viewer */}
            <AnimatePresence>
              {showEvidence && endpoint.evidences && (
                 <motion.div initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: "auto" }} exit={{ opacity: 0, height: 0 }} className="mb-4 space-y-2">
                   {endpoint.evidences.map((evi, idx) => (
                      <div key={idx} className="bg-black border border-white/10 rounded overflow-hidden">
                        <div className="bg-white/5 text-[9px] text-gray-400 px-2 py-1 uppercase tracking-wider font-bold border-b border-white/5 flex justify-between">
                           <span>Source: {evi.source_type}</span>
                           {evi.line_number && <span>Line: {evi.line_number}</span>}
                        </div>
                        {evi.reason && (
                           <div className="text-[10px] text-emerald-300/80 px-2 py-1 border-b border-white/5 bg-emerald-950/20">
                              <strong>Reason:</strong> {evi.reason}
                           </div>
                        )}
                        <pre className="p-2 text-[10px] text-emerald-400/80 font-mono overflow-x-auto whitespace-pre-wrap">
                           {evi.snippet}
                        </pre>
                      </div>
                   ))}
                 </motion.div>
              )}
            </AnimatePresence>

            {/* Advanced Runtime Verification Details */}
            {endpoint.runtime_verification && (
               <div className="mb-4 bg-black/60 border border-emerald-500/20 rounded p-3 grid grid-cols-2 sm:grid-cols-4 gap-2">
                 <div className="flex flex-col">
                   <span className="text-[9px] uppercase tracking-widest text-gray-500 font-bold">Status</span>
                   <span className={`text-xs font-mono font-bold ${endpoint.runtime_verification.is_valid ? 'text-emerald-400' : 'text-red-400'}`}>{endpoint.runtime_verification.status_code}</span>
                 </div>
                 <div className="flex flex-col">
                   <span className="text-[9px] uppercase tracking-widest text-gray-500 font-bold">Latency</span>
                   <span className="text-xs font-mono font-bold text-gray-300">{endpoint.runtime_verification.response_time_ms}ms</span>
                 </div>
                 <div className="flex flex-col">
                   <span className="text-[9px] uppercase tracking-widest text-gray-500 font-bold">Content Type</span>
                   <span className="text-[10px] font-mono font-bold text-gray-400 truncate" title={endpoint.runtime_verification.content_type || 'N/A'}>{endpoint.runtime_verification.content_type || 'N/A'}</span>
                 </div>
                 <div className="flex flex-col">
                   <span className="text-[9px] uppercase tracking-widest text-gray-500 font-bold">Server</span>
                   <span className="text-[10px] font-mono font-bold text-gray-400 truncate" title={endpoint.runtime_verification.server || 'Unknown'}>{endpoint.runtime_verification.server || 'Unknown'}</span>
                 </div>
               </div>
            )}

            {/* Test Controller */}
            <div className="flex gap-2 mb-3">
              <select 
                value={method} 
                onChange={e => setMethod(e.target.value as any)}
                className="bg-black/50 border border-white/10 text-white text-xs rounded px-2 py-1 outline-none focus:border-emerald-500 font-mono"
              >
                <option value="GET">GET</option>
                <option value="POST">POST</option>
                <option value="PUT">PUT</option>
                <option value="DELETE">DELETE</option>
              </select>
              <button 
                onClick={handleTest}
                disabled={isTesting}
                className="bg-emerald-500 text-black font-bold px-4 py-1 rounded text-xs hover:bg-emerald-400 transition-colors flex items-center gap-2 disabled:opacity-50"
              >
                {isTesting ? <Loader2 className="w-3 h-3 animate-spin"/> : <Send className="w-3 h-3"/>}
                Send
              </button>
            </div>

            {method === "GET" && queryParams.length > 0 && (
              <div className="mb-3">
                 <span className="text-[10px] text-gray-500 uppercase font-bold tracking-widest block mb-2">Query / Path Builder</span>
                 <div className="space-y-2">
                   {queryParams.map((q, idx) => (
                      <div key={idx} className="flex gap-2">
                         <span className="text-xs text-gray-400 bg-white/5 border border-white/10 px-2 py-1 rounded flex items-center shrink-0 min-w-24">
                           {q.type === 'path' ? '[PATH]' : '[QUERY]'} {q.key}
                         </span>
                         <input 
                           type="text" 
                           value={q.value}
                           onChange={(e) => {
                              const newParams = [...queryParams];
                              newParams[idx].value = e.target.value;
                              setQueryParams(newParams);
                           }}
                           className="flex-1 bg-black/50 border border-white/10 text-white text-xs font-mono px-2 py-1 rounded outline-none focus:border-emerald-500"
                         />
                      </div>
                   ))}
                 </div>
              </div>
            )}

            {["POST", "PUT"].includes(method) && (
              <div className="mb-3">
                 <div className="flex gap-4 border-b border-white/10 mb-2">
                    <button 
                      onClick={() => setBodyType("json")} 
                      className={`text-[10px] uppercase font-bold py-1 px-2 ${bodyType === "json" ? "text-emerald-400 border-b-2 border-emerald-400" : "text-gray-500 hover:text-white"}`}>
                      JSON Body
                    </button>
                    <button 
                      onClick={() => setBodyType("form")} 
                      className={`text-[10px] uppercase font-bold py-1 px-2 ${bodyType === "form" ? "text-emerald-400 border-b-2 border-emerald-400" : "text-gray-500 hover:text-white"}`}>
                      Form Data Object
                    </button>
                 </div>
                 <div className="relative">
                   <span className="absolute right-2 top-2 text-[10px] text-gray-500 font-bold uppercase pointer-events-none">{bodyType === "json" ? "JSON Editor" : "Payload Editor"}</span>
                   <textarea 
                     value={body}
                     onChange={e => setBody(e.target.value)}
                     placeholder='{"key": "value"}'
                     className="w-full bg-black/50 border border-white/10 rounded p-2 text-xs font-mono text-gray-300 min-h-[100px] outline-none focus:border-emerald-500"
                   />
                 </div>
              </div>
            )}

            {result && (
              <div className={`p-3 rounded border ${result.ok ? "border-emerald-500/30 bg-emerald-500/10" : "border-red-500/30 bg-red-500/10"}`}>
                <div className="flex items-center gap-2 mb-2">
                  <span className={`text-xs font-bold px-2 py-0.5 rounded ${result.ok ? "bg-emerald-500 text-black" : "bg-red-500 text-white"}`}>
                    {result.status === 0 ? "ERROR" : result.status}
                  </span>
                </div>
                <pre className="text-[10px] font-mono text-gray-300 max-h-40 overflow-y-auto custom-scrollbar">
                  {typeof result.data === "object" ? JSON.stringify(result.data, null, 2) : result.data}
                </pre>
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </li>
  );
}
