import React, { useState } from "react";
import { useConsents, useGrantConsent, useRevokeConsent } from "../model/use-consents";
import { ShieldAlert, Shield, CheckCircle, XCircle, Loader2, Trash2, Globe, Clock, User } from "lucide-react";
import { cn } from "@/shared/lib/utils";

export function ConsentManagementPanel() {
  const { data: consents = [], isLoading } = useConsents();
  const grantMutation = useGrantConsent();
  const revokeMutation = useRevokeConsent();

  const [domain, setDomain] = useState("");
  const [level, setLevel] = useState("L3ActiveWithConsent");
  const [hours, setHours] = useState<number>(24);
  const [requestedBy, setRequestedBy] = useState("admin"); // Ideally from auth

  const activeConsents = consents.filter((c: any) => !c.revoked && (!c.expires_at || new Date(c.expires_at) > new Date()));
  const pastConsents = consents.filter((c: any) => c.revoked || (c.expires_at && new Date(c.expires_at) <= new Date()));

  const handleGrant = (e: React.FormEvent) => {
    e.preventDefault();
    if (!domain) return;
    grantMutation.mutate({
      target_domain: domain,
      requested_by: requestedBy,
      scope_level: level,
      expires_in_hours: hours
    }, {
      onSuccess: () => {
        setDomain("");
      }
    });
  };

  return (
    <div className="space-y-6">
      {/* Grant Form */}
      <div className="bg-card/30 backdrop-blur-sm border border-border/60 rounded-xl overflow-hidden p-5">
        <h3 className="flex items-center gap-2 text-[13px] font-bold text-foreground/90 mb-4 pb-3 border-b border-border/40">
          <ShieldAlert className="w-4 h-4 text-primary" />
          Grant New Execution Consent
        </h3>
        
        <form onSubmit={handleGrant} className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div className="space-y-1.5">
            <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-widest">Target Domain / URL</label>
            <input 
              required
              placeholder="e.g. https://target.com"
              value={domain}
              onChange={e => setDomain(e.target.value)}
              className="w-full bg-background border border-border rounded px-3 py-1.5 text-[11px] font-mono focus:border-primary outline-none"
            />
          </div>
          <div className="space-y-1.5">
            <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-widest">Safety Level</label>
            <select 
              value={level}
              onChange={e => setLevel(e.target.value)}
              className="w-full bg-background border border-border rounded px-3 py-1.5 text-[11px] font-mono focus:border-primary outline-none text-red-400 font-bold"
            >
              <option value="L3ActiveWithConsent">L3 - Active Exploitation</option>
            </select>
          </div>
          <div className="space-y-1.5">
            <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-widest">Duration (Hours)</label>
            <input 
              type="number"
              min="1"
              max="720"
              value={hours}
              onChange={e => setHours(parseInt(e.target.value))}
              className="w-full bg-background border border-border rounded px-3 py-1.5 text-[11px] font-mono focus:border-primary outline-none"
            />
          </div>
          <div className="flex items-end">
            <button 
              type="submit"
              disabled={grantMutation.isPending || !domain}
              className="w-full flex items-center justify-center gap-2 bg-primary/10 hover:bg-primary/20 text-primary border border-primary/20 rounded px-4 py-1.5 text-[11px] font-bold transition-all disabled:opacity-50 h-[30px]"
            >
              {grantMutation.isPending ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <CheckCircle className="w-3.5 h-3.5" />}
              Grant Consent
            </button>
          </div>
        </form>
      </div>

      {/* Active Consents */}
      <div className="bg-card/30 backdrop-blur-sm border border-border/60 rounded-xl overflow-hidden p-5">
        <h3 className="flex items-center justify-between text-[13px] font-bold text-foreground/90 mb-4 pb-3 border-b border-border/40">
          <div className="flex items-center gap-2">
            <Shield className="w-4 h-4 text-emerald-400" />
            Active Consents
          </div>
          <span className="bg-emerald-500/10 text-emerald-400 text-[10px] px-2 py-0.5 rounded font-mono border border-emerald-500/20">
            {activeConsents.length} Active
          </span>
        </h3>
        
        {isLoading ? (
          <div className="flex items-center gap-2 text-muted-foreground text-[11px]">
            <Loader2 className="w-3.5 h-3.5 animate-spin" /> Loading...
          </div>
        ) : activeConsents.length === 0 ? (
          <div className="text-[11px] text-muted-foreground italic">No active consents found.</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {activeConsents.map((c: any) => (
              <div key={c.id} className="bg-background/50 border border-emerald-500/30 rounded-lg p-3 flex justify-between items-start">
                <div className="space-y-2 min-w-0">
                  <div className="flex items-center gap-2">
                    <Globe className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                    <span className="text-[12px] font-bold font-mono text-emerald-400 truncate">{c.target_domain}</span>
                  </div>
                  <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-[10px] text-muted-foreground/80">
                    <div className="flex items-center gap-1.5"><User className="w-3 h-3" /> {c.granted_by}</div>
                    <div className="flex items-center gap-1.5">
                      <Clock className="w-3 h-3" /> 
                      Expires: {c.expires_at ? new Date(c.expires_at).toLocaleString() : "Never"}
                    </div>
                  </div>
                </div>
                <button 
                  onClick={() => revokeMutation.mutate({ id: c.id, target_domain: c.target_domain })}
                  disabled={revokeMutation.isPending}
                  className="p-1.5 bg-red-500/10 text-red-400 hover:bg-red-500/20 rounded transition-colors shrink-0"
                  title="Revoke Consent"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Past Consents */}
      {pastConsents.length > 0 && (
        <div className="bg-card/30 backdrop-blur-sm border border-border/60 rounded-xl overflow-hidden p-5 opacity-70">
          <h3 className="flex items-center gap-2 text-[13px] font-bold text-foreground/90 mb-4 pb-3 border-b border-border/40">
            <XCircle className="w-4 h-4 text-muted-foreground" />
            Revoked / Expired Consents
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {pastConsents.map((c: any) => (
              <div key={c.id} className="bg-background/30 border border-border/30 rounded-lg p-3">
                <div className="text-[11px] font-mono text-muted-foreground line-through decoration-red-500/50 mb-1 truncate">{c.target_domain}</div>
                <div className="text-[9px] text-muted-foreground/50">
                  {c.revoked ? `Revoked at ${new Date(c.revoked_at!).toLocaleString()}` : "Expired"}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
