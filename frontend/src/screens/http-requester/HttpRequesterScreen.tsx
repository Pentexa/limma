"use client";

import { useState, useRef } from "react";
import { cn } from "@/shared/lib/utils";
import { proxyRequest } from "@/features/blind-scan/api/blind-scan-api";
import { Globe, Loader2, Send, ArrowDown, ArrowUp, Copy, CheckCircle } from "lucide-react";

type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
const METHODS: HttpMethod[] = ["GET", "POST", "PUT", "DELETE", "PATCH"];

interface RequestEntry {
  id: number;
  method: string;
  url: string;
  body?: string;
  response: Record<string, unknown> | null;
  error: string | null;
  timestamp: number;
  duration: number;
}

export function HttpRequesterScreen() {
  const [url, setUrl] = useState("");
  const [method, setMethod] = useState<HttpMethod>("GET");
  const [body, setBody] = useState("");
  const [isPending, setIsPending] = useState(false);
  const [history, setHistory] = useState<RequestEntry[]>([]);
  const [copied, setCopied] = useState(false);
  const entryIdRef = useRef(0);

  async function handleSend() {
    if (!url) return;
    setIsPending(true);
    const start = Date.now();
    const currentId = entryIdRef.current++;


    try {
      const result = await proxyRequest({
        url,
        method,
        body: method !== "GET" ? body : undefined,
      });
      setHistory(prev => [{
        id: currentId,
        method,
        url,
        body: method !== "GET" ? body : undefined,
        response: result,
        error: null,
        timestamp: Date.now(),
        duration: Date.now() - start,
      }, ...prev.slice(0, 49)]);
    } catch (err) {
      setHistory(prev => [{
        id: currentId,
        method,
        url,
        body: method !== "GET" ? body : undefined,
        response: null,
        error: err instanceof Error ? err.message : "Request failed",
        timestamp: Date.now(),
        duration: Date.now() - start,
      }, ...prev.slice(0, 49)]);
    } finally {
      setIsPending(false);
    }
  }

  function copyResponse(entry: RequestEntry) {
    navigator.clipboard.writeText(JSON.stringify(entry.response, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  const METHOD_COLORS: Record<string, string> = {
    GET: "text-primary",
    POST: "text-verified",
    PUT: "text-attention",
    DELETE: "text-risk",
    PATCH: "text-foreground",
  };

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden">
      {/* Top bar */}
      <div className="shrink-0 flex items-center gap-3 px-4 py-2.5 border-b border-border bg-card/50">
        <Globe className="h-4 w-4 text-primary" />
        <h2 className="text-[13px] font-bold tracking-tight">HTTP Requester</h2>
        <span className="text-[10px] text-muted-foreground">Proxy through backend</span>
      </div>

      {/* Request builder */}
      <div className="shrink-0 px-4 py-3 border-b border-border bg-card/30 space-y-2">
        <div className="flex items-center gap-2">
          {/* Method selector */}
          <select
            value={method}
            onChange={(e) => setMethod(e.target.value as HttpMethod)}
            className="h-8 px-2 text-[11px] font-mono font-bold bg-muted/20 border border-border rounded focus:border-primary focus:outline-none transition-colors w-[90px]"
          >
            {METHODS.map(m => <option key={m} value={m}>{m}</option>)}
          </select>
          {/* URL */}
          <input
            type="text"
            placeholder="https://api.example.com/endpoint"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            className="flex-1 h-8 px-3 text-[11px] font-mono bg-muted/20 border border-border rounded focus:border-primary focus:outline-none transition-colors"
          />
          <button
            className="flex items-center gap-1.5 px-4 h-8 rounded text-[10px] font-semibold bg-primary/10 text-primary hover:bg-primary/20 transition-colors disabled:opacity-50"
            disabled={!url || isPending}
            onClick={handleSend}
          >
            {isPending ? <Loader2 className="h-3 w-3 animate-spin" /> : <Send className="h-3 w-3" />}
            Send
          </button>
        </div>
        {/* Body (for POST/PUT/PATCH) */}
        {method !== "GET" && method !== "DELETE" && (
          <textarea
            placeholder='{"key": "value"}'
            value={body}
            onChange={(e) => setBody(e.target.value)}
            className="w-full h-20 px-3 py-2 text-[10px] font-mono bg-muted/20 border border-border rounded focus:border-primary focus:outline-none resize-y transition-colors"
          />
        )}
      </div>

      {/* History */}
      <div className="flex-1 overflow-y-auto p-4 space-y-2">
        {history.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground/30">
            <Globe className="h-8 w-8 mb-3 opacity-40" />
            <span className="text-[12px]">Send a request to get started</span>
          </div>
        ) : (
          history.map((entry) => (
            <div key={entry.id} className="panel">
              <div className="panel-header">
                <div className="flex items-center gap-2">
                  <ArrowUp className="h-3 w-3 text-muted-foreground/40" />
                  <span className={cn("text-[10px] font-mono font-bold", METHOD_COLORS[entry.method])}>{entry.method}</span>
                  <span className="text-[10px] font-mono text-foreground/70 truncate max-w-[400px]">{entry.url}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-[9px] font-mono text-muted-foreground tabular-nums">{entry.duration}ms</span>
                  {entry.response && (
                    <button
                      className="text-muted-foreground/30 hover:text-primary transition-colors"
                      onClick={() => copyResponse(entry)}
                    >
                      {copied ? <CheckCircle className="h-3 w-3 text-verified" /> : <Copy className="h-3 w-3" />}
                    </button>
                  )}
                </div>
              </div>
              <div className="panel-body p-0">
                {entry.error ? (
                  <div className="px-4 py-3 text-[10px] text-risk">{entry.error}</div>
                ) : (
                  <pre className="text-[9px] font-mono bg-muted/10 px-4 py-3 overflow-x-auto max-h-[200px] overflow-y-auto whitespace-pre-wrap break-all text-foreground/80 leading-relaxed">
                    <span className="flex items-center gap-1.5 mb-1 text-muted-foreground/40"><ArrowDown className="h-2.5 w-2.5" /> Response</span>
                    {JSON.stringify(entry.response, null, 2)}
                  </pre>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
