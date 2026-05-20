"use client";

import { Button } from "@/shared/ui/button";
import { ShieldCheck } from "lucide-react";
import { useState } from "react";
import { verifyFinding } from "../api/verify-finding";
import { toast } from "sonner";
import type { FindingId } from "@/shared/types/common";

interface VerifyFindingButtonProps { findingId: FindingId; disabled?: boolean; onVerified?: () => void; }

export function VerifyFindingButton({ findingId, disabled, onVerified }: VerifyFindingButtonProps) {
  const [isLoading, setIsLoading] = useState(false);
  async function handle() {
    setIsLoading(true);
    try { await verifyFinding(findingId); onVerified?.(); } catch (err) { toast.error(err instanceof Error ? err.message : "Verification failed"); } finally { setIsLoading(false); }
  }
  return (
    <Button onClick={handle} disabled={disabled || isLoading} variant="outline" size="sm" aria-label="Verify finding" className="gap-2 text-verified border-verified/30 hover:bg-verified/10">
      <ShieldCheck className="h-3.5 w-3.5" />
      {isLoading ? "Verifying…" : "Verify"}
    </Button>
  );
}
