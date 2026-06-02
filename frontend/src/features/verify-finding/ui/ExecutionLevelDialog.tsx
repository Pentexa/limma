import React, { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/shared/ui/dialog";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import type { SafetyLevel } from "@/features/blind-scan/api/blind-scan-api";
import { AlertTriangle, ShieldCheck, ShieldAlert, Code } from "lucide-react";

interface ExecutionLevelDialogProps {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: (level: SafetyLevel, targetUrl: string) => void;
  isPending?: boolean;
}

export function ExecutionLevelDialog({ isOpen, onOpenChange, onConfirm, isPending }: ExecutionLevelDialogProps) {
  const [level, setLevel] = useState<SafetyLevel>("l1_safe_read_only");
  const [targetUrl, setTargetUrl] = useState("");
  const [consentChecked, setConsentChecked] = useState(false);

  // Validate L3
  const isL3 = level === "l3_active_with_consent";
  const canConfirm = !isL3 || (consentChecked && targetUrl.trim().length > 3);

  const handleConfirm = () => {
    if (canConfirm) {
      onConfirm(level, targetUrl);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <ShieldAlert className="w-5 h-5 text-indigo-500" />
            Execution Safety Level
          </DialogTitle>
          <DialogDescription>
            Select how you want to execute this exploit. Higher levels carry more risk.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* Level Selection */}
          <div className="grid gap-3">
            <div 
              className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${level === "l1_safe_read_only" ? "border-green-500 bg-green-500/10" : "border-slate-800 hover:border-slate-700"}`}
              onClick={() => setLevel("l1_safe_read_only")}
            >
              <Code className="w-5 h-5 text-green-500 mt-0.5" />
              <div>
                <h4 className="text-sm font-semibold text-slate-200">L1 - Safe Read-Only</h4>
                <p className="text-xs text-slate-400">Syntax check and simulation only. No payload is sent.</p>
              </div>
            </div>

            <div 
              className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${level === "l2_verified_sandbox" ? "border-blue-500 bg-blue-500/10" : "border-slate-800 hover:border-slate-700"}`}
              onClick={() => setLevel("l2_verified_sandbox")}
            >
              <ShieldCheck className="w-5 h-5 text-blue-500 mt-0.5" />
              <div>
                <h4 className="text-sm font-semibold text-slate-200">L2 - Verified Sandbox</h4>
                <p className="text-xs text-slate-400">Executes inside an isolated Docker container safely.</p>
              </div>
            </div>

            <div 
              className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${level === "l3_active_with_consent" ? "border-red-500 bg-red-500/10" : "border-slate-800 hover:border-slate-700"}`}
              onClick={() => setLevel("l3_active_with_consent")}
            >
              <AlertTriangle className="w-5 h-5 text-red-500 mt-0.5" />
              <div>
                <h4 className="text-sm font-semibold text-red-400">L3 - Active Exploitation</h4>
                <p className="text-xs text-slate-400">Sends real exploit payloads to the target. Requires explicit consent.</p>
              </div>
            </div>
          </div>

          {/* L3 Extra Fields */}
          {isL3 && (
            <div className="mt-2 space-y-4 p-4 border border-red-500/30 bg-red-500/5 rounded-lg animate-in fade-in slide-in-from-top-2">
              <div className="space-y-2">
                <label className="text-sm font-medium text-red-400">
                  Target Domain / URL (Required)
                </label>
                <Input 
                  placeholder="e.g. https://target.com"
                  value={targetUrl}
                  onChange={(e) => setTargetUrl(e.target.value)}
                  className="border-red-500/50 focus-visible:ring-red-500"
                />
                <p className="text-[11px] text-slate-400">Type the exact target URL to prevent accidental attacks.</p>
              </div>

              <label className="flex items-start gap-2 cursor-pointer">
                <input 
                  type="checkbox" 
                  checked={consentChecked}
                  onChange={(e) => setConsentChecked(e.target.checked)}
                  className="mt-1 bg-slate-900 border-red-500 text-red-500 focus:ring-red-500 rounded"
                />
                <span className="text-xs text-slate-300">
                  I assume full responsibility and grant active exploitation consent for this target.
                </span>
              </label>
            </div>
          )}
          
          {/* General Target URL for L1/L2 if needed. Actually we always need targetUrl for the API now */}
          {!isL3 && (
            <div className="space-y-2">
               <label className="text-sm font-medium text-slate-300">Target URL</label>
               <Input 
                  placeholder="https://target.com"
                  value={targetUrl}
                  onChange={(e) => setTargetUrl(e.target.value)}
                />
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isPending}>
            Cancel
          </Button>
          <Button 
            variant={isL3 ? "destructive" : "default"} 
            onClick={handleConfirm}
            disabled={!canConfirm || isPending || !targetUrl}
          >
            {isPending ? "Executing..." : isL3 ? "Execute Attack" : "Verify Safe"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
