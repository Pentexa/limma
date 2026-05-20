"use client";

import { Button } from "@/shared/ui/button";
import { Pause } from "lucide-react";
import { useState } from "react";
import { pauseScan } from "../api/pause-scan";
import { toast } from "sonner";

interface PauseScanButtonProps { scanId: string; disabled?: boolean; onPaused?: () => void; }

export function PauseScanButton({ scanId, disabled, onPaused }: PauseScanButtonProps) {
  const [isLoading, setIsLoading] = useState(false);

  async function handlePause() {
    setIsLoading(true);
    try {
      await pauseScan(scanId);
      onPaused?.();
    } catch (err) { toast.error(err instanceof Error ? err.message : "Pause failed"); } finally { setIsLoading(false); }
  }

  return (
    <Button onClick={handlePause} disabled={disabled || isLoading} variant="outline" size="sm" className="gap-2 disabled:opacity-70">
      <Pause className="h-3.5 w-3.5" />
      {isLoading ? "Pausing…" : "Pause"}
    </Button>
  );
}
