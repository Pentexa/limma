import React from "react";
import { ShieldAlert } from "lucide-react";
import { cn } from "@/shared/lib/utils";

interface WafBadgeProps {
  wafName: string;
  className?: string;
}

export function WafBadge({ wafName, className }: WafBadgeProps) {
  if (!wafName) return null;
  
  return (
    <div 
      className={cn(
        "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-bold uppercase tracking-wider",
        "bg-orange-500/15 text-orange-500 border border-orange-500/30",
        "animate-pulse shadow-[0_0_10px_rgba(249,115,22,0.2)]",
        className
      )}
      title="Web Application Firewall Detected"
    >
      <ShieldAlert className="w-3.5 h-3.5" />
      <span>WAF: {wafName}</span>
    </div>
  );
}
