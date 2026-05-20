"use client";

import { StartScanButton } from "@/features/start-scan/ui/StartScanButton";
import { PauseScanButton } from "@/features/pause-scan/ui/PauseScanButton";
import { CancelScanButton } from "@/features/cancel-scan/ui/CancelScanButton";
import type { ScanStatus } from "@/entities/scan/model/types";
import { useStreamStore } from "@/features/stream-scan-events/model/stream-store";
import { useSettingsProfiles } from "@/features/update-settings/model/use-settings";
import { useState } from "react";
import { Settings2 } from "lucide-react";

interface ScanControlsProps {
  targetUrl: string;
  scanId: string | null;
  scanStatus: ScanStatus;
}

export function ScanControls({ targetUrl, scanId, scanStatus }: ScanControlsProps) {
  const localScanState = useStreamStore((s) => s.localScanState);
  const { data: profiles = [] } = useSettingsProfiles();
  const [selectedProfileId, setSelectedProfileId] = useState<string>("default");

  // Use local state for instant feedback — covers the gap before polling catches up
  const isRunning =
    localScanState === "starting" ||
    localScanState === "running" ||
    scanStatus === "running" ||
    scanStatus === "starting" ||
    scanStatus === "pending";

  return (
    <div className="flex items-center gap-2">
      {/* Idle / Completed → Show Profile Selector + Start Scan */}
      {!isRunning && (
        <div className="flex items-center gap-2">
          {/* Profile Selector */}
          <div className="flex items-center gap-1.5 px-2.5 h-[28px] bg-background/50 border border-border/80 rounded shadow-inner transition-colors hover:border-primary/40 focus-within:border-primary/50">
            <Settings2 className="w-3.5 h-3.5 text-muted-foreground/80" />
            <select
              value={selectedProfileId}
              onChange={(e) => setSelectedProfileId(e.target.value)}
              className="bg-transparent text-foreground border-none focus:outline-none focus:ring-0 text-[11px] font-medium cursor-pointer w-[110px]"
            >
              <option value="default" className="bg-background text-foreground">Default Profile</option>
              {profiles.filter(p => p.id !== "default").map((p) => (
                <option key={p.id} value={p.id} className="bg-background text-foreground">
                  {p.name}
                </option>
              ))}
            </select>
          </div>

          <StartScanButton targetUrl={targetUrl} profileId={selectedProfileId} />
        </div>
      )}

      {/* Scanning → Show Pause & Cancel (disabled until scanId arrives) */}
      {isRunning && (
        <>
          <PauseScanButton scanId={scanId ?? ""} disabled={!scanId} />
          <CancelScanButton scanId={scanId ?? ""} disabled={!scanId} />
        </>
      )}
    </div>
  );
}
