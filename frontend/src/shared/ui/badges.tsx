import { cn } from "@/shared/lib/utils";

export function SeverityBadge({ severity }: { severity: string }) {
  return (
    <span className={cn("sev-badge", `sev-badge-${severity.toLowerCase()}`)}>
      {severity}
    </span>
  );
}

export function StatusBadge({ status, icon: Icon }: { status: string; icon?: React.ElementType }) {
  const s = status.toLowerCase();
  const isVerified = s === "verified" || s === "completed";
  const isError = s === "error" || s === "failed";
  const isRunning = s === "running";
  
  return (
    <span className={cn(
      "inline-flex items-center gap-1 text-[10px] font-semibold capitalize",
      isVerified ? "status-verified" : 
      isError ? "text-risk" : 
      isRunning ? "text-primary" : "text-muted-foreground"
    )}>
      {Icon && <Icon className={cn("h-3 w-3", isRunning && "animate-spin")} />}
      {!Icon && status}
    </span>
  );
}
