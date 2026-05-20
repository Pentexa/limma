"use client";

import { TargetInput } from "./TargetInput";
import { ScanControls } from "./ScanControls";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useScans } from "@/entities/scan/model/use-scans";
import { useGlobalFindings } from "@/entities/finding/model/use-findings";
import { useStreamStore } from "@/features/stream-scan-events/model/stream-store";
import { cn } from "@/shared/lib/utils";
import { Loader2, CheckCircle2 } from "lucide-react";

const urlSchema = z.object({
  targetUrl: z.string().refine((val) => {
    if (!val) return true;
    try {
      const url = val.startsWith("http://") || val.startsWith("https://") ? val : `https://${val}`;
      new URL(url);
      return true;
    } catch {
      return false;
    }
  }, "Please enter a valid URL").or(z.string().length(0)),
});

type TopBarForm = z.infer<typeof urlSchema>;

function normalizeUrl(url: string | undefined | null): string {
  if (!url) return "";
  const trimmed = url.trim();
  if (!trimmed.startsWith("http://") && !trimmed.startsWith("https://")) {
    return `https://${trimmed}`;
  }
  return trimmed;
}

export function TopBar() {
  const { register, watch, formState: { errors } } = useForm<TopBarForm>({
    resolver: zodResolver(urlSchema),
    defaultValues: { targetUrl: "" },
    mode: "onChange"
  });
  
  const targetUrl = watch("targetUrl");
  const { data: scans = [] } = useScans();
  // isFetching = true during ANY fetch (initial + refetch after invalidation).
  // isLoading is only true on the very first fetch — useless for refetch detection.
  const { isFetching: findingsStillFetching } = useGlobalFindings();

  // Local scan state (instant, from store) takes priority over polled state
  const localScanState = useStreamStore((s) => s.localScanState);
  const localScanTarget = useStreamStore((s) => s.localScanTarget);
  const localScanId = useStreamStore((s) => s.localScanId);

  const activeScan = scans.find((s) => s.status === "running") ?? scans[0];
  const polledStatus = activeScan?.status ?? "idle";

  // Backend may say "completed" but findings haven't arrived yet.
  // Keep showing "Scanning" until findings are actually loaded.
  const backendSaysCompleted = polledStatus === "completed" || localScanState === "completed";
  const dataFullyReady = backendSaysCompleted && !findingsStillFetching;

  // Derive effective scan state: local state wins during transitional periods
  const isScanning =
    localScanState === "starting" ||
    localScanState === "running" ||
    polledStatus === "running" ||
    polledStatus === "pending" ||
    polledStatus === "starting";

  const isCompleted = dataFullyReady;

  // SCANNING badge visible once a scan has ever been triggered
  const showBadge = isScanning || isCompleted;

  // Sync state cleanly via useEffect to avoid rendering conflicts
  useEffect(() => {
    // If backend says running, make sure local state matches so we can hold the "running" state
    // until findings are done fetching when it eventually completes.
    if (polledStatus === "running" && localScanState !== "running") {
      useStreamStore.getState().setScanRunning(activeScan?.id);
    }
    
    // Only transition to completed if:
    // 1. The backend says completed (or failed)
    // 2. We are locally tracking a running scan
    // 3. The scan that completed IS the scan we were tracking (prevents completing due to old scan data)
    // 4. Findings are fully fetched
    if (
      (polledStatus === "completed" || polledStatus === "failed") &&
      localScanState === "running" &&
      localScanId &&
      activeScan?.id === localScanId &&
      !findingsStillFetching
    ) {
      useStreamStore.getState().setScanCompleted();
    }
  }, [polledStatus, localScanState, findingsStillFetching, activeScan?.id, localScanId]);

  // For ScanControls, derive stable status
  const scanStatus = isScanning ? "running" as const : polledStatus;
  const scanId = activeScan?.id ?? null;
  const displayTarget = localScanTarget ?? activeScan?.targetUrl;

  return (
    <div className="flex items-center gap-3 px-3 py-2" role="toolbar" aria-label="Scan controls toolbar">
      <div className="flex-1 max-w-xl relative flex flex-col justify-center">
        <TargetInput 
          {...register("targetUrl")} 
          value={targetUrl || ""}
          className={cn("w-full", errors.targetUrl && "border-risk focus-within:border-risk focus-within:ring-risk/20")} 
        />
      </div>
      <ScanControls targetUrl={normalizeUrl(targetUrl)} scanId={scanId} scanStatus={scanStatus} />
      {/* SCANNING / COMPLETED badge — ALWAYS visible once a scan starts, never disappears */}
      {showBadge && (
        <div
          role="status"
          aria-live="polite"
          aria-label={isScanning ? "Scan in progress" : "Scan completed"}
          className={cn(
          "ml-auto flex items-center gap-2 px-2.5 py-1 rounded-md border transition-colors duration-500",
          isScanning
            ? "bg-primary/10 border-primary/20"
            : "bg-emerald-500/10 border-emerald-500/20"
        )}>
          {isScanning ? (
            <Loader2 className="h-3 w-3 animate-spin text-primary" />
          ) : (
            <CheckCircle2 className="h-3 w-3 text-emerald-400" />
          )}
          <span className={cn(
            "font-semibold text-[11px]",
            isScanning ? "text-primary" : "text-emerald-400"
          )}>
            {isScanning
              ? (localScanState === "starting" ? "Starting" : "Scanning")
              : "Completed"}
          </span>
          {displayTarget && (
            <span className={cn(
              "text-[10px] font-mono max-w-[160px] truncate",
              isScanning ? "text-primary/60" : "text-emerald-400/60"
            )}>{displayTarget}</span>
          )}
        </div>
      )}
    </div>
  );
}
