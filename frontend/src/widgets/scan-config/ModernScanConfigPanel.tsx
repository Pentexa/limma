"use client";

import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogDescription } from "@/shared/ui/dialog";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { Play, Settings2, ShieldAlert } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { startScan } from "@/features/start-scan/api/start-scan";
import { useStreamStore } from "@/features/stream-scan-events/model/stream-store";
import { startScanStream } from "@/features/stream-scan-events/model/scan-stream-manager";
import { DETECTOR_META } from "@/entities/finding/model/types";
import type { ActiveVulnType } from "@/shared/types/api";

const DETECTOR_TO_VULN_TYPES: Record<string, string[]> = {
  xss: ["reflected_xss", "stored_xss", "dom_xss"],
  sqli: ["sql_injection_error", "sql_injection_union", "sql_injection_blind_time", "sql_injection_blind_boolean"],
  cmdi: ["command_injection", "command_injection_blind"],
  lfi: ["local_file_inclusion"],
  rfi: ["remote_file_inclusion"],
  traversal: ["path_traversal"],
  ssrf: ["server_side_request_forgery"],
  xxe: ["xml_external_entity"],
  redirect: ["open_redirect"],
  jwt: ["jwt_none_algorithm", "jwt_weak_secret"],
  nosql: ["no_sql_injection"],
  ssti: ["server_side_template_injection"],
  graphql: ["graphql_introspection_enabled", "graphql_abuse"],
};

