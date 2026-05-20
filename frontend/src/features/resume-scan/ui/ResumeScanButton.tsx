"use client";

import { Button } from "@/shared/ui/button";
import { Play } from "lucide-react";
import { useState } from "react";
import { resumeScan } from "../api/resume-scan";
import { toast } from "sonner";

interface ResumeScanButtonProps { scanId: string; disabled?: boolean; onResumed?: () => void; }

export function ResumeScanButton({ scanId, disabled, onResumed }: ResumeScanButtonProps) {
  const [isLoading, setIsLoading] = useState(false);

  async function handleResume() {
    setIsLoading(true);
    try { await resumeScan(scanId); onResumed?.(); } catch (err) { toast.error(err instanceof Error ? err.message : "Resume failed"); } finally { setIsLoading(false); }
  }

  return (
    <Button onClick={handleResume} disabled={disabled || isLoading} variant="outline" size="sm" className="gap-2">
      <Play className="h-3.5 w-3.5" />
      {isLoading ? "Resuming…" : "Resume"}
    </Button>
  );
}
