"use client";

import { ModernScanConfigPanel } from "@/widgets/scan-config/ModernScanConfigPanel";
import { PauseScanButton } from "@/features/pause-scan/ui/PauseScanButton";
import { CancelScanButton } from "@/features/cancel-scan/ui/CancelScanButton";
import type { ScanStatus } from "@/entities/scan/model/types";
import { useStreamStore } from "@/features/stream-scan-events/model/stream-store";
import { useSettingsProfiles } from "@/features/update-settings/model/use-settings";
import { useState } from "react";
import { Settings2 } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/ui/select";

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
          <Select value={selectedProfileId} onValueChange={setSelectedProfileId}>
            <SelectTrigger className="h-[28px] w-[140px] px-2.5 text-[11px] font-medium bg-background/50 border-border/80 shadow-inner hover:border-primary/40 focus:ring-0 focus-visible:ring-0 transition-colors justify-start gap-1.5 [&>span]:min-w-0 [&>svg:last-child]:ml-auto">
              <Settings2 className="w-3.5 h-3.5 text-muted-foreground/80 shrink-0" />
              <SelectValue placeholder="Select profile" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="default" className="text-[11px]">Default Profile</SelectItem>
              {profiles.filter(p => p.id !== "default").map((p) => (
                <SelectItem key={p.id} value={p.id} className="text-[11px]">
                  {p.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <ModernScanConfigPanel targetUrl={targetUrl} profileId={selectedProfileId} />
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
