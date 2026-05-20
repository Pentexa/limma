"use client";

import { cn } from "@/shared/lib/utils";
import { Badge } from "@/shared/ui/badge";
import { getSeverityBgColor, formatSeverity } from "../lib/severity-utils";
import type { Finding } from "../model/types";

interface FindingDetailProps {
  finding: Finding;
  className?: string;
}

export function FindingDetail({ finding, className }: FindingDetailProps) {
  return (
    <div className={cn("space-y-4", className)}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <h3 className="font-semibold text-foreground">{finding.title}</h3>
          <p className="text-sm text-muted-foreground mt-1">{finding.description}</p>
        </div>
        <Badge className={cn("shrink-0", getSeverityBgColor(finding.severity))}>
          {formatSeverity(finding.severity)}
        </Badge>
      </div>

      <div className="grid grid-cols-2 gap-3 text-sm">
        <div>
          <span className="text-muted-foreground">Detector</span>
          <p className="font-medium text-foreground">{finding.detector.toUpperCase()}</p>
        </div>
        <div>
          <span className="text-muted-foreground">CWE</span>
          <p className="font-medium text-foreground">{finding.cwe}</p>
        </div>
        <div>
          <span className="text-muted-foreground">Confidence</span>
          <p className="font-medium text-foreground capitalize">{finding.confidence}</p>
        </div>
        <div>
          <span className="text-muted-foreground">CVSS</span>
          <p className="font-medium text-foreground">{finding.cvss ?? "N/A"}</p>
        </div>
      </div>

      <div className="space-y-2">
        <span className="text-sm text-muted-foreground">URL</span>
        <p className="text-sm font-mono bg-muted/50 px-3 py-2 rounded-md break-all">
          {finding.url}
        </p>
      </div>

      {finding.payload && (
        <div className="space-y-2">
          <span className="text-sm text-muted-foreground">Payload</span>
          <pre className="text-xs font-mono bg-muted/50 px-3 py-2 rounded-md overflow-x-auto">
            {finding.payload}
          </pre>
        </div>
      )}

      {finding.evidence.length > 0 && (
        <div className="space-y-2">
          <span className="text-sm text-muted-foreground">Evidence</span>
          <ul className="space-y-1">
            {finding.evidence.map((e, i) => (
              <li key={i} className="text-sm text-foreground flex items-start gap-2">
                <span className="text-evidence-strong mt-1">●</span>
                {e}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
