"use client";

import { Play } from "lucide-react";
import { useState } from "react";
import { startScan } from "../api/start-scan";
import { useQueryClient } from "@tanstack/react-query";
import { useStreamStore } from "@/features/stream-scan-events/model/stream-store";
import { startScanStream } from "@/features/stream-scan-events/model/scan-stream-manager";

interface StartScanButtonProps {
  targetUrl: string;
  profileId?: string;
  disabled?: boolean;
  onStarted?: (scanId: string) => void;
  onError?: (error: string) => void;
}

export function StartScanButton({ targetUrl, profileId, disabled, onStarted, onError }: StartScanButtonProps) {
  const [isLoading, setIsLoading] = useState(false);
  const queryClient = useQueryClient();

  async function handleStart() {
    if (!targetUrl) return;
    setIsLoading(true);

    // 1. IMMEDIATELY update local store — UI reacts instantly
    useStreamStore.getState().setScanStarting(targetUrl);

    // 2. Start SSE stream so events flow right away
    startScanStream(targetUrl);

    try {
      // 3. Fire the actual API call
      const result = await startScan({ target_url: targetUrl, profile_id: profileId });

      // 4. Promote local state to running
      useStreamStore.getState().setScanRunning(result.scan_id);

      onStarted?.(result.scan_id);

      // 5. Refresh scans list to sync polling data
      queryClient.invalidateQueries({ queryKey: ["scans"] });
      queryClient.invalidateQueries({ queryKey: ["findings"] });
    } catch (err) {
      // Revert local state on error
      useStreamStore.getState().setScanIdle();
      const msg = err instanceof Error ? err.message : "Failed to start scan";
      onError?.(msg);
      if (!onError) alert(`Failed to start scan: ${msg}`);
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <button
      onClick={handleStart}
      disabled={disabled || isLoading || !targetUrl}
      className="flex items-center gap-1.5 px-3 py-1.5 bg-primary text-primary-foreground text-[12px] font-medium rounded hover:bg-primary/90 disabled:opacity-40 disabled:pointer-events-none transition-colors"
    >
      <Play className="h-3 w-3" />
      {isLoading ? "Starting…" : "Start Scan"}
    </button>
  );
}
