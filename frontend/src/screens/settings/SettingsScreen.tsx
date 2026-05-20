"use client";

import { useState, useEffect } from "react";
import { cn } from "@/shared/lib/utils";
import { useSettingsProfiles, useScanProfiles, useUpdateProfile } from "@/features/update-settings/model/use-settings";
import { 
  Settings, Loader2, Save, User, Radar, CheckCircle, XCircle, RotateCcw, 
  AlertTriangle, Globe, Crosshair, Network, FileText, ShieldAlert 
} from "lucide-react";
import type { ApiSettingsProfile } from "@/shared/types/api";

type Tab = "general" | "profiles";

function ProfileEditor({ profile, onSave, isSaving }: { profile: ApiSettingsProfile, onSave: (p: ApiSettingsProfile) => void, isSaving: boolean }) {
  const [local, setLocal] = useState<ApiSettingsProfile>(JSON.parse(JSON.stringify(profile)));
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setLocal(JSON.parse(JSON.stringify(profile)));
    setError(null);
  }, [profile]);

  const isDirty = JSON.stringify(local) !== JSON.stringify(profile);

  const validate = (): boolean => {
    if (local.global.timeout_ms <= 0) { setError("Global timeout must be > 0"); return false; }
    if (local.global.rate_limit_req_per_sec <= 0) { setError("Rate limit must be > 0"); return false; }
    if (local.scanner.max_depth < 0) { setError("Max depth must be >= 0"); return false; }
    if (local.global.use_proxy && !local.global.proxy_url) { setError("Proxy URL is required when proxy is enabled"); return false; }
    setError(null);
    return true;
  };

  const handleSave = () => {
    if (validate()) {
      onSave(local);
    }
  };

  const handleDiscard = () => {
    setLocal(JSON.parse(JSON.stringify(profile)));
    setError(null);
  };

  const updateGlobal = (key: keyof ApiSettingsProfile["global"], value: any) => 
    setLocal(p => ({ ...p, global: { ...p.global, [key]: value } }));
    
  const updateScanner = (key: keyof ApiSettingsProfile["scanner"], value: any) => 
    setLocal(p => ({ ...p, scanner: { ...p.scanner, [key]: value } }));
    
  const updateInvestigator = (key: keyof ApiSettingsProfile["investigator"], value: any) => 
    setLocal(p => ({ ...p, investigator: { ...p.investigator, [key]: value } }));
    
  const updateApi = (key: keyof ApiSettingsProfile["api_discovery"], value: any) => 
    setLocal(p => ({ ...p, api_discovery: { ...p.api_discovery, [key]: value } }));
    
  const updateServices = (key: keyof ApiSettingsProfile["services"], value: any) => 
    setLocal(p => ({ ...p, services: { ...p.services, [key]: value } }));
    
  const updateForms = (key: keyof ApiSettingsProfile["forms"], value: any) => 
    setLocal(p => ({ ...p, forms: { ...p.forms, [key]: value } }));
    
  const updateAudit = (key: keyof ApiSettingsProfile["audit"], value: any) => 
    setLocal(p => ({ ...p, audit: { ...p.audit, [key]: value } }));
    
  const updateRules = (key: keyof ApiSettingsProfile["rules"], value: any) => 
    setLocal(p => ({ ...p, rules: { ...p.rules, [key]: value } }));
    
  const updateExploit = (key: keyof ApiSettingsProfile["exploit"], value: any) => 
    setLocal(p => ({ ...p, exploit: { ...p.exploit, [key]: value } }));

  const updateProxy = (key: keyof ApiSettingsProfile["proxy"], value: any) => 
    setLocal(p => ({ ...p, proxy: { ...p.proxy, [key]: value } }));

  const updateSessions = (key: keyof ApiSettingsProfile["sessions"], value: any) => 
    setLocal(p => ({ ...p, sessions: { ...p.sessions, [key]: value } }));

  // Common UI styles
  const inputClass = "w-full bg-background border border-border rounded px-3 py-1.5 text-[11px] font-mono text-foreground focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-all shadow-inner placeholder:text-muted-foreground/30";
  const selectClass = "w-full bg-background border border-border rounded px-2 py-1.5 text-[11px] font-mono text-foreground focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-all cursor-pointer";
  const labelClass = "text-[10px] font-semibold text-muted-foreground uppercase tracking-widest mb-1.5 block";
  const panelClass = "bg-card/40 backdrop-blur-sm border border-border/60 rounded-xl p-5 hover:border-border transition-colors";
  const panelHeaderClass = "flex items-center gap-2 text-[13px] font-bold text-foreground/90 mb-5 pb-3 border-b border-border/40";
  const checkboxWrapperClass = "flex items-center gap-2.5 p-2 rounded-md hover:bg-white/[0.02] transition-colors cursor-pointer";

  return (
    <div className="space-y-6 mt-2 relative">
      {/* Floating Action Bar (Sticky Bottom) */}
      <div className="sticky top-0 z-10 flex items-center justify-between bg-card/80 backdrop-blur-md border border-border/80 rounded-lg p-3 shadow-lg mb-6">
        <div className="flex items-center gap-2">
          {isDirty ? (
            <span className="flex h-2 w-2 rounded-full bg-amber-500 animate-pulse" />
          ) : (
            <span className="flex h-2 w-2 rounded-full bg-emerald-500" />
          )}
          <span className="text-[11px] font-medium text-muted-foreground">
            {isDirty ? "Unsaved changes" : "All changes saved"}
          </span>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={handleDiscard}
            disabled={!isDirty || isSaving}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-transparent hover:bg-white/[0.05] text-muted-foreground hover:text-foreground text-[11px] font-semibold rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <RotateCcw className="w-3.5 h-3.5" />
            Discard
          </button>
          <button
            onClick={handleSave}
            disabled={!isDirty || isSaving}
            className={cn(
              "flex items-center gap-1.5 px-4 py-1.5 text-[11px] font-semibold rounded transition-all",
              isDirty && !isSaving 
                ? "bg-primary text-primary-foreground shadow-[0_0_15px_rgba(var(--primary),0.3)] hover:shadow-[0_0_20px_rgba(var(--primary),0.5)] hover:scale-[1.02]" 
                : "bg-primary/50 text-primary-foreground/50 cursor-not-allowed"
            )}
          >
            {isSaving ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Save className="w-3.5 h-3.5" />}
            Save Profile
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3.5 flex items-start gap-2.5 animate-in fade-in slide-in-from-top-2">
          <XCircle className="w-4 h-4 text-red-400 mt-0.5 shrink-0" />
          <div>
            <h4 className="text-[11px] font-bold text-red-400 uppercase tracking-wider mb-0.5">Validation Error</h4>
            <p className="text-[11px] text-red-300/80">{error}</p>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 1. Global & Proxy */}
        <div className={panelClass}>
          <h3 className={panelHeaderClass}>
            <Globe className="w-4 h-4 text-primary" />
            Global & Proxy Configuration
          </h3>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className={labelClass}>Timeout (ms)</label>
                <input type="number" className={inputClass} value={local.global.timeout_ms} onChange={e => updateGlobal("timeout_ms", parseInt(e.target.value))} />
              </div>
              <div>
                <label className={labelClass}>Rate Limit (req/sec)</label>
                <input type="number" className={inputClass} value={local.global.rate_limit_req_per_sec} onChange={e => updateGlobal("rate_limit_req_per_sec", parseInt(e.target.value))} />
              </div>
            </div>
            <div>
              <label className={labelClass}>Target Scope (Wildcard supported)</label>
              <input type="text" className={inputClass} value={local.global.target_scope} onChange={e => updateGlobal("target_scope", e.target.value)} />
            </div>
            <div className="pt-2 border-t border-border/30">
              <label htmlFor="use_proxy" className={checkboxWrapperClass}>
                <input type="checkbox" id="use_proxy" checked={local.global.use_proxy} onChange={e => updateGlobal("use_proxy", e.target.checked)} className="rounded border-white/[0.2] bg-background/50 text-primary focus:ring-primary/50" />
                <span className="text-[11px] font-medium text-foreground/80 select-none">Route traffic through external proxy</span>
              </label>
              <div className="mt-3 pl-2 border-l-2 border-border/50">
                <label className={labelClass}>Proxy URL</label>
                <input type="text" disabled={!local.global.use_proxy} className={cn(inputClass, !local.global.use_proxy && "opacity-30 cursor-not-allowed")} value={local.global.proxy_url || ""} onChange={e => updateGlobal("proxy_url", e.target.value)} placeholder="http://127.0.0.1:8080" />
              </div>
            </div>
          </div>
        </div>

        {/* 2. Scan & Recon */}
        <div className={panelClass}>
          <h3 className={panelHeaderClass}>
            <Crosshair className="w-4 h-4 text-blue-400" />
            Reconnaissance Engine
          </h3>
          <div className="grid grid-cols-2 gap-5">
            <div>
              <label className={labelClass}>Scanner Wordlist</label>
              <select className={selectClass} value={local.scanner.wordlist_size} onChange={e => updateScanner("wordlist_size", e.target.value)}>
                <option value="small">Small (Fast)</option>
                <option value="medium">Medium (Balanced)</option>
                <option value="large">Large (Thorough)</option>
                <option value="massive">Massive (Intensive)</option>
              </select>
            </div>
            <div>
              <label className={labelClass}>Crawler Max Depth</label>
              <input type="number" className={inputClass} value={local.scanner.max_depth} onChange={e => updateScanner("max_depth", parseInt(e.target.value))} />
            </div>
            <div className="col-span-2">
              <label className={labelClass}>DNS Resolution Strategy</label>
              <select className={selectClass} value={local.investigator.dns_resolution} onChange={e => updateInvestigator("dns_resolution", e.target.value)}>
                <option value="system">System Default</option>
                <option value="custom">Custom Internal Servers</option>
                <option value="cloudflare">Cloudflare (1.1.1.1)</option>
              </select>
            </div>
            <div>
              <label className={labelClass}>Concurrent Hosts</label>
              <input type="number" className={inputClass} value={local.investigator.concurrent_hosts} onChange={e => updateInvestigator("concurrent_hosts", parseInt(e.target.value))} />
            </div>
          </div>
        </div>

        {/* 3. Discovery & Services */}
        <div className={panelClass}>
          <h3 className={panelHeaderClass}>
            <Network className="w-4 h-4 text-purple-400" />
            Discovery & Services
          </h3>
          <div className="space-y-4">
            <div>
              <label className={labelClass}>Port Scan Range</label>
              <input type="text" className={inputClass} value={local.services.port_scan_range} onChange={e => updateServices("port_scan_range", e.target.value)} placeholder="e.g. 1-1024,8080,8443" />
            </div>
            <div>
              <label className={labelClass}>API Discovery Wordlist</label>
              <select className={selectClass} value={local.api_discovery.wordlist_size} onChange={e => updateApi("wordlist_size", e.target.value)}>
                <option value="small">Small</option>
                <option value="medium">Medium</option>
                <option value="large">Large</option>
              </select>
            </div>
            <div className="pt-2 border-t border-border/30 grid grid-cols-1 gap-1">
              <label htmlFor="banner_grab" className={checkboxWrapperClass}>
                <input type="checkbox" id="banner_grab" checked={local.services.banner_grabbing} onChange={e => updateServices("banner_grabbing", e.target.checked)} className="rounded border-white/[0.2] bg-background/50 text-primary focus:ring-primary/50" />
                <span className="text-[11px] font-medium text-foreground/80 select-none">Enable TCP Banner Grabbing</span>
              </label>
              <label htmlFor="schema_parse" className={checkboxWrapperClass}>
                <input type="checkbox" id="schema_parse" checked={local.api_discovery.schema_parsing} onChange={e => updateApi("schema_parsing", e.target.checked)} className="rounded border-white/[0.2] bg-background/50 text-primary focus:ring-primary/50" />
                <span className="text-[11px] font-medium text-foreground/80 select-none">Parse OpenAPI/Swagger Schemas</span>
              </label>
            </div>
          </div>
        </div>

        {/* 4. Forms & Sessions */}
        <div className={panelClass}>
          <h3 className={panelHeaderClass}>
            <FileText className="w-4 h-4 text-emerald-400" />
            Forms & Session Handling
          </h3>
          <div className="grid grid-cols-2 gap-4">
            <div className="col-span-2">
              <label className={labelClass}>Fuzzing Intensity</label>
              <select className={selectClass} value={local.forms.fuzzing_intensity} onChange={e => updateForms("fuzzing_intensity", e.target.value)}>
                <option value="low">Low (Light touch)</option>
                <option value="medium">Medium (Standard payload set)</option>
                <option value="high">High (Extended payload set)</option>
                <option value="aggressive">Aggressive (Maximum coverage)</option>
              </select>
            </div>
            <div>
              <label className={labelClass}>Auto-Delete Data (Days)</label>
              <input type="number" className={inputClass} value={local.sessions.auto_delete_days} onChange={e => updateSessions("auto_delete_days", parseInt(e.target.value))} />
            </div>
            <div className="col-span-2 pt-2 border-t border-border/30 grid grid-cols-1 gap-1">
              <label htmlFor="avoid_waf" className={checkboxWrapperClass}>
                <input type="checkbox" id="avoid_waf" checked={local.forms.avoid_waf} onChange={e => updateForms("avoid_waf", e.target.checked)} className="rounded border-white/[0.2] bg-background/50 text-primary focus:ring-primary/50" />
                <span className="text-[11px] font-medium text-foreground/80 select-none">Enable WAF Evasion Techniques</span>
              </label>
              <label htmlFor="archive_art" className={checkboxWrapperClass}>
                <input type="checkbox" id="archive_art" checked={local.sessions.archive_artifacts} onChange={e => updateSessions("archive_artifacts", e.target.checked)} className="rounded border-white/[0.2] bg-background/50 text-primary focus:ring-primary/50" />
                <span className="text-[11px] font-medium text-foreground/80 select-none">Archive Session Artifacts</span>
              </label>
            </div>
          </div>
        </div>

        {/* 5. Exploit, Audit & Rules (Full Width) */}
        <div className={cn(panelClass, "col-span-1 lg:col-span-2 bg-gradient-to-br from-card/40 to-background/20")}>
          <h3 className={panelHeaderClass}>
            <ShieldAlert className="w-4 h-4 text-rose-400" />
            Exploitation & Auditing Engine
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <div className="space-y-4">
              <div>
                <label className={labelClass}>Exploit Execution Mode</label>
                <select className={cn(selectClass, local.exploit.mode === "authorized_active" && "border-red-500/50 text-red-400 focus:ring-red-500/30")} value={local.exploit.mode} onChange={e => {
                    const mode = e.target.value;
                    updateExploit("mode", mode);
                    if (mode === "authorized_active") {
                      updateExploit("sandbox_validation", true);
                      updateExploit("manual_approval_required", true);
                    }
                  }}>
                  <option value="safe_verification">Safe Verification (Read Only)</option>
                  <option value="authorized_active">Authorized Active Exploit</option>
                </select>
              </div>
              {local.exploit.mode === "authorized_active" && (
                <div className="bg-red-500/10 border border-red-500/30 rounded p-2.5 flex items-start gap-2 animate-in fade-in zoom-in-95">
                  <AlertTriangle className="w-3.5 h-3.5 text-red-400 shrink-0 mt-0.5" />
                  <p className="text-[10px] text-red-300 leading-tight">
                    <strong>Danger:</strong> Active exploitation may modify data or disrupt services. Use only with explicit authorization.
                  </p>
                </div>
              )}
            </div>
            
            <div className="space-y-4">
              <div>
                <label className={labelClass}>Audit Risk Coefficient</label>
                <input type="number" step="0.1" className={inputClass} value={local.audit.risk_coefficient} onChange={e => updateAudit("risk_coefficient", parseFloat(e.target.value))} />
                <p className="text-[9px] text-muted-foreground mt-1.5">Multiplier for CVSS and internal risk scores.</p>
              </div>
            </div>

            <div className="space-y-1 bg-black/20 rounded-lg p-3 border border-border/30">
              <label htmlFor="sandbox_val" className={checkboxWrapperClass}>
                <input type="checkbox" id="sandbox_val" checked={local.exploit.sandbox_validation} onChange={e => updateExploit("sandbox_validation", e.target.checked)} disabled={local.exploit.mode === "authorized_active"} className="rounded border-white/[0.2] bg-background/50 text-primary focus:ring-primary/50 disabled:opacity-50" />
                <span className="text-[11px] font-medium text-foreground/80 select-none">Enforce Sandbox Validation</span>
              </label>
              <label htmlFor="strict_mode" className={checkboxWrapperClass}>
                <input type="checkbox" id="strict_mode" checked={local.rules.strict_mode} onChange={e => updateRules("strict_mode", e.target.checked)} className="rounded border-white/[0.2] bg-background/50 text-primary focus:ring-primary/50" />
                <span className="text-[11px] font-medium text-foreground/80 select-none">Strict Rule Parsing Mode</span>
              </label>
              <label htmlFor="ignore_info" className={checkboxWrapperClass}>
                <input type="checkbox" id="ignore_info" checked={local.audit.ignore_informational} onChange={e => updateAudit("ignore_informational", e.target.checked)} className="rounded border-white/[0.2] bg-background/50 text-primary focus:ring-primary/50" />
                <span className="text-[11px] font-medium text-foreground/80 select-none">Ignore Informational Findings</span>
              </label>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function SettingsScreen() {
  const [activeTab, setActiveTab] = useState<Tab>("general");
  const { data: settingsProfiles = [], isLoading: spLoading } = useSettingsProfiles();
  const { data: scanProfiles = [], isLoading: scpLoading } = useScanProfiles();
  const updateMutation = useUpdateProfile();
  const [editingId, setEditingId] = useState<string | null>(null);

  const isLoading = spLoading || scpLoading;

  const handleSaveProfile = (id: string, updatedProfile: ApiSettingsProfile) => {
    updateMutation.mutate({ id, data: updatedProfile }, {
      onSuccess: () => {
        setEditingId(null);
      }
    });
  };

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 bg-background/50">
      {/* Top bar */}
      <div className="shrink-0 flex items-center gap-3 px-6 py-4 border-b border-border/60 bg-card/40 backdrop-blur-md relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-r from-primary/5 to-transparent pointer-events-none" />
        <div className="h-8 w-8 rounded-lg bg-primary/10 border border-primary/20 flex items-center justify-center">
          <Settings className="h-4 w-4 text-primary" />
        </div>
        <div>
          <h2 className="text-sm font-bold text-foreground tracking-wide">System Settings</h2>
          <p className="text-[10px] text-muted-foreground">Configure scan profiles and engine parameters</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="shrink-0 flex items-center gap-1 border-b border-border/40 bg-card/20 px-6 pt-2">
        {[
          { id: "general" as Tab, label: "Settings Profiles", icon: User },
          { id: "profiles" as Tab, label: "Scan Profiles", icon: Radar },
        ].map((tab) => (
          <button
            key={tab.id}
            className={cn(
              "flex items-center gap-2 px-4 py-2.5 text-[11px] font-semibold border-b-2 transition-all",
              activeTab === tab.id 
                ? "border-primary text-primary" 
                : "border-transparent text-muted-foreground hover:text-foreground hover:bg-white/[0.02]"
            )}
            onClick={() => setActiveTab(tab.id)}
          >
            <tab.icon className="h-3.5 w-3.5" /> 
            {tab.label}
            <span className={cn(
              "text-[9px] font-mono px-1.5 py-0.5 rounded-full ml-1",
              activeTab === tab.id ? "bg-primary/10 text-primary" : "bg-white/[0.05] text-muted-foreground"
            )}>
              {tab.id === "general" ? settingsProfiles.length : scanProfiles.length}
            </span>
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6 scroll-smooth">
        {isLoading ? (
          <div className="flex flex-col items-center justify-center h-full gap-3 text-muted-foreground">
            <Loader2 className="h-6 w-6 animate-spin text-primary" />
            <p className="text-[11px] font-medium tracking-widest uppercase">Loading Profiles...</p>
          </div>
        ) : (
          <div className="max-w-6xl mx-auto">
            {activeTab === "general" && (
              <div className="space-y-6">
                {settingsProfiles.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-16 border border-dashed border-border/50 rounded-xl bg-card/20">
                    <User className="h-8 w-8 text-muted-foreground/30 mb-3" />
                    <p className="text-xs text-muted-foreground">No settings profiles found.</p>
                  </div>
                ) : settingsProfiles.map((profile) => (
                  <div key={profile.id} className="bg-card/30 backdrop-blur-sm border border-border/60 rounded-xl overflow-hidden transition-all hover:border-border/80 hover:shadow-lg hover:shadow-black/20">
                    <div className="px-5 py-4 border-b border-border/40 flex items-center justify-between bg-gradient-to-r from-card/50 to-transparent">
                      <div>
                        <div className="flex items-center gap-3 mb-1">
                          <h3 className="text-[13px] font-bold text-foreground tracking-wide">{profile.name}</h3>
                          {profile.is_custom ? (
                            <span className="px-2 py-0.5 rounded text-[9px] font-semibold bg-blue-500/10 text-blue-400 border border-blue-500/20">Custom</span>
                          ) : (
                            <span className="px-2 py-0.5 rounded text-[9px] font-semibold bg-white/[0.05] text-muted-foreground border border-white/[0.1]">System Default</span>
                          )}
                        </div>
                        <p className="text-[11px] text-muted-foreground/70">{profile.description}</p>
                      </div>
                      <button
                        className={cn(
                          "px-4 py-2 text-[11px] font-semibold rounded-lg transition-all",
                          editingId === profile.id 
                            ? "bg-white/[0.1] text-foreground border border-white/[0.1]" 
                            : "bg-primary/10 text-primary border border-primary/20 hover:bg-primary/20 hover:scale-[1.02]"
                        )}
                        onClick={() => setEditingId(editingId === profile.id ? null : profile.id)}
                      >
                        {editingId === profile.id ? "Close Editor" : "Edit Configuration"}
                      </button>
                    </div>
                    
                    {/* Collapsible Editor Area */}
                    <div className={cn(
                      "transition-all duration-300 ease-in-out overflow-hidden",
                      editingId === profile.id ? "max-h-[3000px] opacity-100" : "max-h-0 opacity-0"
                    )}>
                      <div className="p-5 bg-background/50 border-t border-border/30">
                        {updateMutation.isSuccess && editingId === profile.id && (
                          <div className="mb-6 bg-emerald-500/10 border border-emerald-500/20 rounded-lg p-3 flex items-center gap-3 animate-in slide-in-from-top-2">
                            <div className="h-6 w-6 rounded-full bg-emerald-500/20 flex items-center justify-center shrink-0">
                              <CheckCircle className="w-3.5 h-3.5 text-emerald-400" />
                            </div>
                            <span className="text-xs text-emerald-300 font-medium">Profile configuration saved successfully to the backend.</span>
                          </div>
                        )}
                        <ProfileEditor 
                          profile={profile} 
                          onSave={(p) => handleSaveProfile(profile.id, p)} 
                          isSaving={updateMutation.isPending} 
                        />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* Scan Profiles (Similar structure) */}
            {activeTab === "profiles" && (
              <div className="space-y-6">
                {scanProfiles.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-16 border border-dashed border-border/50 rounded-xl bg-card/20">
                    <Radar className="h-8 w-8 text-muted-foreground/30 mb-3" />
                    <p className="text-xs text-muted-foreground">No scan profiles found.</p>
                  </div>
                ) : scanProfiles.map((profile) => (
                  <div key={profile.id} className="bg-card/30 backdrop-blur-sm border border-border/60 rounded-xl overflow-hidden transition-all hover:border-border/80">
                    <div className="px-5 py-4 border-b border-border/40 flex items-center justify-between bg-gradient-to-r from-card/50 to-transparent">
                      <div>
                        <h3 className="text-[13px] font-bold text-foreground tracking-wide mb-1">{profile.name}</h3>
                        <p className="text-[10px] font-mono text-muted-foreground/50">ID: {profile.id}</p>
                      </div>
                      <button
                        className={cn(
                          "px-4 py-2 text-[11px] font-semibold rounded-lg transition-all",
                          editingId === profile.id 
                            ? "bg-white/[0.1] text-foreground border border-white/[0.1]" 
                            : "bg-primary/10 text-primary border border-primary/20 hover:bg-primary/20 hover:scale-[1.02]"
                        )}
                        onClick={() => setEditingId(editingId === profile.id ? null : profile.id)}
                      >
                        {editingId === profile.id ? "Close Editor" : "Edit Configuration"}
                      </button>
                    </div>
                    <div className={cn(
                      "transition-all duration-300 ease-in-out overflow-hidden",
                      editingId === profile.id ? "max-h-[3000px] opacity-100" : "max-h-0 opacity-0"
                    )}>
                      <div className="p-5 bg-background/50 border-t border-border/30">
                        {updateMutation.isSuccess && editingId === profile.id && (
                          <div className="mb-6 bg-emerald-500/10 border border-emerald-500/20 rounded-lg p-3 flex items-center gap-3">
                            <CheckCircle className="w-4 h-4 text-emerald-400" />
                            <span className="text-xs text-emerald-300 font-medium">Profile saved successfully.</span>
                          </div>
                        )}
                        <ProfileEditor 
                          profile={profile} 
                          onSave={(p) => handleSaveProfile(profile.id, p)} 
                          isSaving={updateMutation.isPending} 
                        />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
