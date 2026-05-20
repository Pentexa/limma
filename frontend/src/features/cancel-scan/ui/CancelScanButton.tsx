"use client";

import { Button } from "@/shared/ui/button";
import { X } from "lucide-react";
import { useState } from "react";
import { cancelScan } from "../api/cancel-scan";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

interface CancelScanButtonProps { scanId: string; disabled?: boolean; onCancelled?: () => void; }

export function CancelScanButton({ scanId, disabled, onCancelled }: CancelScanButtonProps) {
  const [isLoading, setIsLoading] = useState(false);
  const queryClient = useQueryClient();

  async function handleCancel() {
    setIsLoading(true);
    try { 
      await cancelScan(scanId); 
      onCancelled?.(); 
      queryClient.invalidateQueries({ queryKey: ["scans"] });
    } catch (err) { toast.error(err instanceof Error ? err.message : "Cancel failed"); } finally { setIsLoading(false); }
  }

  return (
    <Button onClick={handleCancel} disabled={disabled || isLoading} variant="destructive" size="sm" className="gap-2 disabled:opacity-70">
      <X className="h-3.5 w-3.5" />
      {isLoading ? "Cancelling…" : "Cancel"}
    </Button>
  );
}
