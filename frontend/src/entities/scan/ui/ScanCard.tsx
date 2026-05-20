"use client";

import { cn } from "@/shared/lib/utils";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { Badge } from "@/shared/ui/badge";
import { formatScanStatus, getScanStatusColor } from "../lib/format-scan-status";
import type { Scan } from "../model/types";
import { formatRelativeTime } from "@/shared/lib/format-date";
import { Globe, Clock } from "lucide-react";

interface ScanCardProps {
  scan: Scan;
  onClick?: (scan: Scan) => void;
  className?: string;
}

export function ScanCard({ scan, onClick, className }: ScanCardProps) {
  return (
    <Card
      className={cn(
        "cursor-pointer transition-all duration-200 hover:border-primary/30 hover:shadow-[var(--shadow-glow-blue)]",
        className
      )}
      onClick={() => onClick?.(scan)}
    >
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium truncate">
            {scan.targetUrl}
          </CardTitle>
          <Badge
            variant="outline"
            className={cn("text-xs", getScanStatusColor(scan.status))}
          >
            {formatScanStatus(scan.status)}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <div className="flex items-center gap-4 text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            <Globe className="h-3 w-3" />
            {scan.targetUrl.replace(/https?:\/\//, "").split("/")[0]}
          </span>
          {scan.startedAt && (
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {formatRelativeTime(scan.startedAt)}
            </span>
          )}
        </div>
        {scan.result && (
          <div className="flex gap-2 text-xs">
            {scan.result.criticalCount > 0 && (
              <span className="text-critical font-medium">{scan.result.criticalCount} Critical</span>
            )}
            {scan.result.highCount > 0 && (
              <span className="text-high font-medium">{scan.result.highCount} High</span>
            )}
            {scan.result.mediumCount > 0 && (
              <span className="text-medium">{scan.result.mediumCount} Med</span>
            )}
            {scan.result.lowCount > 0 && (
              <span className="text-low">{scan.result.lowCount} Low</span>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
