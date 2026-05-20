"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import { useBurpSessions, useBurpFindings, useBurpHandshake } from "@/features/connect-burp/model/use-burp";
import { burpImportTraffic } from "@/features/connect-burp/api/connect-burp";
import { IntelStreamLog } from "@/screens/scanner/components/IntelStreamLog";
import { Loader2, Plug, Plus, Bug, Upload } from "lucide-react";

export function IntegrationsScreen() {
  const { data: sessions = [], isLoading } = useBurpSessions();
  const handshakeMutation = useBurpHandshake();
  const [selectedSession, setSelectedSession] = useState<string | undefined>();
  const { data: findings = [], isLoading: findingsLoading } = useBurpFindings(selectedSession);
  const [importing, setImporting] = useState(false);

  async function handleImportTraffic() {
    if (!selectedSession) return;
    setImporting(true);
    try {
      await burpImportTraffic(selectedSession, {});
    } catch (err) {
      console.error("Import traffic failed:", err);
    } finally {
      setImporting(false);
    }
  }

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden">
      <div className="shrink-0 flex items-center justify-between gap-4 px-4 py-2.5 border-b border-border bg-card/50">
        <div className="flex items-center gap-1.5">
          <Plug className="h-4 w-4 text-primary" />
          <h2 className="text-[13px] font-bold tracking-tight">Integrations</h2>
        </div>
        <button
          className="flex items-center gap-1.5 px-3 py-1.5 rounded text-[10px] font-semibold bg-primary/10 text-primary hover:bg-primary/20 transition-colors disabled:opacity-50"
          disabled={handshakeMutation.isPending}
          onClick={() => handshakeMutation.mutate("1.0.0")}
        >
          {handshakeMutation.isPending ? <Loader2 className="h-3 w-3 animate-spin" /> : <Plus className="h-3 w-3" />}
          New Burp Session
        </button>
        {selectedSession && (
          <button
            className="flex items-center gap-1.5 px-3 py-1.5 rounded text-[10px] font-semibold bg-attention/10 text-attention hover:bg-attention/20 transition-colors disabled:opacity-50"
            disabled={importing}
            onClick={handleImportTraffic}
          >
            {importing ? <Loader2 className="h-3 w-3 animate-spin" /> : <Upload className="h-3 w-3" />}
            Import Traffic
          </button>
        )}
      </div>

      <div className="flex-1 flex min-h-0 overflow-hidden">
        {/* Sessions sidebar */}
        <div className="shrink-0 w-[240px] border-r border-border overflow-y-auto bg-card/30 p-3 space-y-3">
          <span className="text-[9px] font-semibold text-muted-foreground/60 uppercase tracking-widest block">
            Burp Suite Sessions
          </span>
          {isLoading ? (
            <div className="flex items-center justify-center py-8"><Loader2 className="h-4 w-4 animate-spin text-muted-foreground" /></div>
          ) : sessions.length === 0 ? (
            <div className="text-[11px] text-muted-foreground/50 py-4 text-center">No sessions</div>
          ) : (
            sessions.map((s) => (
              <div
                key={s.session_id}
                className={cn(
                  "px-3 py-2 rounded cursor-pointer transition-colors",
                  selectedSession === s.session_id ? "bg-primary/10 border border-primary/30" : "hover:bg-muted/30 border border-transparent"
                )}
                onClick={() => setSelectedSession(s.session_id)}
              >
                <div className="flex items-center gap-2 mb-1">
                  <Bug className="h-3 w-3 text-attention" />
                  <span className="text-[11px] font-medium text-foreground font-mono">{s.session_id.slice(0, 12)}</span>
                </div>
                <div className="text-[9px] text-muted-foreground/50">
                  v{s.plugin_version} · {new Date(s.created_at).toLocaleDateString()}
                </div>
              </div>
            ))
          )}
        </div>

        {/* Findings */}
        <div className="flex-1 min-w-0 overflow-y-auto p-4">
          {!selectedSession ? (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground/50">
              <Bug className="h-8 w-8 mb-3 opacity-40" />
              <span className="text-[12px]">Select a session to view findings</span>
            </div>
          ) : findingsLoading ? (
            <div className="flex items-center justify-center h-full">
              <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
            </div>
          ) : findings.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground/50">
              <span className="text-[12px]">No findings in this session</span>
            </div>
          ) : (
            <div className="panel">
              <div className="panel-header">
                <span className="panel-title">Burp Findings</span>
                <span className="text-[10px] text-muted-foreground">{findings.length} findings</span>
              </div>
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="text-[9px] font-semibold text-muted-foreground/70 uppercase tracking-widest bg-muted/20 border-b border-border">
                    <th className="px-3 py-1.5 w-[72px]">Severity</th>
                    <th className="px-2 py-1.5">Name</th>
                    <th className="px-2 py-1.5 w-[200px]">URL</th>
                    <th className="px-2 py-1.5">Detail</th>
                  </tr>
                </thead>
                <tbody>
                  {findings.map((f) => (
                    <tr key={f.id} className="border-b border-border/50 hover:bg-muted/20 transition-colors">
                      <td className="px-3 py-2">
                        <span className={cn("sev-badge text-[9px]", `sev-badge-${f.severity?.toLowerCase()}`)}>{f.severity}</span>
                      </td>
                      <td className="px-2 py-2 text-[11px] font-medium text-foreground">{f.name}</td>
                      <td className="px-2 py-2 text-[10px] font-mono text-muted-foreground truncate max-w-[200px]">{f.url}</td>
                      <td className="px-2 py-2 text-[10px] text-muted-foreground/70 truncate max-w-[300px]">{f.detail}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {/* Burp SSE Event Stream */}
        {selectedSession && (
          <div className="shrink-0 border-t border-border">
            <IntelStreamLog
              streamPath={`/api/burp/stream/${selectedSession}`}
              targetUrl={selectedSession}
              isActive={!!selectedSession}
            />
          </div>
        )}
      </div>
    </div>
  );
}