export function ModernScanConfigPanel({ targetUrl, profileId, disabled }: { targetUrl: string, profileId?: string, disabled?: boolean }) {
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  // Form State
  const [scanMode, setScanMode] = useState<"fast" | "modern_spa_api" | "deep_verification">("fast");
  const [enableHeadlessBrowser, setEnableHeadlessBrowser] = useState(false);
  const [maxBrowserTabs, setMaxBrowserTabs] = useState(2);
  const [bearerToken, setBearerToken] = useState("");
  const [cookie, setCookie] = useState("");
  const [customHeaders, setCustomHeaders] = useState("");
  const [basicAuthUser, setBasicAuthUser] = useState("");
  const [basicAuthPass, setBasicAuthPass] = useState("");
  const [enableJsonFuzzing, setEnableJsonFuzzing] = useState(true);
  const [enableXssVerification, setEnableXssVerification] = useState(true);
  const [allowDestructiveMethods, setAllowDestructiveMethods] = useState(false);
  const [l3ConsentAccepted, setL3ConsentAccepted] = useState(false);

  const allDetectors = Object.keys(DETECTOR_TO_VULN_TYPES);
  const [selectedDetectors, setSelectedDetectors] = useState<Set<string>>(new Set(allDetectors));

  const queryClient = useQueryClient();

  async function handleStart() {
    if (!targetUrl) return;

    if (allowDestructiveMethods && !l3ConsentAccepted) {
      alert("You must accept L3 Consent to use destructive methods.");
      return;
    }

    setIsLoading(true);

    useStreamStore.getState().setScanStarting(targetUrl);
    startScanStream(targetUrl);

    try {
      const result = await startScan({
        target_url: targetUrl,
        profile_id: profileId,
        scan_mode: scanMode,
        enable_headless_browser: enableHeadlessBrowser,
        max_browser_tabs: Math.min(3, Math.max(1, maxBrowserTabs)),
        bearer_token: bearerToken || undefined,
        cookie: cookie || undefined,
        custom_headers: customHeaders || undefined,
        basic_auth_user: basicAuthUser || undefined,
        basic_auth_pass: basicAuthPass || undefined,
        enable_json_fuzzing: enableJsonFuzzing,
        enable_xss_verification: enableXssVerification,
        allow_destructive_methods: allowDestructiveMethods,
        l3_consent_accepted: l3ConsentAccepted,
        vuln_types: Array.from(selectedDetectors).flatMap(d => DETECTOR_TO_VULN_TYPES[d]) as any[],
      });

      useStreamStore.getState().setScanRunning(result.scan_id);
      queryClient.invalidateQueries({ queryKey: ["scans"] });
      queryClient.invalidateQueries({ queryKey: ["findings"] });
      setIsOpen(false);
    } catch (err) {
      useStreamStore.getState().setScanIdle();
      const msg = err instanceof Error ? err.message : "Failed to start scan";
      alert(`Failed to start scan: ${msg}`);
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button disabled={disabled || !targetUrl} className="h-[28px] gap-2 px-3 text-[11px]" size="sm">
          <Settings2 className="h-3.5 w-3.5" /> Configure & Start
        </Button>
      </DialogTrigger>
      <DialogContent className="w-[95vw] max-w-5xl sm:max-w-5xl md:max-w-5xl lg:max-w-5xl max-h-[90vh] flex flex-col bg-[#050505] text-foreground border-white/10 shadow-2xl overflow-hidden rounded-xl">
        <DialogHeader className="shrink-0 pb-5 border-b border-white/10 px-2 pt-2">
          <DialogTitle className="flex items-center gap-2 text-2xl font-bold tracking-wide">
            <Settings2 className="h-6 w-6 text-primary" /> Active Scan Configuration
          </DialogTitle>
          <DialogDescription className="text-sm text-muted-foreground mt-2">Configure advanced parameters before initiating the scan on <span className="font-mono text-primary/80 px-2 py-1 bg-primary/10 rounded">{targetUrl}</span>.</DialogDescription>
        </DialogHeader>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 py-4 overflow-y-auto px-2 flex-1 scrollbar-thin scrollbar-thumb-white/10">
          <div className="space-y-6">
            <div className="space-y-2.5">
              <label className="text-sm font-medium text-foreground/90">Scan Mode</label>
              <select 
                value={scanMode} 
                onChange={e => setScanMode(e.target.value as any)}
                className="w-full h-10 px-3 text-sm rounded-md bg-white/5 border border-white/10 outline-none focus:border-primary/50 focus:bg-white/10 transition-colors"
              >
                <option value="fast" className="bg-[#050505]">Fast (Classic Parameter Based)</option>
                <option value="modern_spa_api" className="bg-[#050505]">Modern SPA / API (Crawler & Replayer)</option>
                <option value="deep_verification" className="bg-[#050505]">Deep Verification</option>
              </select>
            </div>

            <div className="space-y-3">
              <label className="flex items-center gap-3 text-sm font-medium text-foreground/90 cursor-pointer p-3 rounded-md bg-white/5 border border-white/5 hover:bg-white/10 transition-colors">
                <input 
                  type="checkbox" 
                  checked={enableHeadlessBrowser}
                  onChange={e => setEnableHeadlessBrowser(e.target.checked)}
                  className="rounded border-white/20 bg-background/50 text-primary focus:ring-0 h-4 w-4"
                />
                Enable Headless Browser Crawler
              </label>
            </div>

            {enableHeadlessBrowser && (
              <div className="space-y-2 pl-8 border-l-2 border-white/10 ml-2">
                <label className="text-sm font-medium text-muted-foreground">Max Browser Tabs (Max 3)</label>
                <Input 
                  type="number" 
                  min={1} 
                  max={3} 
                  value={maxBrowserTabs} 
                  onChange={e => setMaxBrowserTabs(parseInt(e.target.value) || 2)}
                  className="h-10 text-sm bg-white/5 border-white/10 focus:bg-white/10"
                />
              </div>
            )}

            <div className="space-y-5 p-5 border border-white/10 rounded-xl bg-[#0a0a0c] shadow-inner">
              <h3 className="text-base font-semibold text-primary">Authentication Settings</h3>
              <div className="space-y-2">
                <label className="text-sm font-medium text-foreground/90">Bearer Token</label>
                <Input 
                  placeholder="eyJhb..." 
                  value={bearerToken} 
                  onChange={e => setBearerToken(e.target.value)} 
                  className="h-10 text-sm bg-white/5 border-white/10 focus:bg-white/10 font-mono" 
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium text-foreground/90">Cookie</label>
                <Input 
                  placeholder="session_id=12345;" 
                  value={cookie} 
                  onChange={e => setCookie(e.target.value)} 
                  className="h-10 text-sm bg-white/5 border-white/10 focus:bg-white/10 font-mono" 
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium text-foreground/90">Custom Headers</label>
                <textarea 
                  placeholder="X-API-Key: my-key\nX-Custom-Header: value"
                  value={customHeaders}
                  onChange={e => setCustomHeaders(e.target.value)}
                  className="w-full h-20 p-3 text-sm font-mono rounded-md bg-white/5 border border-white/10 outline-none focus:border-primary/50 focus:bg-white/10 resize-none transition-colors"
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium text-foreground/90">Basic Auth User</label>
                  <Input 
                    placeholder="admin" 
                    value={basicAuthUser} 
                    onChange={e => setBasicAuthUser(e.target.value)} 
                    className="h-10 text-sm bg-white/5 border-white/10 focus:bg-white/10 font-mono" 
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium text-foreground/90">Basic Auth Password</label>
                  <Input 
                    type="password"
                    placeholder="password" 
                    value={basicAuthPass} 
                    onChange={e => setBasicAuthPass(e.target.value)} 
                    className="h-10 text-sm bg-white/5 border-white/10 focus:bg-white/10 font-mono" 
                  />
                </div>
              </div>
            </div>
          </div>

          <div className="space-y-6">
            <div className="space-y-3">
              <label className="flex items-center gap-3 text-sm font-medium text-foreground/90 cursor-pointer p-3 rounded-md bg-white/5 border border-white/5 hover:bg-white/10 transition-colors">
                <input 
                  type="checkbox" 
                  checked={enableJsonFuzzing}
                  onChange={e => setEnableJsonFuzzing(e.target.checked)}
                  className="rounded border-white/20 bg-background/50 text-primary focus:ring-0 h-4 w-4"
                />
                Enable JSON Body Fuzzing
              </label>
            </div>
            <div className="space-y-3">
              <label className="flex items-center gap-3 text-sm font-medium text-foreground/90 cursor-pointer p-3 rounded-md bg-white/5 border border-white/5 hover:bg-white/10 transition-colors">
                <input 
                  type="checkbox" 
                  checked={enableXssVerification}
                  onChange={e => setEnableXssVerification(e.target.checked)}
                  className="rounded border-white/20 bg-background/50 text-primary focus:ring-0 h-4 w-4"
                />
                Advanced XSS Verification (Headless Validation)
              </label>
            </div>

            <div className="space-y-3">
              <div className="flex items-center justify-between px-1">
                <label className="text-sm font-medium text-foreground/90">Active Detectors</label>
                <button 
                  className="text-xs text-primary hover:text-primary/80 transition-colors"
                  onClick={() => setSelectedDetectors(selectedDetectors.size === allDetectors.length ? new Set() : new Set(allDetectors))}
                >
                  {selectedDetectors.size === allDetectors.length ? "Deselect All" : "Select All"}
                </button>
              </div>
              <div className="grid grid-cols-2 gap-3 p-4 border border-white/10 rounded-xl bg-[#0a0a0c] shadow-inner max-h-60 overflow-y-auto scrollbar-thin scrollbar-thumb-white/10">
                {allDetectors.map(key => {
                  const meta = DETECTOR_META[key];
                  return (
                    <label key={key} className="flex items-center gap-3 text-sm text-muted-foreground hover:text-foreground cursor-pointer p-2 rounded hover:bg-white/5 transition-colors">
                      <input 
                        type="checkbox" 
                        checked={selectedDetectors.has(key)}
                        onChange={e => {
                          const next = new Set(selectedDetectors);
                          if (e.target.checked) next.add(key);
                          else next.delete(key);
                          setSelectedDetectors(next);
                        }}
                        className="rounded border-white/20 bg-background/50 text-primary focus:ring-0 h-4 w-4"
                      />
                      {meta?.name || key}
                    </label>
                  )
                })}
              </div>
            </div>

            <div className="pt-4 border-t border-white/10 mt-6 space-y-4">
              <h4 className="text-sm font-semibold text-red-400 flex items-center gap-2"><ShieldAlert className="h-4 w-4"/> Destructive Actions</h4>
              
              <label className="flex items-center gap-3 text-sm font-medium text-foreground/90 cursor-pointer p-3 rounded-md bg-red-500/5 border border-red-500/10 hover:bg-red-500/10 transition-colors">
                <input 
                  type="checkbox" 
                  checked={allowDestructiveMethods}
                  onChange={e => {
                    setAllowDestructiveMethods(e.target.checked);
                    if (!e.target.checked) setL3ConsentAccepted(false);
                  }}
                  className="rounded border-red-500/30 bg-background/50 text-red-500 focus:ring-0 h-4 w-4"
                />
                Allow PUT / PATCH / DELETE
              </label>

              {allowDestructiveMethods && (
                <label className="flex items-start gap-3 text-sm font-medium text-red-400/90 cursor-pointer bg-red-500/10 p-4 rounded-lg border border-red-500/30 shadow-inner">
                  <input 
                    type="checkbox" 
                    checked={l3ConsentAccepted}
                    onChange={e => setL3ConsentAccepted(e.target.checked)}
                    className="rounded border-red-500/40 bg-background/50 text-red-500 focus:ring-0 mt-0.5 h-4 w-4 shrink-0"
                  />
                  <span className="leading-relaxed">I consent (L3) to using destructive methods on the target, understanding they may modify or delete production data.</span>
                </label>
              )}
            </div>
          </div>
        </div>

        <div className="flex justify-between items-center pt-5 mt-2 border-t border-white/10 shrink-0 px-2 pb-2">
          <div className="text-xs text-muted-foreground max-w-[500px] leading-relaxed">
            <strong className="text-foreground/80">Summary:</strong> Will run in <span className="text-primary font-medium">{scanMode}</span> mode. 
            {enableHeadlessBrowser ? ` Crawler enabled (max ${maxBrowserTabs} tabs).` : ""}
            {allowDestructiveMethods ? <span className="text-red-400 font-medium"> Destructive methods ALLOWED.</span> : ""}
          </div>
          <div className="flex gap-3">
            <Button variant="ghost" onClick={() => setIsOpen(false)} className="h-10 text-sm px-6">Cancel</Button>
            <Button onClick={handleStart} disabled={isLoading || (allowDestructiveMethods && !l3ConsentAccepted)} className="h-10 text-sm gap-2 min-w-[140px] shadow-lg shadow-primary/20 hover:shadow-primary/40 transition-all">
              <Play className="h-4 w-4" /> {isLoading ? "Starting..." : "Start Scan"}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
