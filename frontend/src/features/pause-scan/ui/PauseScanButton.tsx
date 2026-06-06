"use client";

import { Button } from "@/shared/ui/button";
import { Pause, Play } from "lucide-react";
import { useState } from "react";
import { pauseScan, resumeScan } from "../api/pause-scan";
import { toast } from "sonner";

interface PauseScanButtonProps { scanId: string; disabled?: boolean; isPausedProp?: boolean; onPaused?: () => void; onResumed?: () => void; }

export function PauseScanButton({ scanId, disabled, isPausedProp = false, onPaused, onResumed }: PauseScanButtonProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [isPaused, setIsPaused] = useState(isPausedProp);

  async function handleToggle() {
    setIsLoading(true);
    try {
      if (isPaused) {
        await resumeScan(scanId);
        setIsPaused(false);
        onResumed?.();
      } else {
        await pauseScan(scanId);
        setIsPaused(true);
        onPaused?.();
      }
    } catch (err) { toast.error(err instanceof Error ? err.message : "Action failed"); } finally { setIsLoading(false); }
  }

  return (
    <Button onClick={handleToggle} disabled={disabled || isLoading} variant="outline" size="sm" className="gap-2 disabled:opacity-70">
      {isPaused ? <Play className="h-3.5 w-3.5" /> : <Pause className="h-3.5 w-3.5" />}
      {isLoading ? "Wait…" : (isPaused ? "Resume" : "Pause")}
    </Button>
  );
}
